use wesl_parse::syntax::{
    Alias, ConstAssert, Declaration, Expression, Function, GlobalDeclaration, Module,
    ModuleMemberDeclaration, Statement, Struct, TranslationUnit, TypeExpression,
};
use wesl_types::CompilerPass;

#[derive(Debug, Default, Clone, Copy)]
pub struct Mangler;

#[derive(Debug, PartialEq, Clone)]
struct ModulePath(im::Vector<String>);

impl Mangler {
    fn mangle_path(path: &mut Vec<String>) {
        for p in path.iter_mut() {
            *p = p.replace('_', "__");
        }
        let joined = path.join("_");
        path.clear();
        path.push(joined);
    }

    fn mangle_name(name: &mut String, path: ModulePath) {
        let mut result = String::new();
        for p in path.0.iter() {
            result.push_str(&p.replace('_', "__"));
            result.push('_');
        }
        result.push_str(&name.replace('_', "__"));
        *name = result;
    }

    fn mangle_statement(statement: &mut Statement, path: ModulePath) {
        match statement {
            Statement::Void => {
                // DO NOTHING
            }
            Statement::Compound(c) => {
                for c in c.statements.iter_mut() {
                    Self::mangle_statement(c, path.clone());
                }
            }
            Statement::Assignment(a) => {
                Self::mangle_expression(&mut a.lhs, path.clone());
                Self::mangle_expression(&mut a.rhs, path);
            }
            Statement::Increment(i) => {
                Self::mangle_expression(i, path);
            }
            Statement::Decrement(d) => {
                Self::mangle_expression(d, path);
            }
            Statement::If(iff) => {
                Self::mangle_expression(&mut iff.if_clause.0, path.clone());
                for c in iff.if_clause.1.statements.iter_mut() {
                    Self::mangle_statement(c, path.clone());
                }
                if let Some(else_clause) = iff.else_clause.as_mut() {
                    for c in else_clause.statements.iter_mut() {
                        Self::mangle_statement(c, path.clone());
                    }
                }
                for (elif_expr, elif_statment) in iff.else_if_clauses.iter_mut() {
                    for c in elif_statment.statements.iter_mut() {
                        Self::mangle_statement(c, path.clone());
                    }
                    Self::mangle_expression(elif_expr, path.clone());
                }
            }
            Statement::Switch(s) => {
                Self::mangle_expression(&mut s.expression, path.clone());
                for c in s.clauses.iter_mut() {
                    for select in c.case_selectors.iter_mut() {
                        match select {
                            wesl_parse::syntax::CaseSelector::Default => {
                                // DO NOTHING
                            }
                            wesl_parse::syntax::CaseSelector::Expression(expr) => {
                                Self::mangle_expression(expr, path.clone());
                            }
                        }
                    }
                    for c in c.body.statements.iter_mut() {
                        Self::mangle_statement(c, path.clone());
                    }
                }
            }
            Statement::Loop(l) => {
                for c in l.body.statements.iter_mut() {
                    Self::mangle_statement(c, path.clone());
                }
                if let Some(cont) = l.continuing.as_mut() {
                    for c in cont.body.statements.iter_mut() {
                        Self::mangle_statement(c, path.clone());
                    }
                    if let Some(break_if) = cont.break_if.as_mut() {
                        Self::mangle_expression(break_if, path);
                    }
                }
            }
            Statement::For(f) => {
                for c in f.body.statements.iter_mut() {
                    Self::mangle_statement(c, path.clone());
                }
                if let Some(cond) = f.condition.as_mut() {
                    Self::mangle_expression(cond, path.clone());
                }
                if let Some(statement) = f.initializer.as_mut() {
                    Self::mangle_statement(statement.as_mut(), path.clone());
                }
                if let Some(update) = f.update.as_mut() {
                    Self::mangle_statement(update.as_mut(), path);
                }
            }
            Statement::While(w) => {
                for c in w.body.statements.iter_mut() {
                    Self::mangle_statement(c, path.clone());
                }
                Self::mangle_expression(&mut w.condition, path);
            }
            Statement::Break => {
                // DO NOTHING
            }
            Statement::Continue => {
                // DO NOTHING
            }
            Statement::Return(ret) => {
                if let Some(ret) = ret.as_mut() {
                    Self::mangle_expression(ret, path);
                }
            }
            Statement::Discard => {
                // DO NOTHING
            }
            Statement::FunctionCall(f) => {
                Self::mangle_path(&mut f.path);
                for arg in f.arguments.iter_mut() {
                    Self::mangle_expression(arg, path.clone());
                }
                if let Some(args) = f.template_args.as_mut() {
                    for arg in args {
                        Self::mangle_expression(arg, path.clone());
                    }
                }
            }
            Statement::ConstAssert(assrt) => {
                Self::mangle_expression(&mut assrt.expression, path);
            }
            Statement::Declaration(decl) => {
                if let Some(typ) = decl.declaration.typ.as_mut() {
                    Self::mangle_type(typ, path.clone());
                }

                if let Some(init) = decl.declaration.initializer.as_mut() {
                    Self::mangle_expression(init, path.clone());
                }
                for statement in decl.statements.iter_mut() {
                    Self::mangle_statement(statement, path.clone());
                }
            }
        }
    }

