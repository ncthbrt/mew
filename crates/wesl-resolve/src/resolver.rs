use wesl_parse::syntax::{
    Alias, ConstAssert, Declaration, Expression, Function, GlobalDeclaration, Module,
    ModuleMemberDeclaration, Statement, Struct, TranslationUnit, TypeExpression,
};
use wesl_types::{CompilerPass, CompilerPassError};

#[derive(Debug, Default, Clone, Copy)]
pub struct Resolver;

#[derive(Debug, PartialEq, Clone)]
struct ModulePath(im::Vector<String>);

#[derive(Debug, PartialEq, Clone)]
enum ScopeMember {
    LocalDeclaration,
    ModuleMemberDeclaration(ModulePath),
    GlobalDeclaration,
    FormalFunctionParameter,
}

impl Resolver {
    fn statement_to_absolute_paths(
        statement: &mut Statement,
        module_path: ModulePath,
        mut scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        match statement {
            Statement::Void => {
                // No action required
            }
            Statement::Compound(c) => {
                for c in c.statements.iter_mut() {
                    Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                }
            }
            Statement::Assignment(a) => {
                Self::expression_to_absolute_paths(&mut a.lhs, module_path.clone(), scope.clone())?;
                Self::expression_to_absolute_paths(&mut a.rhs, module_path.clone(), scope.clone())?;
            }
            Statement::Increment(i) => {
                Self::expression_to_absolute_paths(i, module_path.clone(), scope.clone())?;
            }
            Statement::Decrement(d) => {
                Self::expression_to_absolute_paths(d, module_path.clone(), scope.clone())?;
            }
            Statement::If(iff) => {
                Self::expression_to_absolute_paths(
                    &mut iff.if_clause.0,
                    module_path.clone(),
                    scope.clone(),
                )?;
                for c in iff.if_clause.1.statements.iter_mut() {
                    Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                }
                for (else_if_expr, else_if_statements) in iff.else_if_clauses.iter_mut() {
                    Self::expression_to_absolute_paths(
                        else_if_expr,
                        module_path.clone(),
                        scope.clone(),
                    )?;
                    for c in else_if_statements.statements.iter_mut() {
                        Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                    }
                }
                if let Some(else_clause) = iff.else_clause.as_mut() {
                    for c in else_clause.statements.iter_mut() {
                        Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                    }
                }
            }
            Statement::Switch(s) => {
                Self::expression_to_absolute_paths(
                    &mut s.expression,
                    module_path.clone(),
                    scope.clone(),
                )?;
                for clause in s.clauses.iter_mut() {
                    for c in clause.case_selectors.iter_mut() {
                        match c {
                            wesl_parse::syntax::CaseSelector::Default => {
                                // NO ACTION NEEDED
                            }
                            wesl_parse::syntax::CaseSelector::Expression(e) => {
                                Self::expression_to_absolute_paths(
                                    e,
                                    module_path.clone(),
                                    scope.clone(),
                                )?;
                            }
                        }
                    }
                    for c in clause.body.statements.iter_mut() {
                        Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                    }
                }
            }
            Statement::Loop(l) => {
                for c in l.body.statements.iter_mut() {
                    Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                    if let Statement::Declaration(decl) = c {
                        scope.insert(decl.declaration.name.clone(), ScopeMember::LocalDeclaration);
                    }
                }
                if let Some(cont) = l.continuing.as_mut() {
                    for c in cont.body.statements.iter_mut() {
                        Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                        if let Statement::Declaration(decl) = c {
                            scope.insert(
                                decl.declaration.name.clone(),
                                ScopeMember::LocalDeclaration,
                            );
                        }
                    }
                    if let Some(expr) = cont.break_if.as_mut() {
                        Self::expression_to_absolute_paths(expr, module_path, scope)?;
                    }
                }
            }
            Statement::For(f) => {
                if let Some(init) = f.initializer.as_mut() {
                    Self::statement_to_absolute_paths(
                        init.as_mut(),
                        module_path.clone(),
                        scope.clone(),
                    )?;
                    if let Statement::Declaration(d) = init.as_mut() {
                        scope.insert(d.declaration.name.clone(), ScopeMember::LocalDeclaration);
                    };
                }
                if let Some(cond) = f.condition.as_mut() {
                    Self::expression_to_absolute_paths(cond, module_path.clone(), scope.clone())?;
                }
                if let Some(update) = f.update.as_mut() {
                    Self::statement_to_absolute_paths(
                        update.as_mut(),
                        module_path.clone(),
                        scope.clone(),
                    )?;
                }
                for c in f.body.statements.iter_mut() {
                    Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                }
            }
            Statement::While(w) => {
                Self::expression_to_absolute_paths(
                    &mut w.condition,
                    module_path.clone(),
                    scope.clone(),
                )?;
                for c in w.body.statements.iter_mut() {
                    Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
                }
            }
            Statement::Break => {
                // No action required
            }
            Statement::Continue => {
                // No action required
            }
            Statement::Return(r) => {
                if let Some(r) = r.as_mut() {
                    Self::expression_to_absolute_paths(r, module_path, scope)?;
                }
            }
            Statement::Discard => {
                // No action required
            }
            Statement::FunctionCall(f) => {
                Self::relative_path_to_absolute_path(scope.clone(), &mut f.path)?;
                for a in f.arguments.iter_mut() {
                    Self::expression_to_absolute_paths(a, module_path.clone(), scope.clone())?;
                }
                if let Some(args) = f.template_args.as_mut() {
                    for a in args.iter_mut() {
                        Self::expression_to_absolute_paths(a, module_path.clone(), scope.clone())?;
                    }
                }
            }
            Statement::ConstAssert(a) => {
                Self::expression_to_absolute_paths(
                    &mut a.expression,
                    module_path.clone(),
                    scope.clone(),
                )?;
            }
            Statement::Declaration(d) => {
                if let Some(init) = d.declaration.initializer.as_mut() {
                    Self::expression_to_absolute_paths(init, module_path.clone(), scope.clone())?;
                }
                if let Some(typ) = d.declaration.typ.as_mut() {
                    Self::type_to_absolute_path(typ, module_path.clone(), scope.clone())?;
                };
                let name = d.declaration.name.clone();
                scope.insert(name, ScopeMember::LocalDeclaration);
                for s in d.statements.iter_mut() {
                    Self::statement_to_absolute_paths(s, module_path.clone(), scope.clone())?;
                }
            }
        };
        Ok(())
    }

