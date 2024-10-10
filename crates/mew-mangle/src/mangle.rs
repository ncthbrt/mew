use mew_parse::{
    span::Spanned,
    syntax::{
        Alias, CompoundStatement, ConstAssert, Declaration, Expression, Function,
        GlobalDeclaration, IdentifierExpression, Module, ModuleMemberDeclaration, PathPart,
        Statement, Struct, TranslationUnit, TypeExpression,
    },
};
use mew_types::CompilerPass;

#[derive(Debug, Default, Clone, Copy)]
pub struct Mangler;

#[derive(Debug, PartialEq, Clone)]
struct ModulePath(im::Vector<PathPart>);

impl Mangler {
    fn mangle_path(path: &mut Vec<PathPart>) {
        let mut result = Vec::new();
        let first = path.first();
        let last = path.last();
        let mut mangled_span = 0..0;
        if let (Some(first), Some(last)) = (first, last) {
            let first_name_span = first.name.span();
            let last_name_span = last.name.span();
            let mut end = last_name_span.end;
            let start = first_name_span.start;
            if let Some(last) = last.template_args.as_ref().and_then(|x| x.last()) {
                end = last.span().end;
            }
            mangled_span = start..end;
        };
        for p in path.iter_mut() {
            let mut current = String::new();
            current.push_str(p.name.replace('_', "__").as_str());
            if let Some(args) = p.template_args.as_mut() {
                for arg in args.iter_mut() {
                    current.push_str("__");
                    Self::mangle_expression(&mut arg.expression);
                    current.push_str(format!("{}", arg.expression).as_str());
                }
            }
            result.push(current);
        }
        let joined = result.join("_");
        path.clear();
        path.push(PathPart {
            name: Spanned::new(joined, mangled_span),
            template_args: None,
            inline_template_args: None,
        });
    }

    fn mangle_name(name: &mut String, path: ModulePath) {
        let mut path: Vec<PathPart> = path.0.into_iter().collect();
        Self::mangle_path(&mut path);
        let mut result = String::new();
        result.push_str(path[0].name.value.as_str());
        if !result.is_empty() {
            result.push('_');
        }
        result.push_str(&name.replace('_', "__"));
        *name = result;
    }

    fn mangle_compound(compound: &mut CompoundStatement) {
        for c in compound.statements.iter_mut() {
            Self::mangle_statement(c);
        }
    }

    fn mangle_statement(statement: &mut Statement) {
        match statement {
            Statement::Void => {
                // DO NOTHING
            }
            Statement::Compound(c) => Self::mangle_compound(c),
            Statement::Assignment(a) => {
                Self::mangle_expression(&mut a.lhs);
                Self::mangle_expression(&mut a.rhs);
            }
            Statement::Increment(i) => {
                Self::mangle_expression(i);
            }
            Statement::Decrement(d) => {
                Self::mangle_expression(d);
            }
            Statement::If(iff) => {
                Self::mangle_expression(&mut iff.if_clause.0);
                for c in iff.if_clause.1.statements.iter_mut() {
                    Self::mangle_statement(c);
                }
                if let Some(else_clause) = iff.else_clause.as_mut() {
                    for c in else_clause.statements.iter_mut() {
                        Self::mangle_statement(c);
                    }
                }
                for (elif_expr, elif_statment) in iff.else_if_clauses.iter_mut() {
                    for c in elif_statment.statements.iter_mut() {
                        Self::mangle_statement(c);
                    }
                    Self::mangle_expression(elif_expr);
                }
            }
            Statement::Switch(s) => {
                Self::mangle_expression(&mut s.expression);
                for c in s.clauses.iter_mut() {
                    for select in c.case_selectors.iter_mut() {
                        match select.as_mut() {
                            mew_parse::syntax::CaseSelector::Default => {
                                // DO NOTHING
                            }
                            mew_parse::syntax::CaseSelector::Expression(expr) => {
                                Self::mangle_expression(expr);
                            }
                        }
                    }
                    for c in c.body.statements.iter_mut() {
                        Self::mangle_statement(c);
                    }
                }
            }
            Statement::Loop(l) => {
                for c in l.body.statements.iter_mut() {
                    Self::mangle_statement(c);
                }
                if let Some(cont) = l.continuing.as_mut() {
                    for c in cont.body.statements.iter_mut() {
                        Self::mangle_statement(c);
                    }
                    if let Some(break_if) = cont.break_if.as_mut() {
                        Self::mangle_expression(break_if);
                    }
                }
            }
            Statement::For(f) => {
                for c in f.body.statements.iter_mut() {
                    Self::mangle_statement(c);
                }
                if let Some(cond) = f.condition.as_mut() {
                    Self::mangle_expression(cond);
                }
                if let Some(statement) = f.initializer.as_mut() {
                    Self::mangle_statement(statement.as_mut());
                }
                if let Some(update) = f.update.as_mut() {
                    Self::mangle_statement(update.as_mut());
                }
            }
            Statement::While(w) => {
                for c in w.body.statements.iter_mut() {
                    Self::mangle_statement(c);
                }
                Self::mangle_expression(&mut w.condition);
            }
            Statement::Break => {
                // DO NOTHING
            }
            Statement::Continue => {
                // DO NOTHING
            }
            Statement::Return(ret) => {
                if let Some(ret) = ret.as_mut() {
                    Self::mangle_expression(ret);
                }
            }
            Statement::Discard => {
                // DO NOTHING
            }
            Statement::FunctionCall(f) => {
                Self::mangle_path(&mut f.path);
                for arg in f.arguments.iter_mut() {
                    Self::mangle_expression(arg);
                }
            }
            Statement::ConstAssert(assrt) => {
                Self::mangle_expression(&mut assrt.expression);
            }
            Statement::Declaration(decl) => {
                if let Some(typ) = decl.declaration.typ.as_mut() {
                    Self::mangle_type(typ);
                }

                if let Some(init) = decl.declaration.initializer.as_mut() {
                    Self::mangle_expression(init);
                }
                for statement in decl.statements.iter_mut() {
                    Self::mangle_statement(statement);
                }
            }
        }
    }