    fn mangle_expression(expr: &mut Expression, path: ModulePath) {
        match expr {
            Expression::Literal(_) => {
                // DO NOTHING
            }
            Expression::Parenthesized(p) => {
                Self::mangle_expression(p.as_mut(), path);
            }
            Expression::NamedComponent(n) => {
                Self::mangle_expression(&mut n.base, path);
            }
            Expression::Indexing(idx) => {
                Self::mangle_expression(&mut idx.base, path.clone());
                Self::mangle_expression(&mut idx.index, path);
            }
            Expression::Unary(u) => {
                Self::mangle_expression(&mut u.operand, path);
            }
            Expression::Binary(b) => {
                Self::mangle_expression(&mut b.left, path.clone());
                Self::mangle_expression(&mut b.right, path);
            }
            Expression::FunctionCall(f) => {
                Self::mangle_path(&mut f.path);
                for arg in f.arguments.iter_mut() {
                    Self::mangle_expression(arg, path.clone());
                }
                if let Some(args) = f.template_args.as_mut() {
                    for arg in args.iter_mut() {
                        Self::mangle_expression(arg, path.clone());
                    }
                }
            }
            Expression::Identifier(id) => {
                Self::mangle_path(&mut id.path);
            }
            Expression::Type(typ) => {
                Self::mangle_type(typ, path);
            }
        }
    }

    fn mangle_type(typ: &mut TypeExpression, path: ModulePath) {
        Self::mangle_path(&mut typ.path);
        if let Some(args) = typ.template_args.as_mut() {
            for arg in args.iter_mut() {
                Self::mangle_expression(arg, path.clone());
            }
        }
    }

    fn mangle_decl(decl: &mut Declaration, path: ModulePath) {
        if let Some(init) = decl.initializer.as_mut() {
            Self::mangle_expression(init, path.clone());
        }
        if let Some(args) = decl.template_args.as_mut() {
            for a in args.iter_mut() {
                Self::mangle_expression(a, path.clone());
            }
        }
        if let Some(typ) = decl.typ.as_mut() {
            Self::mangle_type(typ, path.clone());
        }
        Self::mangle_name(&mut decl.name, path);
    }

    fn mangle_alias(a: &mut Alias, path: ModulePath) {
        Self::mangle_name(&mut a.name, path.clone());
        Self::mangle_type(&mut a.typ, path);
    }

    fn mangle_struct(s: &mut Struct, path: ModulePath) {
        for member in s.members.iter_mut() {
            Self::mangle_type(&mut member.typ, path.clone());
        }
        Self::mangle_name(&mut s.name, path);
    }

    fn mangle_func(f: &mut Function, path: ModulePath) {
        Self::mangle_name(&mut f.name, path.clone());
        if let Some(ret) = f.return_type.as_mut() {
            Self::mangle_type(ret, path.clone());
        }
        for arg in f.parameters.iter_mut() {
            Self::mangle_type(&mut arg.typ, path.clone());
        }
        for statement in f.body.statements.iter_mut() {
            Self::mangle_statement(statement, path.clone());
        }
    }

    fn mangle_const_assert(a: &mut ConstAssert, path: ModulePath) {
        Self::mangle_expression(&mut a.expression, path);
    }

    fn mangle_module(m: &mut Module, mut path: ModulePath) {
        path.0.push_back(m.name.clone());
        for decl in m.members.iter_mut() {
            match decl {
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
                    Self::mangle_const_assert(assrt, path.clone());
                }
                ModuleMemberDeclaration::Module(module) => {
                    Self::mangle_module(module, path.clone());
                }
            }
        }
    }

    fn mangle_translation_unit(translation_unit: &mut TranslationUnit, path: ModulePath) {
        for decl in translation_unit.global_declarations.iter_mut() {
            match decl {
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
                    Self::mangle_const_assert(assrt, path.clone());
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
    ) -> Result<(), wesl_types::CompilerPassError> {
        self.mangle_mut(translation_unit);
        Ok(())
    }
}
