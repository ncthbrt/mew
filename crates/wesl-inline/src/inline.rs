use wesl_parse::{
    span::Spanned,
    syntax::{
        Alias, CaseSelector, CompoundStatement, ConstAssert, Declaration, Expression,
        ExtendDirective, FormalTemplateParameter, Function, GlobalDeclaration, GlobalDirective,
        IdentifierExpression, Module, ModuleDirective, ModuleMemberDeclaration, PathPart,
        Statement, Struct, TranslationUnit, TypeExpression, Use,
    },
};
use wesl_types::{CompilerPass, CompilerPassError};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Inliner;

enum Parent<'a> {
    Module(&'a mut Module),
    TranslationUnit(&'a mut TranslationUnit),
}

impl<'a> Parent<'a> {
    fn inline_path<'b>(&'b mut self, path: &mut Vec<PathPart>) -> Result<(), CompilerPassError> {
        for p in path.iter_mut() {
            if let Some(mut inline_args) = p.inline_template_args.take() {
                for directive in inline_args.directives.drain(..) {
                    match directive.value {
                        ModuleDirective::Use(usage) => self.usage_to_inline(usage)?,
                        ModuleDirective::Extend(extend_directive) => {
                            self.extend_to_inline(extend_directive)?;
                        }
                    }
                }

                for arg in inline_args.members {
                    self.add_member(arg);
                }
            }
        }
        Ok(())
    }

    fn inline_expression<'b>(
        &'b mut self,
        expression: &mut Expression,
    ) -> Result<(), CompilerPassError> {
        match expression {
            Expression::Literal(_) => Ok(()),
            Expression::Parenthesized(spanned) => self.inline_expression(spanned),
            Expression::NamedComponent(named_component_expression) => {
                self.inline_expression(&mut named_component_expression.base)
            }
            Expression::Indexing(indexing_expression) => {
                self.inline_expression(&mut indexing_expression.base)
            }
            Expression::Unary(unary_expression) => {
                self.inline_expression(&mut unary_expression.operand)
            }
            Expression::Binary(binary_expression) => {
                self.inline_expression(&mut binary_expression.left)?;
                self.inline_expression(&mut binary_expression.right)?;
                Ok(())
            }
            Expression::FunctionCall(function_call_expression) => {
                self.inline_path(&mut function_call_expression.path)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    self.inline_expression(arg)?;
                }
                Ok(())
            }
            Expression::Identifier(IdentifierExpression { path })
            | Expression::Type(TypeExpression { path }) => self.inline_path(path),
        }
    }

    fn inline_compound_statement<'b>(
        &'b mut self,
        statement: &mut CompoundStatement,
    ) -> Result<(), CompilerPassError> {
        for arg in statement
            .attributes
            .iter_mut()
            .flat_map(|x| x.arguments.iter_mut())
            .flatten()
        {
            self.inline_expression(&mut arg.value)?;
        }

        for statement in statement.statements.iter_mut() {
            self.inline_statement(statement)?;
        }

        Ok(())
    }

    fn inline_statement<'b>(
        &'b mut self,
        statement: &mut Statement,
    ) -> Result<(), CompilerPassError> {
        match statement {
            Statement::Void => Ok(()),
            Statement::Compound(compound_statement) => {
                self.inline_compound_statement(compound_statement)
            }
            Statement::Assignment(assignment_statement) => {
                self.inline_expression(&mut assignment_statement.lhs)?;
                self.inline_expression(&mut assignment_statement.rhs)
            }
            Statement::Increment(expression) => self.inline_expression(expression),
            Statement::Decrement(expression) => self.inline_expression(expression),
            Statement::If(if_statement) => {
                for expr in if_statement
                    .attributes
                    .iter_mut()
                    .flat_map(|x| x.arguments.iter_mut().flatten())
                {
                    self.inline_expression(expr.as_mut())?;
                }

                self.inline_compound_statement(&mut if_statement.if_clause.1)?;
                self.inline_expression(&mut if_statement.if_clause.0)?;

                for elif in if_statement.else_if_clauses.iter_mut() {
                    self.inline_compound_statement(&mut elif.1)?;
                    self.inline_expression(&mut elif.0)?;
                }

                if let Some(els) = if_statement.else_clause.as_mut() {
                    self.inline_compound_statement(els)?;
                }

                Ok(())
            }
            Statement::Switch(switch_statement) => {
                for arg in switch_statement
                    .attributes
                    .iter_mut()
                    .chain(switch_statement.body_attributes.iter_mut())
                    .flat_map(|x| x.arguments.iter_mut().flatten())
                {
                    self.inline_expression(arg)?;
                }
                self.inline_expression(&mut switch_statement.expression)?;
                for clause in switch_statement.clauses.iter_mut() {
                    self.inline_compound_statement(&mut clause.body)?;
                    for case_seletor in clause.case_selectors.iter_mut() {
                        if let CaseSelector::Expression(expr) = case_seletor.as_mut() {
                            self.inline_expression(expr)?;
                        }
                    }
                }
                Ok(())
            }
            Statement::Loop(loop_statement) => {
                for arg in loop_statement
                    .attributes
                    .iter_mut()
                    .flat_map(|x| x.arguments.iter_mut().flatten())
                {
                    self.inline_expression(arg)?;
                }

                self.inline_compound_statement(&mut loop_statement.body)?;

                if let Some(continuing) = loop_statement.continuing.as_mut() {
                    self.inline_compound_statement(&mut continuing.body)?;
                    if let Some(break_if) = continuing.break_if.as_mut() {
                        self.inline_expression(break_if)?;
                    }
                }
                Ok(())
            }
            Statement::For(for_statement) => {
                for arg in for_statement
                    .attributes
                    .iter_mut()
                    .flat_map(|x| x.arguments.iter_mut().flatten())
                {
                    self.inline_expression(arg)?;
                }

                self.inline_compound_statement(&mut for_statement.body)?;

                if let Some(cond) = for_statement.condition.as_mut() {
                    self.inline_expression(cond)?;
                }

                if let Some(statement) = for_statement.initializer.as_mut() {
                    self.inline_statement(statement)?;
                }

                if let Some(statement) = for_statement.update.as_mut() {
                    self.inline_statement(statement)?;
                }

                Ok(())
            }
            Statement::While(while_statement) => {
                for arg in while_statement
                    .attributes
                    .iter_mut()
                    .flat_map(|x| x.arguments.iter_mut())
                    .flatten()
                {
                    self.inline_expression(arg)?;
                }

                self.inline_compound_statement(&mut while_statement.body)?;
                self.inline_expression(&mut while_statement.condition)?;
                Ok(())
            }
            Statement::Break => Ok(()),
            Statement::Continue => Ok(()),
            Statement::Return(spanned) => {
                if let Some(expr) = spanned.as_mut() {
                    self.inline_expression(expr)?;
                }
                Ok(())
            }
            Statement::Discard => Ok(()),
            Statement::FunctionCall(function_call_expression) => {
                self.inline_path(&mut function_call_expression.path)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    self.inline_expression(arg)?;
                }
                Ok(())
            }
            Statement::ConstAssert(const_assert) => {
                self.inline_expression(&mut const_assert.expression)?;
                Ok(())
            }
            Statement::Declaration(declaration_statement) => {
                self.declaration_to_inline(&mut declaration_statement.declaration)?;

                for statement in declaration_statement.statements.iter_mut() {
                    self.inline_statement(statement)?;
                }
                Ok(())
            }
        }
    }

    fn function_to_inline<'b>(&'b mut self, func: &mut Function) -> Result<(), CompilerPassError> {
        self.inline_template_params(&mut func.template_parameters)?;
        self.inline_compound_statement(&mut func.body)?;

        for func_param in func.parameters.iter_mut() {
            self.inline_path(&mut func_param.typ.path)?;
        }

        for expr in func
            .attributes
            .iter_mut()
            .chain(func.return_attributes.iter_mut())
            .chain(
                func.parameters
                    .iter_mut()
                    .flat_map(|x| x.attributes.iter_mut()),
            )
            .flat_map(|x| x.arguments.iter_mut().flatten())
        {
            self.inline_expression(expr.as_mut())?;
        }

        if let Some(ret) = func.return_type.as_mut() {
            self.inline_path(&mut ret.path)?;
        }

        Ok(())
    }

    fn inline_template_params<'b>(
        &'b mut self,
        template_params: &mut Vec<Spanned<FormalTemplateParameter>>,
    ) -> Result<(), CompilerPassError> {
        for p in template_params.iter_mut() {
            if let Some(def) = p.default_value.as_mut() {
                self.inline_expression(&mut def.value)?;
            }
        }
        Ok(())
    }

    fn alias_to_inline<'b>(&'b mut self, alias: &mut Alias) -> Result<(), CompilerPassError> {
        self.inline_template_params(&mut alias.template_parameters)?;
        self.inline_path(&mut alias.typ.path)?;
        Ok(())
    }

    fn declaration_to_inline<'b>(
        &'b mut self,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerPassError> {
        self.inline_template_params(&mut declaration.template_parameters)?;
        if let Some(typ) = declaration.typ.as_mut() {
            self.inline_path(&mut typ.path)?;
        }

        if let Some(init) = declaration.initializer.as_mut() {
            self.inline_expression(init)?;
        }

        for expr in declaration
            .attributes
            .iter_mut()
            .flat_map(|x| x.arguments.iter_mut().flatten())
        {
            self.inline_expression(expr.as_mut())?;
        }
        Ok(())
    }

    fn usage_to_inline<'b>(&'b mut self, mut usage: Use) -> Result<(), CompilerPassError> {
        self.inline_path(&mut usage.path)?;
        for arg in usage
            .attributes
            .iter_mut()
            .flat_map(|x| x.arguments.iter_mut())
            .flatten()
        {
            self.inline_expression(&mut arg.value)?;
        }
        match usage.content.as_mut() {
            wesl_parse::syntax::UseContent::Item(use_item) => {
                if let Some(mut args) = use_item.inline_template_args.take() {
                    for directive in args.directives.drain(..) {
                        match directive.value {
                            ModuleDirective::Use(usage) => self.usage_to_inline(usage)?,
                            ModuleDirective::Extend(extend_directive) => {
                                self.extend_to_inline(extend_directive)?;
                            }
                        }
                    }

                    for arg in args.members {
                        self.add_member(arg);
                    }
                }
            }
            wesl_parse::syntax::UseContent::Collection(vec) => {
                for item in vec.drain(..) {
                    self.usage_to_inline(item.value)?;
                }
            }
        }

        Ok(())
    }

    fn struct_to_inline<'b>(&'b mut self, strct: &mut Struct) -> Result<(), CompilerPassError> {
        self.inline_template_params(&mut strct.template_parameters)?;
        for member in strct.members.iter_mut() {
            self.inline_path(&mut member.typ.path)?;
            for arg in member
                .attributes
                .iter_mut()
                .flat_map(|x| x.arguments.iter_mut())
                .flatten()
            {
                self.inline_expression(&mut arg.value)?;
            }
        }
        Ok(())
    }

    fn const_assert_to_inline<'b>(
        &'b mut self,
        const_assert: &mut ConstAssert,
    ) -> Result<(), CompilerPassError> {
        self.inline_template_params(&mut const_assert.template_parameters)?;
        self.inline_expression(&mut const_assert.expression)?;
        Ok(())
    }

    fn extend_to_inline<'b>(
        &'b mut self,
        mut extend: ExtendDirective,
    ) -> Result<(), CompilerPassError> {
        for arg in extend
            .attributes
            .iter_mut()
            .flat_map(|x| x.arguments.iter_mut())
            .flatten()
        {
            self.inline_expression(&mut arg.value)?;
        }
        self.inline_path(&mut extend.path)?;
        Ok(())
    }

    fn add_member(&mut self, child: Spanned<ModuleMemberDeclaration>) {
        match self {
            Parent::Module(m) => {
                m.members.push(child);
            }
            Parent::TranslationUnit(t) => {
                let span = child.span();
                t.global_declarations
                    .push(Spanned::new(child.value.into(), span));
            }
        }
    }

    fn inline<'b>(&'b mut self) -> Result<(), CompilerPassError> {
        match self {
            Parent::Module(m) => {
                let mut other_directives = vec![];
                for Spanned { value, span: _ } in m
                    .directives
                    .drain(..)
                    .collect::<Vec<Spanned<ModuleDirective>>>()
                {
                    let mut parent: Parent<'_> = Parent::Module(m);
                    match value {
                        ModuleDirective::Use(usage) => {
                            parent.usage_to_inline(usage)?;
                        }
                        ModuleDirective::Extend(extend_directive) => {
                            parent.extend_to_inline(extend_directive)?;
                        } // other => other_directives.push(Spanned::new(other, span)),
                    }
                }
                m.directives.append(&mut other_directives);

                for mut member in m
                    .members
                    .drain(..)
                    .collect::<Vec<Spanned<ModuleMemberDeclaration>>>()
                {
                    let mut parent: Parent<'_> = Parent::Module(m);
                    match &mut member.value {
                        ModuleMemberDeclaration::Void => {}
                        ModuleMemberDeclaration::Declaration(declaration) => {
                            parent.declaration_to_inline(declaration)?;
                        }
                        ModuleMemberDeclaration::Alias(alias) => {
                            parent.alias_to_inline(alias)?;
                        }
                        ModuleMemberDeclaration::Struct(strct) => {
                            parent.struct_to_inline(strct)?;
                        }
                        ModuleMemberDeclaration::Function(function) => {
                            parent.function_to_inline(function)?;
                        }
                        ModuleMemberDeclaration::ConstAssert(const_assert) => {
                            parent.const_assert_to_inline(const_assert)?;
                        }
                        ModuleMemberDeclaration::Module(module) => {
                            let mut parent = Parent::Module(module);
                            parent.inline()?;
                        }
                    }
                    parent.add_member(member);
                }
            }
            Parent::TranslationUnit(t) => {
                let mut other_directives = vec![];
                for Spanned { value, span } in t
                    .global_directives
                    .drain(..)
                    .collect::<Vec<Spanned<GlobalDirective>>>()
                {
                    let mut parent: Parent<'_> = Parent::TranslationUnit(t);

                    match value {
                        GlobalDirective::Use(usage) => {
                            parent.usage_to_inline(usage)?;
                        }
                        GlobalDirective::Extend(extend_directive) => {
                            parent.extend_to_inline(extend_directive)?;
                        }
                        other => other_directives.push(Spanned::new(other, span)),
                    }
                }
                t.global_directives.append(&mut other_directives);

                for mut member in t
                    .global_declarations
                    .drain(..)
                    .collect::<Vec<Spanned<GlobalDeclaration>>>()
                {
                    let mut parent: Parent<'_> = Parent::TranslationUnit(t);
                    match &mut member.value {
                        GlobalDeclaration::Void => {}
                        GlobalDeclaration::Declaration(declaration) => {
                            parent.declaration_to_inline(declaration)?;
                        }
                        GlobalDeclaration::Alias(alias) => {
                            parent.alias_to_inline(alias)?;
                        }
                        GlobalDeclaration::Struct(strct) => {
                            parent.struct_to_inline(strct)?;
                        }
                        GlobalDeclaration::Function(function) => {
                            parent.function_to_inline(function)?;
                        }
                        GlobalDeclaration::ConstAssert(const_assert) => {
                            parent.const_assert_to_inline(const_assert)?;
                        }
                        GlobalDeclaration::Module(module) => {
                            let mut parent = Parent::Module(module);
                            parent.inline()?;
                        }
                    }
                    t.global_declarations.push(member);
                }
            }
        }

        Ok(())
    }
}

impl CompilerPass for Inliner {
    fn apply_mut(
        &mut self,
        translation_unit: &mut wesl_parse::syntax::TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let mut parent: Parent<'_> = Parent::TranslationUnit(translation_unit);
        parent.inline()?;
        Ok(())
    }
}
