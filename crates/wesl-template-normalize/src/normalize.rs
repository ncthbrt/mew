use std::collections::VecDeque;

use wesl_parse::{span::Spanned, syntax::*};
use wesl_types::{CompilerPass, CompilerPassError};

#[derive(Debug, Default, Clone, Copy)]
pub struct TemplateNormalizer;

#[derive(Debug, PartialEq, Clone, Hash)]
enum GenericMember<'a> {
    Func(&'a Function),
    Struct(&'a Struct),
    Mod(&'a Module),
    Alias(&'a Alias),
    Declaration(&'a Declaration),
}

impl<'a> GenericMember<'a> {
    fn template_params(&self) -> &Vec<Spanned<FormalTemplateParameter>> {
        match self {
            GenericMember::Func(function) => function.template_parameters.as_ref(),
            GenericMember::Mod(module) => module.template_parameters.as_ref(),
            GenericMember::Struct(strct) => strct.template_parameters.as_ref(),
            GenericMember::Alias(alias) => alias.template_parameters.as_ref(),
            GenericMember::Declaration(decl) => decl.template_parameters.as_ref(),
        }
    }
}

impl TemplateNormalizer {
    fn template_args_to_none_if_empty(path: &mut Spanned<Vec<PathPart>>) {
        for p in path.iter_mut() {
            if let Some(t) = p.template_args.as_ref() {
                if t.is_empty() {
                    p.template_args = None;
                }
            }
        }
    }

    fn normalize_path_part(
        generic_member: &GenericMember,
        path_part: &mut PathPart,
        translation_unit: &TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let mut template_args = path_part.template_args.take().unwrap_or_default();
        let template_params = generic_member.template_params();
        let mut result: Vec<Spanned<TemplateArg>> = vec![];
        for (idx, param) in template_params.iter().enumerate() {
            if let Some(default_value) = param.default_value.as_ref() {
                let mut value = if let Some(value) = template_args
                    .iter()
                    .find(|x| x.arg_name.as_ref() == Some(&param.name))
                    .cloned()
                {
                    value
                } else {
                    Spanned::new(
                        TemplateArg {
                            expression: default_value.clone(),
                            arg_name: Some(param.name.clone()),
                        },
                        default_value.span(),
                    )
                };
                Self::normalize_template_arguments_from_expr(
                    &mut value.expression,
                    translation_unit,
                )?;
                result.push(value);
            } else if let Some(template_arg) = template_args.get_mut(idx) {
                if template_arg.arg_name.is_none()
                    || template_arg.arg_name.as_ref() == Some(&param.name)
                {
                    template_arg.arg_name = Some(param.name.clone());
                    Self::normalize_template_arguments_from_expr(
                        &mut template_arg.expression,
                        translation_unit,
                    )?;
                    result.push(template_arg.clone());
                } else {
                    return Err(CompilerPassError::UnknownTemplateArgument(
                        template_arg.span(),
                    ));
                }
            } else {
                return Err(CompilerPassError::MissingRequiredTemplateArgument(
                    param.clone(),
                    path_part.name.span(),
                ));
            }
        }
        if let Some(args) = path_part.template_args.as_mut() {
            args.append(&mut result);
        } else {
            path_part.template_args = Some(result);
        }

        Ok(())
    }

    fn normalize_path(
        path: &mut Spanned<Vec<PathPart>>,
        translation_unit: &TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        assert!(!path.is_empty());
        Self::template_args_to_none_if_empty(path);

        let mut remaining_path: VecDeque<&mut PathPart> = path.value.iter_mut().collect();
        let fst: &mut PathPart = remaining_path.pop_front().unwrap();

        if let Some(generic_member) =
            translation_unit
                .global_declarations
                .iter()
                .find_map(|x| match x.as_ref() {
                    GlobalDeclaration::Module(m) => {
                        if m.name == fst.name {
                            Some(GenericMember::Mod(m))
                        } else {
                            None
                        }
                    }
                    GlobalDeclaration::Function(f) => {
                        if f.name == fst.name {
                            Some(GenericMember::Func(f))
                        } else {
                            None
                        }
                    }
                    GlobalDeclaration::Struct(s) => {
                        if s.name == fst.name {
                            Some(GenericMember::Struct(s))
                        } else {
                            None
                        }
                    }
                    GlobalDeclaration::Alias(a) => {
                        if a.name == fst.name {
                            Some(GenericMember::Alias(a))
                        } else {
                            None
                        }
                    }
                    GlobalDeclaration::Declaration(d) => {
                        if d.name == fst.name {
                            Some(GenericMember::Declaration(d))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
        {
            let mut generic_member = generic_member;
            Self::normalize_path_part(&generic_member, fst, translation_unit)?;

            let process_alias = |a: &Alias,
                                 mut remaining_path: VecDeque<&mut PathPart>|
             -> Result<(), CompilerPassError> {
                let mut remaining_path_with_alias: Spanned<Vec<PathPart>> =
                    Spanned::new(vec![], a.typ.path.span());
                remaining_path_with_alias.append(&mut a.typ.path.clone());

                for p in remaining_path.iter() {
                    remaining_path_with_alias.push((**p).clone());
                }
                Self::normalize_path(&mut remaining_path_with_alias, translation_unit)?;
                for (part, resultant_part) in remaining_path
                    .iter_mut()
                    .zip(remaining_path_with_alias.into_iter().skip(a.typ.path.len()))
                {
                    part.template_args = resultant_part.template_args;
                }

                Ok(())
            };

            'outer: while !remaining_path.is_empty() {
                match &generic_member {
                    GenericMember::Func(_) => {
                        return Err(CompilerPassError::SymbolNotFound(
                            path.value.clone(),
                            path.span(),
                        ));
                    }
                    GenericMember::Declaration(_) => {
                        return Err(CompilerPassError::SymbolNotFound(
                            path.value.clone(),
                            path.span(),
                        ));
                    }
                    GenericMember::Alias(a) => {
                        return process_alias(a, remaining_path);
                    }
                    GenericMember::Struct(_) => {
                        return Err(CompilerPassError::SymbolNotFound(
                            path.value.clone(),
                            path.span(),
                        ));
                    }
                    GenericMember::Mod(m) => {
                        for decl in m.members.iter() {
                            match decl.as_ref() {
                                ModuleMemberDeclaration::Module(inner) => {
                                    if inner.name.value
                                        == remaining_path.front().as_ref().unwrap().name.value
                                    {
                                        let path_part: &mut PathPart =
                                            remaining_path.pop_front().unwrap();
                                        generic_member = GenericMember::Mod(inner);
                                        Self::normalize_path_part(
                                            &generic_member,
                                            path_part,
                                            translation_unit,
                                        )?;
                                        continue 'outer;
                                    }
                                }
                                ModuleMemberDeclaration::Function(func) => {
                                    if func.name.value
                                        == remaining_path.front().as_ref().unwrap().name.value
                                    {
                                        let path_part = remaining_path.pop_front().unwrap();
                                        generic_member = GenericMember::Func(func);
                                        Self::normalize_path_part(
                                            &generic_member,
                                            path_part,
                                            translation_unit,
                                        )?;
                                        continue 'outer;
                                    }
                                }
                                ModuleMemberDeclaration::Struct(s) => {
                                    if s.name.value
                                        == remaining_path.front().as_ref().unwrap().name.value
                                    {
                                        let path_part = remaining_path.pop_front().unwrap();
                                        generic_member = GenericMember::Struct(s);
                                        Self::normalize_path_part(
                                            &generic_member,
                                            path_part,
                                            translation_unit,
                                        )?;
                                        continue 'outer;
                                    }
                                }
                                ModuleMemberDeclaration::Alias(a) => {
                                    if a.name.value
                                        == remaining_path.front().as_ref().unwrap().name.value
                                    {
                                        let path_part = remaining_path.pop_front().unwrap();
                                        generic_member = GenericMember::Alias(a);
                                        Self::normalize_path_part(
                                            &generic_member,
                                            path_part,
                                            translation_unit,
                                        )?;
                                        return process_alias(a, remaining_path);
                                    }
                                }
                                ModuleMemberDeclaration::Void => {}
                                ModuleMemberDeclaration::ConstAssert(_) => {}
                                ModuleMemberDeclaration::Declaration(d) => {
                                    if d.name.value
                                        == remaining_path.front().as_ref().unwrap().name.value
                                    {
                                        let path_part = remaining_path.pop_front().unwrap();
                                        generic_member = GenericMember::Declaration(d);
                                        Self::normalize_path_part(
                                            &generic_member,
                                            path_part,
                                            translation_unit,
                                        )?;
                                        continue 'outer;
                                    }
                                }
                            }
                        }

                        return Err(CompilerPassError::SymbolNotFound(
                            path.value.clone(),
                            path.span(),
                        ));
                    }
                }
            }
            return Ok(());
        } else {
            for part in vec![fst].iter_mut().chain(remaining_path.iter_mut()) {
                for arg in part.template_args.iter_mut().flatten() {
                    Self::normalize_template_arguments_from_expr(
                        &mut arg.expression,
                        translation_unit,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn normalize_template_arguments_from_module(
        module: &mut Module,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for decl in module.directives.iter_mut() {
            match &mut decl.value {
                ModuleDirective::Use(_) => {
                    panic!("USE SHOULD HAVE ALREADY BEEN REMOVED");
                }
                ModuleDirective::Extend(extend_directive) => {
                    Self::normalize_path(&mut extend_directive.path, &translation_unit)?;
                }
            }
        }
        for decl in module.members.iter_mut() {
            match decl.as_mut() {
                ModuleMemberDeclaration::Void => {
                    // NO ACTION REQUIRED
                }
                ModuleMemberDeclaration::Declaration(decl) => {
                    Self::normalize_template_arguments_from_decl(decl, translation_unit)?;
                }
                ModuleMemberDeclaration::Alias(alias) => {
                    Self::normalize_template_arguments_from_type(&mut alias.typ, translation_unit)?;
                }
                ModuleMemberDeclaration::Struct(s) => {
                    Self::normalize_template_arguments_from_struct(s, translation_unit)?;
                }
                ModuleMemberDeclaration::Function(f) => {
                    Self::normalize_template_arguments_from_function(f, translation_unit)?;
                }
                ModuleMemberDeclaration::ConstAssert(assrt) => {
                    Self::normalize_template_arguments_from_const_assert(assrt, translation_unit)?;
                }
                ModuleMemberDeclaration::Module(m) => {
                    Self::normalize_template_arguments_from_module(m, translation_unit)?;
                }
            }
        }
        Ok(())
    }

    fn normalize_template_arguments_from_expr(
        expr: &mut Expression,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        match expr {
            Expression::Literal(_) => {
                // No action required
            }
            Expression::Parenthesized(spanned) => {
                Self::normalize_template_arguments_from_expr(spanned, translation_unit)?;
            }
            Expression::NamedComponent(named_component_expression) => {
                Self::normalize_template_arguments_from_expr(
                    &mut named_component_expression.base,
                    translation_unit,
                )?;
            }
            Expression::Indexing(indexing_expression) => {
                Self::normalize_template_arguments_from_expr(
                    &mut indexing_expression.base,
                    translation_unit,
                )?;
            }
            Expression::Unary(unary_expression) => {
                Self::normalize_template_arguments_from_expr(
                    &mut unary_expression.operand,
                    translation_unit,
                )?;
            }
            Expression::Binary(binary_expression) => {
                Self::normalize_template_arguments_from_expr(
                    &mut binary_expression.left,
                    translation_unit,
                )?;
                Self::normalize_template_arguments_from_expr(
                    &mut binary_expression.right,
                    translation_unit,
                )?;
            }
            Expression::FunctionCall(function_call_expression) => {
                Self::normalize_path(&mut function_call_expression.path, translation_unit)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::normalize_template_arguments_from_expr(arg, translation_unit)?;
                }
            }
            Expression::Identifier(identifier_expression) => {
                Self::normalize_path(&mut identifier_expression.path, translation_unit)?;
            }
            Expression::Type(type_expression) => {
                Self::normalize_template_arguments_from_type(type_expression, translation_unit)?;
            }
        }
        Ok(())
    }

    fn normalize_template_arguments_from_statement(
        statement: &mut Statement,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        match statement {
            Statement::Void => {
                // No action required
            }
            Statement::Compound(compound_statement) => {
                Self::normalize_template_arguments_from_compound_statement(
                    compound_statement,
                    translation_unit,
                )?;
            }
            Statement::Assignment(assignment_statement) => {
                Self::normalize_template_arguments_from_expr(
                    &mut assignment_statement.lhs,
                    translation_unit,
                )?;
                Self::normalize_template_arguments_from_expr(
                    &mut assignment_statement.rhs,
                    translation_unit,
                )?;
            }
            Statement::Increment(expression) => {
                Self::normalize_template_arguments_from_expr(expression, translation_unit)?;
            }
            Statement::Decrement(expression) => {
                Self::normalize_template_arguments_from_expr(expression, translation_unit)?;
            }
            Statement::If(iff) => {
                Self::normalize_template_arguments_from_expr(
                    &mut iff.if_clause.0,
                    translation_unit,
                )?;
                Self::normalize_template_arguments_from_compound_statement(
                    &mut iff.if_clause.1,
                    translation_unit,
                )?;
                for (else_if_expr, else_if_statements) in iff.else_if_clauses.iter_mut() {
                    Self::normalize_template_arguments_from_expr(else_if_expr, translation_unit)?;
                    Self::normalize_template_arguments_from_compound_statement(
                        else_if_statements,
                        translation_unit,
                    )?;
                }
                if let Some(else_clause) = iff.else_clause.as_mut() {
                    Self::normalize_template_arguments_from_compound_statement(
                        else_clause,
                        translation_unit,
                    )?;
                }
            }
            Statement::Switch(s) => {
                Self::normalize_template_arguments_from_expr(&mut s.expression, translation_unit)?;
                for clause in s.clauses.iter_mut() {
                    for c in clause.case_selectors.iter_mut() {
                        match &mut c.value {
                            wesl_parse::syntax::CaseSelector::Default => {
                                // NO ACTION NEEDED
                            }
                            wesl_parse::syntax::CaseSelector::Expression(e) => {
                                Self::normalize_template_arguments_from_expr(e, translation_unit)?;
                            }
                        }
                    }
                    Self::normalize_template_arguments_from_compound_statement(
                        &mut clause.body,
                        translation_unit,
                    )?;
                }
            }
            Statement::Loop(l) => {
                Self::normalize_template_arguments_from_compound_statement(
                    &mut l.body,
                    translation_unit,
                )?;
                if let Some(cont) = l.continuing.as_mut() {
                    Self::normalize_template_arguments_from_compound_statement(
                        &mut l.body,
                        translation_unit,
                    )?;
                    if let Some(expr) = cont.break_if.as_mut() {
                        Self::normalize_template_arguments_from_expr(expr, translation_unit)?;
                    }
                }
            }
            Statement::For(f) => {
                if let Some(init) = f.initializer.as_mut() {
                    Self::normalize_template_arguments_from_statement(
                        init.as_mut(),
                        translation_unit,
                    )?;
                }
                if let Some(cond) = f.condition.as_mut() {
                    Self::normalize_template_arguments_from_expr(cond, translation_unit)?;
                }
                if let Some(update) = f.update.as_mut() {
                    Self::normalize_template_arguments_from_statement(
                        update.as_mut(),
                        translation_unit,
                    )?;
                }
                Self::normalize_template_arguments_from_compound_statement(
                    &mut f.body,
                    translation_unit,
                )?;
            }
            Statement::While(w) => {
                Self::normalize_template_arguments_from_expr(&mut w.condition, translation_unit)?;
                Self::normalize_template_arguments_from_compound_statement(
                    &mut w.body,
                    translation_unit,
                )?;
            }
            Statement::Break => {
                // No action required
            }
            Statement::Continue => {
                // No action required
            }
            Statement::Return(spanned) => {
                if let Some(expr) = spanned.as_mut() {
                    Self::normalize_template_arguments_from_expr(expr, translation_unit)?;
                }
            }
            Statement::Discard => {
                // No action required
            }
            Statement::FunctionCall(function_call_expression) => {
                Self::normalize_path(&mut function_call_expression.path, translation_unit)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::normalize_template_arguments_from_expr(arg, translation_unit)?;
                }
            }
            Statement::ConstAssert(const_assert) => {
                Self::normalize_template_arguments_from_const_assert(
                    const_assert,
                    translation_unit,
                )?;
            }
            Statement::Declaration(declaration_statement) => {
                Self::normalize_template_arguments_from_decl(
                    &mut declaration_statement.declaration,
                    translation_unit,
                )?;
                for statement in declaration_statement.statements.iter_mut() {
                    Self::normalize_template_arguments_from_statement(statement, translation_unit)?;
                }
            }
        }
        Ok(())
    }

    fn normalize_template_arguments_from_type(
        expr: &mut TypeExpression,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Self::normalize_path(&mut expr.path, translation_unit)?;
        Ok(())
    }

    fn normalize_template_arguments_from_decl(
        decl: &mut Declaration,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        if let Some(init) = decl.initializer.as_mut() {
            Self::normalize_template_arguments_from_expr(init.as_mut(), translation_unit)?;
        }

        if let Some(typ) = decl.typ.as_mut() {
            Self::normalize_template_arguments_from_type(typ, translation_unit)?;
        }

        Ok(())
    }

    fn normalize_template_arguments_from_struct(
        strct: &mut Struct,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for m in strct.members.iter_mut() {
            Self::normalize_template_arguments_from_type(&mut m.typ, translation_unit)?;
        }
        Ok(())
    }

    fn normalize_template_arguments_from_template_params(
        params: &mut Vec<Spanned<FormalTemplateParameter>>,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for p in params {
            if let Some(def) = p.default_value.as_mut() {
                Self::normalize_template_arguments_from_expr(def, translation_unit)?;
            }
        }
        Ok(())
    }

    fn normalize_template_arguments_from_compound_statement(
        statement: &mut CompoundStatement,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        for statement in statement.statements.iter_mut() {
            Self::normalize_template_arguments_from_statement(
                statement.as_mut(),
                translation_unit,
            )?;
        }
        Ok(())
    }

    fn normalize_template_arguments_from_function(
        func: &mut Function,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        if let Some(r) = func.return_type.as_mut() {
            Self::normalize_template_arguments_from_type(r, translation_unit)?;
        }
        Self::normalize_template_arguments_from_template_params(
            &mut func.template_parameters,
            translation_unit,
        )?;

        for p in func.parameters.iter_mut() {
            Self::normalize_template_arguments_from_type(&mut p.typ, translation_unit)?;
        }

        Self::normalize_template_arguments_from_compound_statement(
            &mut func.body,
            translation_unit,
        )?;
        Ok(())
    }

    fn normalize_template_arguments_from_const_assert(
        assrt: &mut ConstAssert,
        translation_unit: &TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Self::normalize_template_arguments_from_expr(&mut assrt.expression, translation_unit)?;
        Ok(())
    }

    fn normalize_template_arguments_from_translation_unit(
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        let clone = translation_unit.clone();
        for decl in translation_unit.global_directives.iter_mut() {
            match &mut decl.value {
                GlobalDirective::Diagnostic(_) => {}
                GlobalDirective::Enable(_) => {}
                GlobalDirective::Requires(_) => {}
                GlobalDirective::Use(_) => {
                    panic!("USE SHOULD HAVE ALREADY BEEN REMOVED");
                }
                GlobalDirective::Extend(extend_directive) => {
                    Self::normalize_path(&mut extend_directive.path, &clone)?;
                }
            }
        }
        for decl in translation_unit.global_declarations.iter_mut() {
            match decl.as_mut() {
                GlobalDeclaration::Void => {
                    // NO ACTION REQUIRED REQUIRED
                }
                GlobalDeclaration::Declaration(decl) => {
                    Self::normalize_template_arguments_from_decl(decl, &clone)?;
                }
                GlobalDeclaration::Alias(alias) => {
                    Self::normalize_template_arguments_from_type(&mut alias.typ, &clone)?;
                }
                GlobalDeclaration::Struct(s) => {
                    Self::normalize_template_arguments_from_struct(s, &clone)?;
                }
                GlobalDeclaration::Function(f) => {
                    Self::normalize_template_arguments_from_function(f, &clone)?;
                }
                GlobalDeclaration::ConstAssert(assrt) => {
                    Self::normalize_template_arguments_from_const_assert(assrt, &clone)?;
                }
                GlobalDeclaration::Module(m) => {
                    Self::normalize_template_arguments_from_module(m, &clone)?;
                }
            }
        }
        Ok(())
    }
}

impl CompilerPass for TemplateNormalizer {
    fn apply_mut(
        &mut self,
        translation_unit: &mut wesl_parse::syntax::TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Self::normalize_template_arguments_from_translation_unit(translation_unit)?;
        Ok(())
    }
}