    fn mangle_expression(expr: &mut Expression) {
        match expr {
            Expression::Literal(_) => {
                // DO NOTHING
            }
            Expression::Parenthesized(p) => {
                Self::mangle_expression(p.as_mut());
            }
            Expression::NamedComponent(n) => {
                Self::mangle_expression(&mut n.base);
            }
            Expression::Indexing(idx) => {
                Self::mangle_expression(&mut idx.base);
                Self::mangle_expression(&mut idx.index);
            }
            Expression::Unary(u) => {
                Self::mangle_expression(&mut u.operand);
            }
            Expression::Binary(b) => {
                Self::mangle_expression(&mut b.left);
                Self::mangle_expression(&mut b.right);
            }
            Expression::FunctionCall(f) => {
                let mut mangle_function_path = true;
                if f.path.len() == 1 {
                    let builtin_functions = mew_types::builtins::get_builtin_functions();
                    mangle_function_path = !builtin_functions
                        .functions
                        .contains_key(&f.path[0].name.value.clone());
                }
                if mangle_function_path {
                    Self::mangle_path(&mut f.path);
                } else if let Some(args) = f.path[0].template_args.as_mut() {
                    for arg in args {
                        Self::mangle_expression(&mut arg.expression);
                    }
                }

                for arg in f.arguments.iter_mut() {
                    Self::mangle_expression(arg);
                }
            }
            Expression::Identifier(id) => {
                Self::mangle_identifier_expression(id);
            }
            Expression::Type(typ) => {
                Self::mangle_type(typ);
            }
        }
    }

    fn mangle_type(typ: &mut TypeExpression) {
        let mut mangle_type_path = true;
        if typ.path.len() == 1 {
            let builtin_tokens = mew_types::builtins::get_builtin_tokens();
            mangle_type_path = !builtin_tokens
                .type_generators
                .contains(&typ.path[0].name.value.clone());
            if mangle_type_path {
                mangle_type_path = !builtin_tokens
                    .type_aliases
                    .contains_key(&typ.path[0].name.value.clone());
            }
        }
        if mangle_type_path {
            Self::mangle_path(&mut typ.path);
        } else if let Some(args) = typ.path[0].template_args.as_mut() {
            for arg in args {
                Self::mangle_expression(&mut arg.expression);
            }
        }
    }
    fn mangle_identifier_expression(id: &mut IdentifierExpression) {
        let mut mangle_type_path = true;
        if id.path.len() == 1 {
            let builtin_tokens = mew_types::builtins::get_builtin_tokens();
            mangle_type_path = !builtin_tokens
                .type_generators
                .contains(&id.path[0].name.value.clone());
            if mangle_type_path {
                mangle_type_path = !builtin_tokens
                    .type_aliases
                    .contains_key(&id.path[0].name.value.clone());
            }
        }
        if mangle_type_path {
            Self::mangle_path(&mut id.path);
        } else if let Some(args) = id.path[0].template_args.as_mut() {
            for arg in args {
                Self::mangle_expression(&mut arg.expression);
            }
        }
    }

