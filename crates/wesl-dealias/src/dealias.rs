use std::collections::HashMap;

use wesl_parse::{
    span::Spanned,
    syntax::{
        Alias, CompoundStatement, ConstAssert, Declaration, Expression, FormalTemplateParameter,
        Function, GlobalDeclaration, IdentifierExpression, Module, ModuleMemberDeclaration,
        PathPart, Statement, Struct, TemplateArg, TranslationUnit, TypeExpression,
    },
};
use wesl_types::CompilerPass;
pub struct Dealias;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct AliasPath(im::Vector<PathPart>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct ModulePath(im::Vector<PathPart>);

impl Dealias {
    fn add_alias_to_cache(
        mut module_path: ModulePath,
        alias: &Alias,
        cache: &mut HashMap<AliasPath, AliasPath>,
    ) {
        module_path.0.push_back(PathPart {
            name: alias.name.clone(),
            template_args: None,
        });
        let alias_path = AliasPath(module_path.0.iter().cloned().collect());
        let target = AliasPath(alias.typ.path.value.iter().cloned().collect());
        cache.insert(alias_path, target);
    }

    fn populate_aliases_from_module(
        module: &mut Module,
        mut module_path: ModulePath,
        cache: &mut HashMap<AliasPath, AliasPath>,
    ) {
        module_path.0.push_back(PathPart {
            name: module.name.clone(),
            template_args: module
                .template_parameters
                .iter()
                .map(|x| {
                    Spanned::new(
                        TemplateArg {
                            expression: Spanned::new(
                                Expression::Identifier(IdentifierExpression {
                                    path: Spanned::new(
                                        vec![PathPart {
                                            name: x.name.clone(),
                                            template_args: None,
                                        }],
                                        x.span(),
                                    ),
                                }),
                                x.span(),
                            ),
                            arg_name: if x.default_value.is_some() {
                                Some(x.name.clone())
                            } else {
                                None
                            },
                        },
                        x.span(),
                    )
                })
                .collect::<Vec<Spanned<TemplateArg>>>()
                .into(),
        });

        let mut others = vec![];
        for decl in module.members.drain(0..) {
            let span = decl.span();
            match decl.value {
                ModuleMemberDeclaration::Alias(alias) => {
                    Self::add_alias_to_cache(module_path.clone(), &alias, cache);
                }
                ModuleMemberDeclaration::Module(mut module) => {
                    Self::populate_aliases_from_module(&mut module, module_path.clone(), cache);
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
        cache: &mut HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let module_path = ModulePath(im::Vector::new());
        let mut others = vec![];
        for decl in translation_unit.global_declarations.drain(0..) {
            let span = decl.span();
            match decl.value {
                GlobalDeclaration::Alias(alias) => {
                    Self::add_alias_to_cache(module_path.clone(), &alias, cache);
                }
                GlobalDeclaration::Module(mut module) => {
                    Self::populate_aliases_from_module(&mut module, module_path.clone(), cache);
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

    fn resolve_aliases_from_cache(
        cache: &mut HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let keys: im::Vector<AliasPath> = cache.keys().cloned().collect();
        for k in keys {
            let mut prev = cache.get(&k).unwrap();
            let mut current = Some(prev);
            while let Some(unwrapped) = current {
                prev = unwrapped;
                current = cache.get(prev);
            }
            cache.insert(k, prev.clone());
        }
        Ok(())
    }

    fn replace_alias_usages_from_module(
        module: &mut Module,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for decl in module.members.iter_mut() {
            match decl.as_mut() {
                ModuleMemberDeclaration::Void => {
                    // NO ACTION REQUIRED REQUIRED
                }
                ModuleMemberDeclaration::Declaration(decl) => {
                    Self::replace_alias_usages_from_decl(decl, cache)?;
                }
                ModuleMemberDeclaration::Alias(_) => {
                    panic!("INVARIANT FAILURE. EXPECTED ALIASES TO HAVE ALL BEEN REMOVED BY NOW");
                }
                ModuleMemberDeclaration::Struct(s) => {
                    Self::replace_alias_usages_from_struct(s, cache)?;
                }
                ModuleMemberDeclaration::Function(f) => {
                    Self::replace_alias_usages_from_function(f, cache)?;
                }
                ModuleMemberDeclaration::ConstAssert(assrt) => {
                    Self::replace_alias_usages_from_const_assert(assrt, cache)?;
                }
                ModuleMemberDeclaration::Module(m) => {
                    Self::replace_alias_usages_from_module(m, cache)?;
                }
            }
        }
        Ok(())
    }

    fn replace_alias_usages_from_expr(
        expr: &mut Expression,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        match expr {
            Expression::Literal(_) => {
                // No action required
            }
            Expression::Parenthesized(spanned) => {
                Self::replace_alias_usages_from_expr(spanned, cache)?;
            }
            Expression::NamedComponent(named_component_expression) => {
                Self::replace_alias_usages_from_expr(&mut named_component_expression.base, cache)?;
            }
            Expression::Indexing(indexing_expression) => {
                Self::replace_alias_usages_from_expr(&mut indexing_expression.base, cache)?;
            }
            Expression::Unary(unary_expression) => {
                Self::replace_alias_usages_from_expr(&mut unary_expression.operand, cache)?;
            }
            Expression::Binary(binary_expression) => {
                Self::replace_alias_usages_from_expr(&mut binary_expression.left, cache)?;
                Self::replace_alias_usages_from_expr(&mut binary_expression.right, cache)?;
            }
            Expression::FunctionCall(function_call_expression) => {
                Self::replace_path_with_alias(&mut function_call_expression.path, cache)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::replace_alias_usages_from_expr(arg, cache)?;
                }
            }
            Expression::Identifier(identifier_expression) => {
                Self::replace_path_with_alias(&mut identifier_expression.path, cache)?;
            }
            Expression::Type(type_expression) => {
                Self::replace_alias_usages_from_type(type_expression, cache)?;
            }
        }
        Ok(())
    }

    fn replace_alias_usages_from_statement(
        statement: &mut Statement,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        match statement {
            Statement::Void => {
                // No action required
            }
            Statement::Compound(compound_statement) => {
                Self::replace_alias_usages_from_compound_statement(compound_statement, cache)?;
            }
            Statement::Assignment(assignment_statement) => {
                Self::replace_alias_usages_from_expr(&mut assignment_statement.lhs, cache)?;
                Self::replace_alias_usages_from_expr(&mut assignment_statement.rhs, cache)?;
            }
            Statement::Increment(expression) => {
                Self::replace_alias_usages_from_expr(expression, cache)?;
            }
            Statement::Decrement(expression) => {
                Self::replace_alias_usages_from_expr(expression, cache)?;
            }
            Statement::If(iff) => {
                Self::replace_alias_usages_from_expr(&mut iff.if_clause.0, cache)?;
                Self::replace_alias_usages_from_compound_statement(&mut iff.if_clause.1, cache)?;
                for (else_if_expr, else_if_statements) in iff.else_if_clauses.iter_mut() {
                    Self::replace_alias_usages_from_expr(else_if_expr, cache)?;
                    Self::replace_alias_usages_from_compound_statement(else_if_statements, cache)?;
                }
                if let Some(else_clause) = iff.else_clause.as_mut() {
                    Self::replace_alias_usages_from_compound_statement(else_clause, cache)?;
                }
            }
            Statement::Switch(s) => {
                Self::replace_alias_usages_from_expr(&mut s.expression, cache)?;
                for clause in s.clauses.iter_mut() {
                    for c in clause.case_selectors.iter_mut() {
                        match &mut c.value {
                            wesl_parse::syntax::CaseSelector::Default => {
                                // NO ACTION NEEDED
                            }
                            wesl_parse::syntax::CaseSelector::Expression(e) => {
                                Self::replace_alias_usages_from_expr(e, cache)?;
                            }
                        }
                    }
                    Self::replace_alias_usages_from_compound_statement(&mut clause.body, cache)?;
                }
            }
            Statement::Loop(l) => {
                Self::replace_alias_usages_from_compound_statement(&mut l.body, cache)?;
                if let Some(cont) = l.continuing.as_mut() {
                    Self::replace_alias_usages_from_compound_statement(&mut l.body, cache)?;
                    if let Some(expr) = cont.break_if.as_mut() {
                        Self::replace_alias_usages_from_expr(expr, cache)?;
                    }
                }
            }
            Statement::For(f) => {
                if let Some(init) = f.initializer.as_mut() {
                    Self::replace_alias_usages_from_statement(init.as_mut(), cache)?;
                }
                if let Some(cond) = f.condition.as_mut() {
                    Self::replace_alias_usages_from_expr(cond, cache)?;
                }
                if let Some(update) = f.update.as_mut() {
                    Self::replace_alias_usages_from_statement(update.as_mut(), cache)?;
                }
                Self::replace_alias_usages_from_compound_statement(&mut f.body, cache)?;
            }
            Statement::While(w) => {
                Self::replace_alias_usages_from_expr(&mut w.condition, cache)?;
                Self::replace_alias_usages_from_compound_statement(&mut w.body, cache)?;
            }
            Statement::Break => {
                // No action required
            }
            Statement::Continue => {
                // No action required
            }
            Statement::Return(spanned) => {
                if let Some(expr) = spanned.as_mut() {
                    Self::replace_alias_usages_from_expr(expr, cache)?;
                }
            }
            Statement::Discard => {
                // No action required
            }
            Statement::FunctionCall(function_call_expression) => {
                Self::replace_path_with_alias(&mut function_call_expression.path, cache)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::replace_alias_usages_from_expr(arg, cache)?;
                }
            }
            Statement::ConstAssert(const_assert) => {
                Self::replace_alias_usages_from_const_assert(const_assert, cache)?;
            }
            Statement::Declaration(declaration_statement) => {
                Self::replace_alias_usages_from_decl(
                    &mut declaration_statement.declaration,
                    cache,
                )?;
            }
        }
        Ok(())
    }

    fn replace_path_with_alias(
        mutable_path: &mut Spanned<Vec<PathPart>>,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let path: im::Vector<PathPart> = mutable_path.value.clone().into();
        if let Some(path) = cache.get(&AliasPath(path)) {
            mutable_path.value.clear();
            mutable_path.value.extend(path.0.iter().cloned());
        };
        Ok(())
    }

    fn replace_alias_usages_from_type(
        expr: &mut TypeExpression,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Self::replace_path_with_alias(&mut expr.path, cache)?;
        Ok(())
    }

    fn replace_alias_usages_from_decl(
        decl: &mut Declaration,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        if let Some(init) = decl.initializer.as_mut() {
            Self::replace_alias_usages_from_expr(init.as_mut(), cache)?;
        }

        if let Some(typ) = decl.typ.as_mut() {
            Self::replace_alias_usages_from_type(typ, cache)?;
        }

        Ok(())
    }

    fn replace_alias_usages_from_struct(
        strct: &mut Struct,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for m in strct.members.iter_mut() {
            Self::replace_alias_usages_from_type(&mut m.typ, cache)?;
        }
        Ok(())
    }

    fn replace_alias_usages_from_template_params(
        params: &mut Vec<Spanned<FormalTemplateParameter>>,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for p in params {
            if let Some(def) = p.default_value.as_mut() {
                Self::replace_alias_usages_from_expr(def, cache)?;
            }
        }
        Ok(())
    }

    fn replace_alias_usages_from_compound_statement(
        statement: &mut CompoundStatement,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for statement in statement.statements.iter_mut() {
            Self::replace_alias_usages_from_statement(statement.as_mut(), cache)?;
        }
        Ok(())
    }

    fn replace_alias_usages_from_function(
        func: &mut Function,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        if let Some(r) = func.return_type.as_mut() {
            Self::replace_alias_usages_from_type(r, cache)?;
        }
        Self::replace_alias_usages_from_template_params(&mut func.template_parameters, cache)?;

        for p in func.parameters.iter_mut() {
            Self::replace_alias_usages_from_type(&mut p.typ, cache)?;
        }

        Self::replace_alias_usages_from_compound_statement(&mut func.body, cache)?;
        Ok(())
    }

    fn replace_alias_usages_from_const_assert(
        assrt: &mut ConstAssert,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Self::replace_alias_usages_from_expr(&mut assrt.expression, cache)?;
        Ok(())
    }

    fn replace_alias_usages_from_translation_unit(
        translation_unit: &mut TranslationUnit,
        cache: &HashMap<AliasPath, AliasPath>,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for decl in translation_unit.global_declarations.iter_mut() {
            match decl.as_mut() {
                GlobalDeclaration::Void => {
                    // NO ACTION REQUIRED REQUIRED
                }
                GlobalDeclaration::Declaration(decl) => {
                    Self::replace_alias_usages_from_decl(decl, cache)?;
                }
                GlobalDeclaration::Alias(_) => {
                    panic!("INVARIANT FAILURE. EXPECTED ALIASES TO HAVE ALL BEEN REMOVED BY NOW");
                }
                GlobalDeclaration::Struct(s) => {
                    Self::replace_alias_usages_from_struct(s, cache)?;
                }
                GlobalDeclaration::Function(f) => {
                    Self::replace_alias_usages_from_function(f, cache)?;
                }
                GlobalDeclaration::ConstAssert(assrt) => {
                    Self::replace_alias_usages_from_const_assert(assrt, cache)?;
                }
                GlobalDeclaration::Module(m) => {
                    Self::replace_alias_usages_from_module(m, cache)?;
                }
            }
        }
        Ok(())
    }
}

impl CompilerPass for Dealias {
    fn apply_mut(
        &mut self,
        translation_unit: &mut wesl_parse::syntax::TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let mut cache: HashMap<AliasPath, AliasPath> = HashMap::new();
        Self::populate_aliases_from_translation_unit(translation_unit, &mut cache)?;
        Self::resolve_aliases_from_cache(&mut cache)?;
        Self::replace_alias_usages_from_translation_unit(translation_unit, &cache)?;
        Ok(())
    }
}
