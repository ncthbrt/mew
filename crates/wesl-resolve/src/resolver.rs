use wesl_parse::syntax::{
    Alias, CompoundDirective, CompoundStatement, ConstAssert, Declaration, DeclarationStatement,
    Expression, ExtendDirective, Function, GlobalDeclaration, GlobalDirective, Module,
    ModuleDirective, ModuleMemberDeclaration, Statement, Struct, TranslationUnit, TypeExpression,
    Use,
};
use wesl_types::{CompilerPass, CompilerPassError};

#[derive(Debug, Default, Clone, Copy)]
pub struct Resolver;

#[derive(Debug, PartialEq, Clone, Hash)]
struct ModulePath(im::Vector<String>);

#[derive(Debug, PartialEq, Clone)]
enum ScopeMember {
    LocalDeclaration,
    ModuleMemberDeclaration(
        ModulePath,
        ModuleMemberDeclaration,
        im::HashMap<String, ScopeMember>,
    ),
    UseDeclaration(ModulePath, String),
    GlobalDeclaration(GlobalDeclaration, im::HashMap<String, ScopeMember>),
    FormalFunctionParameter,
}

impl Resolver {
    fn compound_statement_to_absolute_paths(
        statement: &mut CompoundStatement,
        module_path: ModulePath,
        mut scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        for CompoundDirective::Use(usage) in statement.directives.drain(0..) {
            Self::add_usage_to_scope(usage, &mut scope)?;
        }
        for c in statement.statements.iter_mut() {
            Self::statement_to_absolute_paths(c, module_path.clone(), scope.clone())?;
        }
        Ok(())
    }

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
                Self::compound_statement_to_absolute_paths(c, module_path, scope)?;
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
                Self::compound_statement_to_absolute_paths(
                    &mut iff.if_clause.1,
                    module_path.clone(),
                    scope.clone(),
                )?;
                for (else_if_expr, else_if_statements) in iff.else_if_clauses.iter_mut() {
                    Self::expression_to_absolute_paths(
                        else_if_expr,
                        module_path.clone(),
                        scope.clone(),
                    )?;
                    Self::compound_statement_to_absolute_paths(
                        else_if_statements,
                        module_path.clone(),
                        scope.clone(),
                    )?;
                }
                if let Some(else_clause) = iff.else_clause.as_mut() {
                    Self::compound_statement_to_absolute_paths(else_clause, module_path, scope)?;
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
                    Self::compound_statement_to_absolute_paths(
                        &mut clause.body,
                        module_path.clone(),
                        scope.clone(),
                    )?;
                }
            }
            Statement::Loop(l) => {
                for CompoundDirective::Use(usage) in l.body.directives.drain(0..) {
                    Self::add_usage_to_scope(usage, &mut scope)?;
                }
                Self::compound_statement_to_absolute_paths(
                    &mut l.body,
                    module_path.clone(),
                    scope.clone(),
                )?;
                // Unfortunate asymmetry (and redundant work) here as the continuing statement is within the same scope
                for c in l.body.statements.iter_mut() {
                    if let Statement::Declaration(decl) = c {
                        Self::add_all_local_declarations_recursively_to_scope_ONLY_FOR_loop_statement(
                            decl,
                            module_path.clone(),
                            &mut scope,
                        )?;
                    }
                }
                if let Some(cont) = l.continuing.as_mut() {
                    // Unfortunate asymmetry (and redundant work) AGAIN as the break_if expr is in the same scope
                    for CompoundDirective::Use(usage) in cont.body.directives.drain(0..) {
                        Self::add_usage_to_scope(usage, &mut scope)?;
                    }
                    Self::compound_statement_to_absolute_paths(
                        &mut l.body,
                        module_path.clone(),
                        scope.clone(),
                    )?;
                    for c in cont.body.statements.iter_mut() {
                        if let Statement::Declaration(decl) = c {
                            Self::add_all_local_declarations_recursively_to_scope_ONLY_FOR_loop_statement(decl, module_path.clone(), &mut scope)?;
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
                Self::compound_statement_to_absolute_paths(&mut f.body, module_path, scope)?;
            }
            Statement::While(w) => {
                Self::expression_to_absolute_paths(
                    &mut w.condition,
                    module_path.clone(),
                    scope.clone(),
                )?;
                Self::compound_statement_to_absolute_paths(&mut w.body, module_path, scope)?;
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
        if !module.name.is_empty() {
            module_path.0.push_back(module.name.clone());
        }
        Self::update_module_scope_and_apply_extensions(&mut module_path, module, &mut scope)?;

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
                ScopeMember::ModuleMemberDeclaration(module_path, _parent, _scope) => {
                    let mut new_path = module_path
                        .0
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    new_path.extend(path.iter().cloned());
                    *path = new_path;
                }
                ScopeMember::GlobalDeclaration(_tum, _scope) => {
                    // No action required
                }
                ScopeMember::FormalFunctionParameter => {
                    // No action required
                }
                ScopeMember::UseDeclaration(module_path, underlying_name) => {
                    let mut new_path = module_path
                        .0
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    new_path.extend(path.iter().cloned());
                    *new_path.last_mut().unwrap() = underlying_name.to_string();
                    *path = new_path;
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

        Self::compound_statement_to_absolute_paths(&mut func.body, module_path, scope)?;
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

    fn add_usage_to_scope(
        mut usage: Use,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        Self::relative_path_to_absolute_path(scope.clone(), &mut usage.path)?;
        match usage.content {
            wesl_parse::syntax::UseContent::Item(item) => {
                if let Some(rename) = item.rename.as_ref() {
                    scope.insert(
                        rename.clone(),
                        ScopeMember::UseDeclaration(
                            ModulePath(im::Vector::from(usage.path.clone())),
                            item.name.clone(),
                        ),
                    );
                } else {
                    scope.insert(
                        item.name.clone(),
                        ScopeMember::UseDeclaration(
                            ModulePath(im::Vector::from(usage.path.clone())),
                            item.name.clone(),
                        ),
                    );
                }
            }
            wesl_parse::syntax::UseContent::Collection(c) => {
                for mut c in c {
                    c.path.extend(usage.path.iter().cloned());
                    Self::add_usage_to_scope(c, scope)?;
                }
            }
        }
        Ok(())
    }

    #[allow(non_snake_case)]
    fn add_all_local_declarations_recursively_to_scope_ONLY_FOR_loop_statement(
        decl: &DeclarationStatement,
        module_path: ModulePath,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        scope.insert(decl.declaration.name.clone(), ScopeMember::LocalDeclaration);
        for s in decl.statements.iter() {
            if let Statement::Declaration(s) = s {
                Self::add_all_local_declarations_recursively_to_scope_ONLY_FOR_loop_statement(
                    s,
                    module_path.clone(),
                    scope,
                )?;
            }
        }
        Ok(())
    }

    fn find_module_and_scope(
        mut scope: im::HashMap<String, ScopeMember>,
        path: &[String],
    ) -> Result<(Module, im::HashMap<String, ScopeMember>), CompilerPassError> {
        assert!(!path.is_empty());
        let mut module_path = ModulePath(im::Vector::new());
        let mut remaining_path: im::Vector<String> = path.into();
        let fst = remaining_path.pop_front().unwrap();
        if let Some(scope_member) = scope.remove(&fst) {
            let (m, mut scope) = match scope_member {
                ScopeMember::ModuleMemberDeclaration(
                    _,
                    ModuleMemberDeclaration::Module(m),
                    scope,
                ) => (m, scope),
                ScopeMember::GlobalDeclaration(GlobalDeclaration::Module(m), scope) => (m, scope),
                _ => {
                    panic!(
                        "INVARIANT FAILURE: UNEXPECTED SCOPE MEMBER IN THIS STAGE OF PROCESSING"
                    );
                }
            };
            let mut module = m;
            'outer: while !remaining_path.is_empty() {
                Self::update_module_scope_and_apply_extensions(
                    &mut module_path,
                    &mut module,
                    &mut scope,
                )?;
                for decl in module.members.iter_mut() {
                    if let ModuleMemberDeclaration::Module(m) = decl {
                        if &m.name == remaining_path.head().unwrap() {
                            let _ = remaining_path.pop_front().unwrap();
                            module = m.clone();
                            continue 'outer;
                        }
                    }
                }
                return Err(CompilerPassError::SymbolNotFound(path.into()));
            }
            Ok((module.clone(), scope))
        } else {
            Err(CompilerPassError::SymbolNotFound(path.into()))
        }
    }

    fn update_module_scope_and_apply_extensions(
        module_path: &mut ModulePath,
        module: &mut Module,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        let mut other_dirs = vec![];
        let mut extend_dirs = vec![];
        let parent_scope = scope.clone();
        for decl in module.members.iter() {
            if let Some(name) = decl.name() {
                scope.insert(
                    name,
                    ScopeMember::ModuleMemberDeclaration(
                        module_path.clone(),
                        decl.clone(),
                        parent_scope.clone(),
                    ),
                );
            }
        }

        for dir in module.directives.drain(0..) {
            match dir {
                ModuleDirective::Use(usage) => {
                    Self::add_usage_to_scope(usage, scope)?;
                }
                ModuleDirective::Extend(extend) => {
                    extend_dirs.push(extend);
                }
            }
        }
        module.directives.append(&mut other_dirs);

        for extension in extend_dirs {
            Self::extend_module(module, &extension, module_path.clone(), scope)?;
        }

        Ok(())
    }

    fn extend_translation_unit(
        translation_unit: &mut TranslationUnit,
        extend: &ExtendDirective,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        let (mut module, module_scope) = Self::find_module_and_scope(scope.clone(), &extend.path)?;
        let parent_scope = module_scope.clone();

        // OK. So we need to add the symbols to the translation unit
        // Clear the module name because we don't want to affect the module path
        module.name.clear();
        Self::module_to_absolute_path(&mut module, ModulePath(im::Vector::new()), module_scope)?;
        for member in module.members.drain(0..) {
            if let Some(name) = member.name() {
                scope.insert(
                    name,
                    ScopeMember::GlobalDeclaration(member.clone().into(), parent_scope.clone()),
                );
            }
            translation_unit.global_declarations.push(member.into());
        }
        for directive in module.directives.drain(0..) {
            match directive {
                ModuleDirective::Use(_) => {
                    // DO NOTHING
                }
                ModuleDirective::Extend(_) => {
                    // DO NOTHING
                } // other => {
                  //     module.directives.push(other);
                  // }
            }
        }
        Ok(())
    }

    fn extend_module(
        module: &mut Module,
        extend: &ExtendDirective,
        module_path: ModulePath,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        let mut extension_path = extend.path.clone();
        Self::relative_path_to_absolute_path(scope.clone(), &mut extension_path)?;

        let (mut other_module, other_module_scope) =
            Self::find_module_and_scope(scope.clone(), &extension_path)?;

        let parent_other_module_scope = other_module_scope.clone();
        Self::module_to_absolute_path(
            &mut other_module,
            module_path.clone(),
            other_module_scope.clone(),
        )?;

        for member in other_module.members.drain(0..) {
            if let Some(name) = member.name() {
                scope.insert(
                    name,
                    ScopeMember::ModuleMemberDeclaration(
                        module_path.clone(),
                        member.clone(),
                        parent_other_module_scope.clone(),
                    ),
                );
            }
            module.members.push(member);
        }

        for directive in other_module.directives.drain(0..) {
            match directive {
                ModuleDirective::Use(_) => {
                    // DO NOTHING
                }
                ModuleDirective::Extend(_) => {
                    // DO NOTHING
                } // other => {
                  //     module.directives.push(other);
                  // }
            }
        }

        Ok(())
    }

    fn translation_unit_to_absolute_path(
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let module_path = ModulePath(im::Vector::new());
        let mut scope = im::HashMap::new();
        let mut other_directives = vec![];
        let mut extend_directives = vec![];
        for dir in translation_unit.global_directives.drain(0..) {
            match dir {
                GlobalDirective::Use(usage) => {
                    Self::add_usage_to_scope(usage, &mut scope)?;
                }
                GlobalDirective::Extend(mut extend) => {
                    Self::relative_path_to_absolute_path(scope.clone(), &mut extend.path)?;
                    extend_directives.push(extend);
                }
                other => other_directives.push(other),
            }
        }

        translation_unit
            .global_directives
            .append(&mut other_directives);

        for decl in translation_unit.global_declarations.iter() {
            if let Some(name) = decl.name() {
                scope.insert(
                    name,
                    ScopeMember::GlobalDeclaration(decl.clone(), scope.clone()),
                );
            }
        }

        for extend in extend_directives.iter() {
            Self::extend_translation_unit(translation_unit, extend, &mut scope)?;
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
