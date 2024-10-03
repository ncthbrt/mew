use std::fmt::Debug;

use wesl_parse::{
    span::Spanned,
    syntax::{
        Alias, CompoundDirective, CompoundStatement, ConstAssert, Declaration,
        DeclarationStatement, Expression, ExtendDirective, Function, GlobalDeclaration,
        GlobalDirective, IdentifierExpression, Module, ModuleDirective, ModuleMemberDeclaration,
        PathPart, Statement, Struct, TemplateArg, TranslationUnit, TypeExpression, Use,
    },
};
use wesl_types::{
    builtins::{get_builtin_functions, get_builtin_tokens},
    CompilerPass, CompilerPassError,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Resolver;

#[derive(Debug, PartialEq, Clone, Hash)]
struct ModulePath(im::Vector<PathPart>);

#[derive(Debug, PartialEq, Clone)]
enum ScopeMember {
    LocalDeclaration,
    BuiltIn,
    ModuleMemberDeclaration(ModulePath, ModuleMemberDeclaration),
    UseDeclaration(ModulePath, String, Option<Vec<Spanned<TemplateArg>>>),
    GlobalDeclaration(GlobalDeclaration),
    FormalFunctionParameter,
    TemplateParam(String),
}

impl Resolver {
    fn compound_statement_to_absolute_paths(
        statement: &mut CompoundStatement,
        module_path: ModulePath,
        mut scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        for CompoundDirective::Use(usage) in statement.directives.drain(..).map(|x| x.value) {
            Self::add_usage_to_scope(usage, module_path.clone(), &mut scope)?;
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
                        match &mut c.value {
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
                for usage in l.body.directives.drain(..) {
                    let CompoundDirective::Use(usage) = usage.value;
                    Self::add_usage_to_scope(usage, module_path.clone(), &mut scope)?;
                }
                Self::compound_statement_to_absolute_paths(
                    &mut l.body,
                    module_path.clone(),
                    scope.clone(),
                )?;
                // Unfortunate asymmetry (and redundant work) here as the continuing statement is within the same scope
                for c in l.body.statements.iter_mut() {
                    if let Statement::Declaration(decl) = c.as_mut() {
                        Self::add_all_local_declarations_recursively_to_scope_ONLY_FOR_loop_statement(
                            decl,
                            module_path.clone(),
                            &mut scope,
                        )?;
                    }
                }
                if let Some(cont) = l.continuing.as_mut() {
                    // Unfortunate asymmetry (and redundant work) AGAIN as the break_if expr is in the same scope
                    for usage in cont.body.directives.drain(..) {
                        let CompoundDirective::Use(usage) = usage.value;
                        Self::add_usage_to_scope(usage, module_path.clone(), &mut scope)?;
                    }
                    Self::compound_statement_to_absolute_paths(
                        &mut l.body,
                        module_path.clone(),
                        scope.clone(),
                    )?;
                    for c in cont.body.statements.iter_mut() {
                        if let Statement::Declaration(decl) = c.as_ref() {
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
                    if let Statement::Declaration(d) = init.as_mut().as_mut() {
                        scope.insert(
                            d.declaration.name.value.clone(),
                            ScopeMember::LocalDeclaration,
                        );
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
                Self::relative_path_to_absolute_path(
                    scope.clone(),
                    module_path.clone(),
                    &mut f.path,
                )?;
                for a in f.arguments.iter_mut() {
                    Self::expression_to_absolute_paths(a, module_path.clone(), scope.clone())?;
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
                let name = d.declaration.name.value.clone();
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
                Self::relative_path_to_absolute_path(
                    scope.clone(),
                    module_path.clone(),
                    &mut f.path,
                )?;
                for arg in f.arguments.iter_mut() {
                    Self::expression_to_absolute_paths(arg, module_path.clone(), scope.clone())?;
                }
            }
            Expression::Identifier(ident) => {
                Self::relative_path_to_absolute_path(scope, module_path.clone(), &mut ident.path)?;
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
        Self::update_module_scope(&mut module_path, module, &mut scope)?;
        Self::add_extensions_and_usages_to_scope(&mut module_path, module, &mut scope)?;

        for decl in module.members.iter_mut() {
            match decl.as_mut() {
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
        module_path: ModulePath,
        path: &mut Spanned<Vec<PathPart>>,
    ) -> Result<(), CompilerPassError> {
        for p in path.iter_mut() {
            if let Some(template_args) = p.template_args.as_mut() {
                for arg in template_args.iter_mut() {
                    Self::expression_to_absolute_paths(
                        &mut arg.value.expression,
                        module_path.clone(),
                        scope.clone(),
                    )?;
                }
            }
        }
        if let Some(symbol) = scope.remove(path.first().as_ref().unwrap().name.as_str()) {
            match symbol {
                ScopeMember::LocalDeclaration => {
                    // No action required
                }
                ScopeMember::ModuleMemberDeclaration(module_path, _parent) => {
                    let mut new_path = module_path.0.iter().cloned().collect::<Vec<PathPart>>();
                    new_path.extend(path.iter().cloned());
                    path.value = new_path;
                }
                ScopeMember::GlobalDeclaration(_) => {
                    // No action required
                }
                ScopeMember::FormalFunctionParameter => {
                    // No action required
                }
                ScopeMember::UseDeclaration(module_path, underlying_name, template_args) => {
                    let mut new_path = module_path.0.iter().cloned().collect::<Vec<PathPart>>();
                    *path.first_mut().unwrap().name = underlying_name.to_string();
                    if template_args.is_some() {
                        path.first_mut().unwrap().template_args = template_args;
                    }
                    new_path.extend(path.iter().cloned());
                    path.value = new_path;
                }
                ScopeMember::BuiltIn => {
                    // No action required
                }
                ScopeMember::TemplateParam(new_name) => {
                    let fst = path.value.first_mut().unwrap();
                    fst.name.value = new_name;
                    // No action required
                }
            }
        } else {
            return Err(CompilerPassError::SymbolNotFound(
                path.value.clone().to_owned(),
                path.span(),
            ));
        };

        Ok(())
    }

    fn type_to_absolute_path(
        typ: &mut TypeExpression,
        module_path: ModulePath,
        scope: im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        Self::relative_path_to_absolute_path(scope.clone(), module_path, &mut typ.path)?;
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
        Self::function_template_parameters_to_absolute_path(module_path.clone(), func, &mut scope)?;
        if let Some(r) = func.return_type.as_mut() {
            Self::relative_path_to_absolute_path(scope.clone(), module_path.clone(), &mut r.path)?;
        }

        for p in func.parameters.iter_mut() {
            Self::type_to_absolute_path(&mut p.typ, module_path.clone(), scope.clone())?;
            scope.insert(p.name.value.clone(), ScopeMember::FormalFunctionParameter);
        }

        Self::compound_statement_to_absolute_paths(&mut func.body, module_path, scope)?;

        Ok(())
    }

    fn mangle_template_parameter_name(
        module_path: &ModulePath,
        containing_name: &str,
        old_name: &str,
    ) -> String {
        let mut name: String = String::new();
        let path = module_path
            .0
            .iter()
            .map(|x| x.name.as_str())
            .chain([containing_name])
            .map(|x: &str| x.replace('_', "__"))
            .collect::<Vec<String>>()
            .join("_");

        name.push_str(&path);
        name.push('_');
        name.push_str(&old_name.replace('_', "__"));
        name
    }

    fn module_template_parameters_to_absolute_path(
        module_path: &mut ModulePath,
        module: &mut Module,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        let mut template_args = vec![];
        for param in module.template_parameters.iter_mut() {
            let old_name = param.name.value.clone();
            let new_name =
                Self::mangle_template_parameter_name(module_path, &module.name, &old_name);
            if let Some(default_value) = param.default_value.as_mut() {
                Self::expression_to_absolute_paths(
                    default_value.as_mut(),
                    module_path.clone(),
                    scope.clone(),
                )?;
            }
            param.name.value.clone_from(&new_name);
            template_args.push(Spanned::new(
                TemplateArg {
                    expression: Spanned::new(
                        Expression::Identifier(IdentifierExpression {
                            path: Spanned::new(
                                vec![PathPart {
                                    name: Spanned::new(new_name.clone(), param.name.span()),
                                    template_args: None,
                                }],
                                param.span(),
                            ),
                        }),
                        param.span(),
                    ),
                    arg_name: if param.default_value.is_some() {
                        Some(param.name.clone())
                    } else {
                        None
                    },
                },
                param.span(),
            ));
            scope.insert(
                new_name.clone(),
                ScopeMember::TemplateParam(new_name.clone()),
            );
            scope.insert(old_name, ScopeMember::TemplateParam(new_name.clone()));
        }

        if !module.name.is_empty() {
            module_path.0.push_back(PathPart {
                name: module.name.clone(),
                template_args: if template_args.is_empty() {
                    None
                } else {
                    Some(template_args)
                },
            });
        }

        Ok(())
    }

    fn function_template_parameters_to_absolute_path(
        module_path: ModulePath,
        function: &mut Function,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        for param in function.template_parameters.iter_mut() {
            if let Some(default_value) = param.default_value.as_mut() {
                Self::expression_to_absolute_paths(
                    default_value.as_mut(),
                    module_path.clone(),
                    scope.clone(),
                )?;
            }
            let old_name = param.name.value.clone();
            let new_name =
                Self::mangle_template_parameter_name(&module_path, &function.name, &param.name);
            param.name.value.clone_from(&new_name);
            scope.insert(
                new_name.clone(),
                ScopeMember::TemplateParam(new_name.clone()),
            );
            scope.insert(old_name.clone(), ScopeMember::TemplateParam(new_name));
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

    fn add_usage_to_scope(
        mut usage: Use,
        module_path: ModulePath,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        Self::relative_path_to_absolute_path(scope.clone(), module_path.clone(), &mut usage.path)?;
        match usage.content.into_inner() {
            wesl_parse::syntax::UseContent::Item(item) => {
                if let Some(rename) = item.rename.as_ref() {
                    scope.insert(
                        rename.value.clone(),
                        ScopeMember::UseDeclaration(
                            ModulePath(im::Vector::from(usage.path.value.clone())),
                            item.name.value.clone(),
                            item.template_args,
                        ),
                    );
                } else {
                    scope.insert(
                        item.name.value.clone(),
                        ScopeMember::UseDeclaration(
                            ModulePath(im::Vector::from(usage.path.value.clone())),
                            item.name.value.clone(),
                            item.template_args,
                        ),
                    );
                }
            }
            wesl_parse::syntax::UseContent::Collection(c) => {
                for mut c in c.into_iter() {
                    c.value.path.value.extend(usage.path.iter().cloned());
                    Self::add_usage_to_scope(c.value.clone(), module_path.clone(), scope)?;
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
        scope.insert(
            decl.declaration.name.value.clone(),
            ScopeMember::LocalDeclaration,
        );
        for s in decl.statements.iter() {
            if let Statement::Declaration(s) = s.as_ref() {
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
        path: &Spanned<Vec<PathPart>>,
    ) -> Result<(Module, im::HashMap<String, ScopeMember>), CompilerPassError> {
        assert!(!path.is_empty());
        let mut module_path = ModulePath(im::Vector::new());
        let mut remaining_path: im::Vector<PathPart> = path.value.clone().into();
        let fst: PathPart = remaining_path.pop_front().unwrap();
        if let Some(scope_member) = scope.get(fst.name.as_ref()).cloned() {
            let m = match scope_member {
                ScopeMember::ModuleMemberDeclaration(_, ModuleMemberDeclaration::Module(m)) => m,
                ScopeMember::GlobalDeclaration(GlobalDeclaration::Module(m)) => m,
                _ => {
                    panic!(
                        "INVARIANT FAILURE: UNEXPECTED SCOPE MEMBER IN THIS STAGE OF PROCESSING"
                    );
                }
            };
            let mut module = m;
            'outer: while !remaining_path.is_empty() {
                Self::update_module_scope(&mut module_path, &mut module, &mut scope)?;
                Self::add_extensions_and_usages_to_scope(
                    &mut module_path,
                    &mut module,
                    &mut scope,
                )?;
                for decl in module.members.iter_mut() {
                    if let ModuleMemberDeclaration::Module(m) = decl.as_mut() {
                        if m.name == remaining_path.head().as_ref().unwrap().name {
                            let _ = remaining_path.pop_front().unwrap();
                            module = m.clone();
                            continue 'outer;
                        }
                    }
                }
                return Err(CompilerPassError::SymbolNotFound(
                    path.value.clone(),
                    path.span(),
                ));
            }
            Ok((module.clone(), scope))
        } else {
            Err(CompilerPassError::SymbolNotFound(
                path.value.clone(),
                path.span(),
            ))
        }
    }

    fn update_module_scope(
        module_path: &mut ModulePath,
        module: &mut Module,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        Self::module_template_parameters_to_absolute_path(module_path, module, scope)?;
        for decl in module.members.iter() {
            if let Some(name) = decl.name() {
                scope.insert(
                    name.value,
                    ScopeMember::ModuleMemberDeclaration(module_path.clone(), decl.value.clone()),
                );
            }
        }
        Ok(())
    }

    fn add_extensions_and_usages_to_scope(
        module_path: &mut ModulePath,
        module: &mut Module,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<(), CompilerPassError> {
        let mut other_dirs: Vec<Spanned<ModuleDirective>> = vec![];
        let mut extend_dirs = vec![];

        for dir in module.directives.drain(..) {
            let span = dir.span();
            match dir.into_inner() {
                ModuleDirective::Use(usage) => {
                    Self::add_usage_to_scope(usage, module_path.clone(), scope)?;
                }
                ModuleDirective::Extend(extend) => {
                    extend_dirs.push(Spanned::new(extend, span));
                } // other => {
                  // other_dirs.push(Spanned::new(ModuleDirective::Extend(extend), span));
                  // }
            }
        }

        for extension in extend_dirs {
            let aliases = Self::add_extension_to_scope(&extension, &module_path, scope)?;

            for alias in aliases {
                module.members.push(Spanned::new(
                    ModuleMemberDeclaration::Alias(alias),
                    extension.span(),
                ));
            }
        }

        module.directives.append(&mut other_dirs);

        Ok(())
    }

    fn add_extension_to_scope(
        extend: &Spanned<ExtendDirective>,
        module_path: &ModulePath,
        scope: &mut im::HashMap<String, ScopeMember>,
    ) -> Result<Vec<Alias>, CompilerPassError> {
        let mut extension_path = extend.path.clone();

        let (mut module, module_scope) =
            Self::find_module_and_scope(scope.clone(), &extension_path)?;

        Self::module_to_absolute_path(&mut module, ModulePath(im::Vector::new()), module_scope)?;

        let mut aliases = vec![];

        Self::relative_path_to_absolute_path(
            scope.clone(),
            module_path.clone(),
            &mut extension_path,
        )?;

        for member in module.members.iter() {
            if let Some(name) = member.name() {
                let mut path = extension_path.clone();
                path.push(PathPart {
                    name: name.clone(),
                    template_args: None,
                });
                let alias = Alias {
                    name: name.clone(),
                    typ: Spanned::new(TypeExpression { path }, extend.span()),
                    template_parameters: member
                        .template_parameters()
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|mut x| {
                            x.name.value =
                                Self::mangle_template_parameter_name(module_path, &name, &x.name);
                            x
                        })
                        .collect(),
                };
                let alias_path: ModulePath = module_path.clone();

                scope.insert(
                    alias.name.value.clone(),
                    ScopeMember::ModuleMemberDeclaration(
                        alias_path,
                        ModuleMemberDeclaration::Alias(alias.clone()),
                    ),
                );
                aliases.push(alias);
            }
        }

        Ok(aliases)
    }

    fn translation_unit_to_absolute_path(
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let module_path = ModulePath(im::Vector::new());
        let mut scope = im::HashMap::new();
        let mut other_directives: Vec<Spanned<GlobalDirective>> = vec![];
        let mut extend_directives = vec![];

        let builtin_functions = get_builtin_functions();
        let builtin_tokens = get_builtin_tokens();

        scope = scope.union(
            builtin_tokens
                .builtin_values
                .keys()
                .chain(builtin_tokens.type_aliases.keys())
                .chain(builtin_functions.functions.keys())
                .chain(builtin_tokens.primitive_types.iter())
                .map(|x| (x.clone(), ScopeMember::BuiltIn))
                .collect(),
        );

        for decl in translation_unit.global_declarations.iter() {
            if let Some(name) = decl.name().as_ref() {
                scope.insert(
                    name.value.clone(),
                    ScopeMember::GlobalDeclaration(decl.as_ref().clone()),
                );
            }
        }

        for dir in translation_unit.global_directives.drain(..) {
            let span = dir.span();
            match dir.value {
                GlobalDirective::Use(usage) => {
                    Self::add_usage_to_scope(usage, module_path.clone(), &mut scope)?;
                }
                GlobalDirective::Extend(mut extend) => {
                    extend_directives.push(Spanned::new(extend.clone(), span));
                    Self::relative_path_to_absolute_path(
                        scope.clone(),
                        module_path.clone(),
                        &mut extend.path,
                    )?;
                }
                other => other_directives.push(Spanned::new(other, span)),
            }
        }
        translation_unit
            .global_directives
            .append(&mut other_directives);

        for extend in extend_directives {
            let aliases = Self::add_extension_to_scope(&extend, &module_path, &mut scope)?;
            for alias in aliases {
                translation_unit
                    .global_declarations
                    .push(Spanned::new(GlobalDeclaration::Alias(alias), extend.span()));
            }
        }

        for decl in translation_unit.global_declarations.iter_mut() {
            match decl.as_mut() {
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
