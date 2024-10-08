use std::{collections::HashMap, fmt::Display};

use wesl_parse::{
    span::Spanned,
    syntax::{
        Alias, CompoundStatement, ConstAssert, Declaration, Expression, FormalTemplateParameter,
        Function, GlobalDeclaration, IdentifierExpression, Module, ModuleMemberDeclaration,
        PathPart, Statement, Struct, TranslationUnit, TypeExpression,
    },
};
use wesl_types::{builtins, mangling::mangle_template_args, CompilerPass};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
struct AliasPath(im::Vector<PathPart>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Dealiaser;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct ModulePath(im::Vector<PathPart>);

#[derive(Debug)]
enum AliasEntry {
    Leaf(AliasPath),
    Node(Box<AliasTree>),
}

impl Display for AliasEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AliasEntry::Leaf(alias_path) => {
                write!(
                    f,
                    "{}",
                    alias_path
                        .0
                        .iter()
                        .map(|x| format!("{x}"))
                        .collect::<Vec<String>>()
                        .join("::")
                )
            }
            AliasEntry::Node(alias_tree) => {
                let result = format!("{}", &alias_tree);
                write!(f, "{}", result.replace('\n', "\n    "))
            }
        }
    }
}

#[derive(Debug, Default)]
struct AliasTree(HashMap<PathPart, AliasEntry>);

impl Display for AliasTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n{}",
            self.0
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl AliasTree {
    fn add(&mut self, mut key: AliasPath, value: AliasPath) {
        if let Some(fst) = key.0.pop_front() {
            match self.0.entry(fst).or_insert_with(|| {
                if key.0.is_empty() {
                    AliasEntry::Leaf(value.clone())
                } else {
                    AliasEntry::Node(Box::new(AliasTree(HashMap::new())))
                }
            }) {
                AliasEntry::Leaf(_) => {}
                AliasEntry::Node(alias_tree) => {
                    alias_tree.add(key, value);
                }
            }
        }
    }

    fn resolve(&self, mut current: AliasPath, path: &mut AliasPath) -> bool {
        if let Some(fst) = path.0.pop_front() {
            current.0.push_back(fst.clone());
            if let Some(entry) = self.0.get(&fst) {
                match entry {
                    AliasEntry::Leaf(alias_path) => {
                        let mut new_path = alias_path.clone();
                        new_path.0.append(path.0.clone());
                        path.0 = new_path.0;
                        return true;
                    }
                    AliasEntry::Node(alias_tree) => {
                        return alias_tree.resolve(current, path);
                    }
                }
            } else {
                current.0.append(path.0.clone());
                path.0 = current.0;
                return false;
            }
        } else {
            path.0 = current.0;
            return false;
        }
    }

    fn resolve_root(&self, path: &mut AliasPath) {
        while self.resolve(AliasPath::default(), path) {}
    }
}

impl AliasPath {
    fn normalize(&mut self) {
        if self.0.len() == 1 {
            let item: &String = &self.0.get(0).unwrap().name.value;
            let builtin_tokens = builtins::get_builtin_tokens();
            let builtin_functions = builtins::get_builtin_functions();
            if builtin_tokens.type_aliases.contains_key(item)
                || builtin_tokens.type_generators.contains(item)
                || builtin_tokens.interpolation_type_names.contains(item)
                || builtin_functions.functions.contains_key(item)
            {
                for p in self.0.iter_mut() {
                    for arg in p.template_args.iter_mut().flatten() {
                        if let Ok(path) = TryInto::<Spanned<Vec<PathPart>>>::try_into(
                            arg.expression.value.clone(),
                        ) {
                            let mut result: AliasPath = AliasPath(path.into_iter().collect());
                            result.normalize();
                            *arg.expression = Expression::Identifier(IdentifierExpression {
                                path: Spanned::new(
                                    result.0.into_iter().collect(),
                                    arg.expression.span(),
                                ),
                            })
                        }
                    }
                }
                return;
            }
        }
        for p in self.0.iter_mut() {
            p.name.value = mangle_template_args(p);
            p.template_args = None;
        }
    }
}

impl Dealiaser {
    fn add_alias_to_tree(mut module_path: ModulePath, alias: &Alias, tree: &mut AliasTree) {
        module_path.0.push_back(PathPart {
            name: alias.name.clone(),
            template_args: None,
            inline_template_args: None,
        });
        let mut alias_path = AliasPath(module_path.0.iter().cloned().collect());
        let mut target_path = AliasPath(alias.typ.path.value.iter().cloned().collect());
        target_path.normalize();
        alias_path.normalize();
        tree.add(alias_path, target_path);
    }