    fn expression_to_absolute_paths(
        expression: &mut Expression,
        module_path: ModulePath,
        scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        match expression {
            Expression::Literal(_) => {}
            Expression::Parenthesized(p) => {
                Self::expression_to_absolute_paths(p.as_mut(), module_path, scope)?
            }
            Expression::NamedComponent(n) => {
                Self::expression_to_absolute_paths(&mut n.base, module_path, scope)?
            }
            Expression::Indexing(idx) => {
                Self::expression_to_absolute_paths(&mut idx.base, module_path, scope)?
            }
            Expression::Unary(u) => {
                Self::expression_to_absolute_paths(&mut u.operand, module_path, scope)?
            }
            Expression::Binary(b) => {
                Self::expression_to_absolute_paths(
                    &mut b.left,
                    module_path.clone(),
                    scope.clone(),
                )?;
                Self::expression_to_absolute_paths(&mut b.right, module_path, scope)?;
            }
            Expression::FunctionCall(f) => {
                Self::relative_path_to_absolute_path(scope.clone(), &mut f.path)?;
                for arg in f.arguments.iter_mut() {
                    Self::expression_to_absolute_paths(arg, module_path.clone(), scope.clone())?;
                }
                if let Some(args) = f.template_args.as_mut() {
                    for a in args.iter_mut() {
                        Self::expression_to_absolute_paths(a, module_path.clone(), scope.clone())?;
                    }
                }
            }
            Expression::Identifier(ident) => {
                Self::relative_path_to_absolute_path(scope, &mut ident.path)?;
            }
            Expression::Type(typ) => {
                Self::type_to_absolute_path(typ, module_path.clone(), scope)?;
            }
        };
        Ok(())
    }

    fn module_to_absolute_path(
        module: &mut Module,
        mut module_path: ModulePath,
        mut scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        module_path.0.push_back(module.name.clone());

        for decl in module.members.iter() {
            if let Some(name) = decl.name() {
                scope.insert(
                    name,
                    ScopeMember::ModuleMemberDeclaration(module_path.clone()),
                );
            }
        }

        for decl in module.members.iter_mut() {
            match decl {
                ModuleMemberDeclaration::Void => {
                    // NO ACTION REQUIRED REQUIRED
                }
                ModuleMemberDeclaration::Declaration(decl) => {
                    Self::decl_to_absolute_path(decl, module_path.clone(), scope.clone())?;
                }
                ModuleMemberDeclaration::Alias(a) => {
                    Self::alias_to_absolute_path(a, module_path.clone(), scope.clone())?;
                }
                ModuleMemberDeclaration::Struct(s) => {
                    Self::struct_to_absolute_path(s, module_path.clone(), scope.clone())?;
                }
                ModuleMemberDeclaration::Function(f) => {
                    Self::func_to_absolute_path(f, module_path.clone(), scope.clone())?;
                }
                ModuleMemberDeclaration::ConstAssert(assrt) => {
                    Self::const_assert_to_absolute_path(assrt, module_path.clone(), scope.clone())?;
                }
                ModuleMemberDeclaration::Module(m) => {
                    Self::module_to_absolute_path(m, module_path.clone(), scope.clone())?;
                }
            }
        }
        Ok(())
    }