    fn mangle_decl(decl: &mut Declaration, path: ModulePath) {
        if let Some(init) = decl.initializer.as_mut() {
            Self::mangle_expression(init);
        }
        if let Some(typ) = decl.typ.as_mut() {
            Self::mangle_type(typ);
        }
        Self::mangle_name(&mut decl.name, path);
    }

    fn mangle_alias(a: &mut Alias, path: ModulePath) {
        Self::mangle_name(&mut a.name, path);
        Self::mangle_type(&mut a.typ);
    }

    fn mangle_struct(s: &mut Struct, path: ModulePath) {
        for member in s.members.iter_mut() {
            Self::mangle_type(&mut member.typ);
        }
        Self::mangle_name(&mut s.name, path);
    }

    fn mangle_func(f: &mut Function, path: ModulePath) {
        Self::mangle_name(&mut f.name, path);

        if let Some(ret) = f.return_type.as_mut() {
            Self::mangle_type(ret);
        }
        for arg in f.parameters.iter_mut() {
            Self::mangle_type(&mut arg.typ);
        }
        for statement in f.body.statements.iter_mut() {
            Self::mangle_statement(statement);
        }
    }

    fn mangle_const_assert(a: &mut ConstAssert) {
        Self::mangle_expression(&mut a.expression);
    }

    fn mangle_module(m: &mut Module, mut path: ModulePath) {
        path.0.push_back(PathPart {
            name: m.name.clone(),
            template_args: None,
            inline_template_args: None,
        });
        for decl in m.members.iter_mut() {
            match decl.as_mut() {
                ModuleMemberDeclaration::Void => {}
                ModuleMemberDeclaration::Declaration(decl) => {
                    Self::mangle_decl(decl, path.clone());
                }
                ModuleMemberDeclaration::Alias(a) => {
                    Self::mangle_alias(a, path.clone());
                }
                ModuleMemberDeclaration::Struct(strct) => {
                    Self::mangle_struct(strct, path.clone());
                }
                ModuleMemberDeclaration::Function(f) => {
                    Self::mangle_func(f, path.clone());
                }
                ModuleMemberDeclaration::ConstAssert(assrt) => {
                    Self::mangle_const_assert(assrt);
                }
                ModuleMemberDeclaration::Module(module) => {
                    Self::mangle_module(module, path.clone());
                }
            }
        }
    }

    fn mangle_translation_unit(translation_unit: &mut TranslationUnit, path: ModulePath) {
        for decl in translation_unit.global_declarations.iter_mut() {
            match decl.as_mut() {
                GlobalDeclaration::Void => {}
                GlobalDeclaration::Declaration(decl) => {
                    Self::mangle_decl(decl, path.clone());
                }
                GlobalDeclaration::Alias(a) => {
                    Self::mangle_alias(a, path.clone());
                }
                GlobalDeclaration::Struct(strct) => {
                    Self::mangle_struct(strct, path.clone());
                }
                GlobalDeclaration::Function(f) => {
                    Self::mangle_func(f, path.clone());
                }
                GlobalDeclaration::ConstAssert(assrt) => {
                    Self::mangle_const_assert(assrt);
                }
                GlobalDeclaration::Module(module) => {
                    Self::mangle_module(module, path.clone());
                }
            }
        }
    }

    pub fn mangle_mut(&self, translation_unit: &mut TranslationUnit) {
        let path = ModulePath(im::Vector::new());
        Self::mangle_translation_unit(translation_unit, path);
    }
}

impl CompilerPass for Mangler {
    fn apply_mut(
        &mut self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), mew_types::CompilerPassError> {
        self.mangle_mut(translation_unit);
        Ok(())
    }
}