    fn populate_aliases_from_module(
        module: &mut Module,
        mut module_path: ModulePath,
        tree: &mut AliasTree,
    ) {
        module_path.0.push_back(PathPart {
            name: module.name.clone(),
            template_args: None,
            inline_template_args: None,
        });

        let mut others = vec![];
        for decl in module.members.drain(..) {
            let span = decl.span();
            assert!(decl.template_parameters().is_none());
            match decl.value {
                ModuleMemberDeclaration::Alias(alias) => {
                    Self::add_alias_to_tree(module_path.clone(), &alias, tree);
                }
                ModuleMemberDeclaration::Module(mut module) => {
                    Self::populate_aliases_from_module(&mut module, module_path.clone(), tree);
                    others.push(Spanned::new(ModuleMemberDeclaration::Module(module), span));
                }
                other => {
                    others.push(Spanned::new(other, span));
                }
            }
        }
        module.members.append(&mut others);
    }

    fn populate_aliases_from_translation_unit(
        translation_unit: &mut TranslationUnit,
        tree: &mut AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let module_path = ModulePath(im::Vector::new());
        let mut others = vec![];
        for decl in translation_unit.global_declarations.drain(..) {
            let span = decl.span();
            match decl.value {
                GlobalDeclaration::Alias(alias) => {
                    Self::add_alias_to_tree(module_path.clone(), &alias, tree);
                }
                GlobalDeclaration::Module(mut module) if module.template_parameters.is_empty() => {
                    Self::populate_aliases_from_module(&mut module, module_path.clone(), tree);
                    others.push(Spanned::new(GlobalDeclaration::Module(module), span));
                }
                other => {
                    others.push(Spanned::new(other, span));
                }
            }
        }
        translation_unit.global_declarations.append(&mut others);
        Ok(())
    }

    fn replace_alias_usages_from_module(
        module: &mut Module,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for decl in module.members.iter_mut() {
            match decl.as_mut() {
                ModuleMemberDeclaration::Void => {
                    // NO ACTION REQUIRED REQUIRED
                }
                ModuleMemberDeclaration::Declaration(decl) => {
                    Self::replace_alias_usages_from_decl(decl, tree)?;
                }
                ModuleMemberDeclaration::Alias(_) => {
                    panic!("INVARIANT FAILURE. EXPECTED ALIASES TO HAVE ALL BEEN REMOVED BY NOW");
                }
                ModuleMemberDeclaration::Struct(s) => {
                    Self::replace_alias_usages_from_struct(s, tree)?;
                }
                ModuleMemberDeclaration::Function(f) => {
                    Self::replace_alias_usages_from_function(f, tree)?;
                }
                ModuleMemberDeclaration::ConstAssert(assrt) => {
                    Self::replace_alias_usages_from_const_assert(assrt, tree)?;
                }
                ModuleMemberDeclaration::Module(m) => {
                    Self::replace_alias_usages_from_module(m, tree)?;
                }
            }
        }
        Ok(())
    }