    fn relative_path_to_absolute_path(
        mut scope: im::HashMap<String, ScopeMember>,
        path: &mut Vec<String>,
    ) -> Result<(), CompilerPassError> {
        if let Some(symbol) = scope.remove(path.first().unwrap().as_str()) {
            match symbol {
                ScopeMember::LocalDeclaration => {
                    // No action required
                }
                ScopeMember::ModuleMemberDeclaration(module_path) => {
                    let mut new_path = module_path
                        .0
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    new_path.extend(path.iter().cloned());
                    *path = new_path;
                }
                ScopeMember::GlobalDeclaration => {
                    // No action required
                }
                ScopeMember::FormalFunctionParameter => {
                    // No action required
                }
            }
        } else {
            // TODO: Have to return Ok unless we can enumerate all the built in symbols.
            // That should in theory be possible as they're defined by the spec
            // return Err(CompilerPassError::SymbolNotFound(path.clone().to_owned()));
            return Ok(());
        };

        Ok(())
    }

    fn type_to_absolute_path(
        typ: &mut TypeExpression,
        module_path: ModulePath,
        scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        Self::relative_path_to_absolute_path(scope.clone(), &mut typ.path)?;
        if let Some(args) = typ.template_args.as_mut() {
            for arg in args {
                Self::expression_to_absolute_paths(arg, module_path.clone(), scope.clone())?;
            }
        }
        Ok(())
    }

    fn struct_to_absolute_path(
        strct: &mut Struct,
        module_path: ModulePath,
        scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        for m in strct.members.iter_mut() {
            Self::type_to_absolute_path(&mut m.typ, module_path.clone(), scope.clone())?;
        }
        Ok(())
    }

    fn decl_to_absolute_path(
        declaration: &mut Declaration,
        module_path: ModulePath,
        scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        if let Some(init) = declaration.initializer.as_mut() {
            Self::expression_to_absolute_paths(init, module_path.clone(), scope.clone())?;
        };
        if let Some(typ) = declaration.typ.as_mut() {
            Self::type_to_absolute_path(typ, module_path.clone(), scope.clone())?;
        };
        Ok(())
    }

    fn func_to_absolute_path(
        func: &mut Function,
        module_path: ModulePath,
        mut scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        if let Some(r) = func.return_type.as_mut() {
            Self::relative_path_to_absolute_path(scope.clone(), &mut r.path)?;
            if let Some(args) = r.template_args.as_mut() {
                for arg in args.iter_mut() {
                    Self::expression_to_absolute_paths(arg, module_path.clone(), scope.clone())?;
                }
            }
        }

        for p in func.parameters.iter_mut() {
            Self::type_to_absolute_path(&mut p.typ, module_path.clone(), scope.clone())?;
            scope.insert(p.name.clone(), ScopeMember::FormalFunctionParameter);
        }

        for b in func.body.statements.iter_mut() {
            Self::statement_to_absolute_paths(b, module_path.clone(), scope.clone())?;
        }
        Ok(())
    }

    fn const_assert_to_absolute_path(
        assrt: &mut ConstAssert,
        module_path: ModulePath,
        scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        Self::expression_to_absolute_paths(&mut assrt.expression, module_path, scope)?;
        Ok(())
    }

    fn alias_to_absolute_path(
        alias: &mut Alias,
        module_path: ModulePath,
        scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        Self::type_to_absolute_path(&mut alias.typ, module_path, scope)?;
        Ok(())
    }

    fn translation_unit_to_absolute_path(
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let module_path = ModulePath(im::Vector::new());
        let mut scope = im::HashMap::new();
        for decl in translation_unit.global_declarations.iter() {
            if let Some(name) = decl.name() {
                scope.insert(name, ScopeMember::GlobalDeclaration);
            }
        }
        for decl in translation_unit.global_declarations.iter_mut() {
            match decl {
                GlobalDeclaration::Void => {
                    // NO ACTION REQUIRED
                }
                GlobalDeclaration::Declaration(decl) => {
                    Self::decl_to_absolute_path(decl, module_path.clone(), scope.clone())?;
                }
                GlobalDeclaration::Alias(a) => {
                    Self::alias_to_absolute_path(a, module_path.clone(), scope.clone())?;
                }
                GlobalDeclaration::Struct(s) => {
                    Self::struct_to_absolute_path(s, module_path.clone(), scope.clone())?;
                }
                GlobalDeclaration::Function(f) => {
                    Self::func_to_absolute_path(f, module_path.clone(), scope.clone())?;
                }
                GlobalDeclaration::ConstAssert(assrt) => {
                    Self::const_assert_to_absolute_path(assrt, module_path.clone(), scope.clone())?;
                }
                GlobalDeclaration::Module(m) => {
                    Self::module_to_absolute_path(m, module_path.clone(), scope.clone())?;
                }
            }
        }

        Ok(())
    }

    pub fn resolve_mut(
        &self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        Self::translation_unit_to_absolute_path(translation_unit)?;
        Ok(())
    }
}

impl CompilerPass for Resolver {
    fn apply_mut(
        &mut self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        self.resolve_mut(translation_unit)
    }
}