    fn replace_alias_usages_from_expr(
        expr: &mut Expression,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        match expr {
            Expression::Literal(_) => {
                // No action required
            }
            Expression::Parenthesized(spanned) => {
                Self::replace_alias_usages_from_expr(spanned, tree)?;
            }
            Expression::NamedComponent(named_component_expression) => {
                Self::replace_alias_usages_from_expr(&mut named_component_expression.base, tree)?;
            }
            Expression::Indexing(indexing_expression) => {
                Self::replace_alias_usages_from_expr(&mut indexing_expression.base, tree)?;
            }
            Expression::Unary(unary_expression) => {
                Self::replace_alias_usages_from_expr(&mut unary_expression.operand, tree)?;
            }
            Expression::Binary(binary_expression) => {
                Self::replace_alias_usages_from_expr(&mut binary_expression.left, tree)?;
                Self::replace_alias_usages_from_expr(&mut binary_expression.right, tree)?;
            }
            Expression::FunctionCall(function_call_expression) => {
                Self::replace_path_with_alias(&mut function_call_expression.path, tree)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::replace_alias_usages_from_expr(arg, tree)?;
                }
            }
            Expression::Identifier(identifier_expression) => {
                Self::replace_path_with_alias(&mut identifier_expression.path, tree)?;
            }
            Expression::Type(type_expression) => {
                Self::replace_alias_usages_from_type(type_expression, tree)?;
            }
        }
        Ok(())
    }

    fn replace_alias_usages_from_statement(
        statement: &mut Statement,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        match statement {
            Statement::Void => {
                // No action required
            }
            Statement::Compound(compound_statement) => {
                Self::replace_alias_usages_from_compound_statement(compound_statement, tree)?;
            }
            Statement::Assignment(assignment_statement) => {
                Self::replace_alias_usages_from_expr(&mut assignment_statement.lhs, tree)?;
                Self::replace_alias_usages_from_expr(&mut assignment_statement.rhs, tree)?;
            }
            Statement::Increment(expression) => {
                Self::replace_alias_usages_from_expr(expression, tree)?;
            }
            Statement::Decrement(expression) => {
                Self::replace_alias_usages_from_expr(expression, tree)?;
            }
            Statement::If(iff) => {
                Self::replace_alias_usages_from_expr(&mut iff.if_clause.0, tree)?;
                Self::replace_alias_usages_from_compound_statement(&mut iff.if_clause.1, tree)?;
                for (else_if_expr, else_if_statements) in iff.else_if_clauses.iter_mut() {
                    Self::replace_alias_usages_from_expr(else_if_expr, tree)?;
                    Self::replace_alias_usages_from_compound_statement(else_if_statements, tree)?;
                }
                if let Some(else_clause) = iff.else_clause.as_mut() {
                    Self::replace_alias_usages_from_compound_statement(else_clause, tree)?;
                }
            }
            Statement::Switch(s) => {
                Self::replace_alias_usages_from_expr(&mut s.expression, tree)?;
                for clause in s.clauses.iter_mut() {
                    for c in clause.case_selectors.iter_mut() {
                        match &mut c.value {
                            wesl_parse::syntax::CaseSelector::Default => {
                                // NO ACTION NEEDED
                            }
                            wesl_parse::syntax::CaseSelector::Expression(e) => {
                                Self::replace_alias_usages_from_expr(e, tree)?;
                            }
                        }
                    }
                    Self::replace_alias_usages_from_compound_statement(&mut clause.body, tree)?;
                }
            }
            Statement::Loop(l) => {
                Self::replace_alias_usages_from_compound_statement(&mut l.body, tree)?;
                if let Some(cont) = l.continuing.as_mut() {
                    Self::replace_alias_usages_from_compound_statement(&mut l.body, tree)?;
                    if let Some(expr) = cont.break_if.as_mut() {
                        Self::replace_alias_usages_from_expr(expr, tree)?;
                    }
                }
            }
            Statement::For(f) => {
                if let Some(init) = f.initializer.as_mut() {
                    Self::replace_alias_usages_from_statement(init.as_mut(), tree)?;
                }
                if let Some(cond) = f.condition.as_mut() {
                    Self::replace_alias_usages_from_expr(cond, tree)?;
                }
                if let Some(update) = f.update.as_mut() {
                    Self::replace_alias_usages_from_statement(update.as_mut(), tree)?;
                }
                Self::replace_alias_usages_from_compound_statement(&mut f.body, tree)?;
            }
            Statement::While(w) => {
                Self::replace_alias_usages_from_expr(&mut w.condition, tree)?;
                Self::replace_alias_usages_from_compound_statement(&mut w.body, tree)?;
            }
            Statement::Break => {
                // No action required
            }
            Statement::Continue => {
                // No action required
            }
            Statement::Return(spanned) => {
                if let Some(expr) = spanned.as_mut() {
                    Self::replace_alias_usages_from_expr(expr, tree)?;
                }
            }
            Statement::Discard => {
                // No action required
            }
            Statement::FunctionCall(function_call_expression) => {
                Self::replace_path_with_alias(&mut function_call_expression.path, tree)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::replace_alias_usages_from_expr(arg, tree)?;
                }
            }
            Statement::ConstAssert(const_assert) => {
                Self::replace_alias_usages_from_const_assert(const_assert, tree)?;
            }
            Statement::Declaration(declaration_statement) => {
                Self::replace_alias_usages_from_decl(&mut declaration_statement.declaration, tree)?;
                for statement in declaration_statement.statements.iter_mut() {
                    Self::replace_alias_usages_from_statement(statement, tree)?;
                }
            }
        }
        Ok(())
    }

    fn replace_path_with_alias(
        mutable_path: &mut Spanned<Vec<PathPart>>,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let mut path = AliasPath(mutable_path.value.drain(..).collect());
        path.normalize();
        for p in path.0.iter_mut() {
            for arg in p.template_args.iter_mut().flatten() {
                Self::replace_alias_usages_from_expr(&mut arg.expression, tree)?;
            }
        }

        tree.resolve_root(&mut path);
        mutable_path.value = path.0.into_iter().collect();

        Ok(())
    }

    fn replace_alias_usages_from_type(
        expr: &mut TypeExpression,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Self::replace_path_with_alias(&mut expr.path, tree)?;
        Ok(())
    }

    fn replace_alias_usages_from_decl(
        decl: &mut Declaration,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        if let Some(init) = decl.initializer.as_mut() {
            Self::replace_alias_usages_from_expr(init.as_mut(), tree)?;
        }

        if let Some(typ) = decl.typ.as_mut() {
            Self::replace_alias_usages_from_type(typ, tree)?;
        }

        Ok(())
    }

    fn replace_alias_usages_from_struct(
        strct: &mut Struct,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for m in strct.members.iter_mut() {
            Self::replace_alias_usages_from_type(&mut m.typ, tree)?;
        }
        Ok(())
    }

    fn replace_alias_usages_from_template_params(
        params: &mut Vec<Spanned<FormalTemplateParameter>>,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for p in params {
            if let Some(def) = p.default_value.as_mut() {
                Self::replace_alias_usages_from_expr(def, tree)?;
            }
        }
        Ok(())
    }

    fn replace_alias_usages_from_compound_statement(
        statement: &mut CompoundStatement,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for statement in statement.statements.iter_mut() {
            Self::replace_alias_usages_from_statement(statement.as_mut(), tree)?;
        }
        Ok(())
    }

    fn replace_alias_usages_from_function(
        func: &mut Function,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        if let Some(r) = func.return_type.as_mut() {
            Self::replace_alias_usages_from_type(r, tree)?;
        }
        Self::replace_alias_usages_from_template_params(&mut func.template_parameters, tree)?;

        for p in func.parameters.iter_mut() {
            Self::replace_alias_usages_from_type(&mut p.typ, tree)?;
        }

        if let Some(ret) = func.return_type.as_mut() {
            Self::replace_alias_usages_from_type(&mut ret.value, tree)?;
        }

        Self::replace_alias_usages_from_compound_statement(&mut func.body, tree)?;
        Ok(())
    }

    fn replace_alias_usages_from_const_assert(
        assrt: &mut ConstAssert,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Self::replace_alias_usages_from_expr(&mut assrt.expression, tree)?;
        Ok(())
    }

    fn replace_alias_usages_from_translation_unit(
        translation_unit: &mut TranslationUnit,
        tree: &AliasTree,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for decl in translation_unit.global_declarations.iter_mut() {
            match decl.as_mut() {
                GlobalDeclaration::Void => {
                    // NO ACTION REQUIRED
                }
                GlobalDeclaration::Declaration(decl) => {
                    Self::replace_alias_usages_from_decl(decl, tree)?;
                }
                GlobalDeclaration::Alias(_) => {
                    panic!("INVARIANT FAILURE. EXPECTED ALIASES TO HAVE ALL BEEN REMOVED BY NOW");
                }
                GlobalDeclaration::Struct(s) => {
                    Self::replace_alias_usages_from_struct(s, tree)?;
                }
                GlobalDeclaration::Function(f) => {
                    Self::replace_alias_usages_from_function(f, tree)?;
                }
                GlobalDeclaration::ConstAssert(assrt) => {
                    Self::replace_alias_usages_from_const_assert(assrt, tree)?;
                }
                GlobalDeclaration::Module(m) => {
                    Self::replace_alias_usages_from_module(m, tree)?;
                }
            }
        }
        Ok(())
    }
}

impl CompilerPass for Dealiaser {
    fn apply_mut(
        &mut self,
        translation_unit: &mut wesl_parse::syntax::TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let mut tree = AliasTree::default();
        Self::populate_aliases_from_translation_unit(translation_unit, &mut tree)?;
        Self::replace_alias_usages_from_translation_unit(translation_unit, &tree)?;
        Ok(())
    }
}
