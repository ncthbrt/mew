use std::collections::VecDeque;

use im::{HashMap, HashSet};
use wesl_parse::{
    span::{Span, Spanned},
    syntax::*,
};
use wesl_types::{mangling::mangle_template_args, CompilerPass, CompilerPassError};

#[derive(Debug, Clone)]
pub struct Specializer {
    pub entrypoint: Option<Vec<PathPart>>,
}

type ConcreteSymbolPath = im::Vector<String>;

#[derive(Debug, Default, Clone)]
struct Usages {
    set: HashSet<im::Vector<PathPart>>,
    queue: VecDeque<im::Vector<PathPart>>,
}

impl Usages {
    fn new() -> Usages {
        Default::default()
    }

    fn insert(&mut self, path: im::Vector<PathPart>) -> bool {
        if self.set.insert(path.clone()).is_none() {
            self.queue.push_front(path);
            return true;
        }
        false
    }

    fn pop(&mut self) -> Option<im::Vector<PathPart>> {
        self.queue.pop_front()
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
enum OwnedMember {
    Global(Spanned<GlobalDeclaration>),
    Module(Spanned<ModuleMemberDeclaration>),
}

#[derive(Debug, PartialEq, Hash)]
enum BorrowedMember<'a> {
    Global {
        declaration: &'a mut Spanned<GlobalDeclaration>,
        is_initialized: bool,
    },
    Module {
        declaration: &'a mut Spanned<ModuleMemberDeclaration>,
        is_initialized: bool,
    },
}

impl OwnedMember {
    fn requires_push_down(&self) -> bool {
        matches!(self, OwnedMember::Global(Spanned {
                value: GlobalDeclaration::Module(m),
                ..
            }) | OwnedMember::Module(Spanned {
                value: ModuleMemberDeclaration::Module(m),
                ..
            }) if !m.template_parameters.is_empty(),
        )
    }

    fn requires_specialization(&self) -> bool {
        !matches!(
            self,
            OwnedMember::Global(Spanned {
                value: GlobalDeclaration::Module(_),
                ..
            }) | OwnedMember::Module(Spanned {
                value: ModuleMemberDeclaration::Module(_),
                ..
            })
        ) && self.template_parameters().is_some()
    }

    fn name_mut(&mut self) -> Option<&mut Spanned<String>> {
        match self {
            OwnedMember::Global(spanned) => spanned.name_mut(),
            OwnedMember::Module(spanned) => spanned.name_mut(),
        }
    }

    fn template_parameters(&self) -> Option<&Vec<Spanned<FormalTemplateParameter>>> {
        match self {
            OwnedMember::Global(Spanned { value: decl, .. }) => decl.template_parameters(),
            OwnedMember::Module(Spanned { value: decl, .. }) => decl.template_parameters(),
        }
    }

    fn specialize(&mut self, mut with: PathPart) -> Result<(), CompilerPassError> {
        if let Some(params) = self.template_parameters().cloned() {
            if let Some(name) = self.name_mut() {
                if let Some(template_args) = with.template_args.as_mut() {
                    template_args
                        .retain(|x| params.iter().any(|y| Some(&y.name) == x.arg_name.as_ref()));
                }
                name.value = mangle_template_args(&with);
            }
        }

        match self {
            OwnedMember::Global(other) => {
                Self::specialize_global_declarations(other, with.clone())?
            }
            OwnedMember::Module(other) => {
                Self::specialize_module_member_declarations(other, with.clone())?
            }
        };
        Ok(())
    }

    fn specialize_global_declarations(
        decl: &mut GlobalDeclaration,
        path_part: PathPart,
    ) -> Result<(), CompilerPassError> {
        match decl {
            GlobalDeclaration::Void => Ok(()),
            GlobalDeclaration::Declaration(declaration) => {
                Self::specialize_declaration(declaration, path_part)
            }
            GlobalDeclaration::Alias(alias) => Self::specialize_alias(alias, path_part),
            GlobalDeclaration::Struct(strct) => Self::specialize_struct(strct, path_part),
            GlobalDeclaration::Function(function) => Self::specialize_function(function, path_part),
            GlobalDeclaration::ConstAssert(const_assert) => {
                Self::specialize_const_assert(const_assert, path_part)
            }
            GlobalDeclaration::Module(_) => {
                panic!("MODULES ARE NOT SPECIALIZED");
            }
        }
    }

    fn specialize_module_member_declarations(
        decl: &mut ModuleMemberDeclaration,
        path_part: PathPart,
    ) -> Result<(), CompilerPassError> {
        match decl {
            ModuleMemberDeclaration::Void => Ok(()),
            ModuleMemberDeclaration::Declaration(declaration) => {
                Self::specialize_declaration(declaration, path_part)
            }
            ModuleMemberDeclaration::Alias(alias) => Self::specialize_alias(alias, path_part),
            ModuleMemberDeclaration::Struct(strct) => Self::specialize_struct(strct, path_part),
            ModuleMemberDeclaration::Function(function) => {
                Self::specialize_function(function, path_part)
            }
            ModuleMemberDeclaration::ConstAssert(const_assert) => {
                Self::specialize_const_assert(const_assert, path_part)
            }
            ModuleMemberDeclaration::Module(_) => {
                panic!("MODULES ARE NOT SPECIALIZED");
            }
        }
    }

    fn substitute_expression(
        expression: &mut Expression,
        name: &String,
        value: &Spanned<TemplateArg>,
    ) -> Result<(), CompilerPassError> {
        match expression {
            Expression::Literal(_) => Ok(()),
            Expression::Parenthesized(spanned) => Self::substitute_expression(spanned, name, value),
            Expression::NamedComponent(named_component_expression) => {
                Self::substitute_expression(&mut named_component_expression.base, name, value)
            }
            Expression::Indexing(indexing_expression) => {
                Self::substitute_expression(&mut indexing_expression.base, name, value)
            }
            Expression::Unary(unary_expression) => {
                Self::substitute_expression(&mut unary_expression.operand, name, value)
            }
            Expression::Binary(binary_expression) => {
                Self::substitute_expression(&mut binary_expression.left, name, value)?;
                Self::substitute_expression(&mut binary_expression.right, name, value)?;
                Ok(())
            }
            Expression::FunctionCall(function_call_expression) => {
                Self::substitute_path(&mut function_call_expression.path, name, value)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::substitute_expression(arg, name, value)?;
                }
                Ok(())
            }
            Expression::Identifier(IdentifierExpression { path })
            | Expression::Type(TypeExpression { path }) => {
                let start_name = path.first().unwrap().name.value.clone();
                if name == &start_name {
                    if path.len() == 1 {
                        *expression = value.expression.clone().value;
                    } else {
                        Self::substitute_path(path, name, value)?;
                    }
                    Ok(())
                } else {
                    Self::substitute_path(path, name, value)
                }
            }
        }
    }

    fn substitute_compound_statement(
        statement: &mut CompoundStatement,
        name: &String,
        value: &Spanned<TemplateArg>,
    ) -> Result<(), CompilerPassError> {
        for arg in statement
            .attributes
            .iter_mut()
            .flat_map(|x| x.arguments.iter_mut())
            .flatten()
        {
            Self::substitute_expression(&mut arg.value, name, value)?;
        }

        for statement in statement.statements.iter_mut() {
            Self::substitute_statement(statement, name, value)?;
        }

        Ok(())
    }

    fn substitute_statement(
        statement: &mut Statement,
        name: &String,
        value: &Spanned<TemplateArg>,
    ) -> Result<(), CompilerPassError> {
        match statement {
            Statement::Void => Ok(()),
            Statement::Compound(compound_statement) => {
                Self::substitute_compound_statement(compound_statement, name, value)
            }
            Statement::Assignment(assignment_statement) => {
                Self::substitute_expression(&mut assignment_statement.lhs, name, value)?;
                Self::substitute_expression(&mut assignment_statement.rhs, name, value)
            }
            Statement::Increment(expression) => {
                Self::substitute_expression(expression, name, value)
            }
            Statement::Decrement(expression) => {
                Self::substitute_expression(expression, name, value)
            }
            Statement::If(if_statement) => {
                for expr in if_statement
                    .attributes
                    .iter_mut()
                    .flat_map(|x| x.arguments.iter_mut().flatten())
                {
                    Self::substitute_expression(expr.as_mut(), name, value)?;
                }

                Self::substitute_compound_statement(&mut if_statement.if_clause.1, name, value)?;
                Self::substitute_expression(&mut if_statement.if_clause.0, name, value)?;

                for elif in if_statement.else_if_clauses.iter_mut() {
                    Self::substitute_compound_statement(&mut elif.1, name, value)?;
                    Self::substitute_expression(&mut elif.0, name, value)?;
                }

                if let Some(els) = if_statement.else_clause.as_mut() {
                    Self::substitute_compound_statement(els, name, value)?;
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
                    Self::substitute_expression(arg, name, value)?;
                }
                Self::substitute_expression(&mut switch_statement.expression, name, value)?;
                for clause in switch_statement.clauses.iter_mut() {
                    Self::substitute_compound_statement(&mut clause.body, name, value)?;
                    for case_seletor in clause.case_selectors.iter_mut() {
                        if let CaseSelector::Expression(expr) = case_seletor.as_mut() {
                            Self::substitute_expression(expr, name, value)?;
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
                    Self::substitute_expression(arg, name, value)?;
                }

                Self::substitute_compound_statement(&mut loop_statement.body, name, value)?;

                if let Some(continuing) = loop_statement.continuing.as_mut() {
                    Self::substitute_compound_statement(&mut continuing.body, name, value)?;
                    if let Some(break_if) = continuing.break_if.as_mut() {
                        Self::substitute_expression(break_if, name, value)?;
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
                    Self::substitute_expression(arg, name, value)?;
                }

                Self::substitute_compound_statement(&mut for_statement.body, name, value)?;

                if let Some(cond) = for_statement.condition.as_mut() {
                    Self::substitute_expression(cond, name, value)?;
                }

                if let Some(statement) = for_statement.initializer.as_mut() {
                    Self::substitute_statement(statement, name, value)?;
                }

                if let Some(statement) = for_statement.update.as_mut() {
                    Self::substitute_statement(statement, name, value)?;
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
                    Self::substitute_expression(arg, name, value)?;
                }

                Self::substitute_compound_statement(&mut while_statement.body, name, value)?;
                Self::substitute_expression(&mut while_statement.condition, name, value)?;
                Ok(())
            }
            Statement::Break => Ok(()),
            Statement::Continue => Ok(()),
            Statement::Return(spanned) => {
                if let Some(expr) = spanned.as_mut() {
                    Self::substitute_expression(expr, name, value)?;
                }
                Ok(())
            }
            Statement::Discard => Ok(()),
            Statement::FunctionCall(function_call_expression) => {
                Self::substitute_path(&mut function_call_expression.path, name, value)?;
                for arg in function_call_expression.arguments.iter_mut() {
                    Self::substitute_expression(arg, name, value)?;
                }
                Ok(())
            }
            Statement::ConstAssert(const_assert) => {
                Self::substitute_expression(&mut const_assert.expression, name, value)?;
                Ok(())
            }
            Statement::Declaration(declaration_statement) => {
                Self::substitute_declaration(&mut declaration_statement.declaration, name, value)?;

                for statement in declaration_statement.statements.iter_mut() {
                    Self::substitute_statement(statement, name, value)?;
                }
                Ok(())
            }
        }
    }

    fn substitute_declaration(
        declaration: &mut Declaration,
        name: &String,
        value: &Spanned<TemplateArg>,
    ) -> Result<(), CompilerPassError> {
        for arg in declaration
            .attributes
            .iter_mut()
            .flat_map(|x| x.arguments.iter_mut().flatten())
        {
            Self::substitute_expression(arg, name, value)?;
        }

        if let Some(init) = declaration.initializer.as_mut() {
            Self::substitute_expression(init, name, value)?;
        }
        if let Some(typ) = declaration.typ.as_mut() {
            Self::substitute_path(&mut typ.path, name, value)?;
        }
        Ok(())
    }

    fn substitute_path(
        path: &mut Spanned<Vec<PathPart>>,
        name: &String,
        value: &Spanned<TemplateArg>,
    ) -> Result<(), CompilerPassError> {
        for part in path.iter_mut() {
            if let Some(args) = part.template_args.as_mut() {
                for template_arg in args.iter_mut() {
                    Self::substitute_expression(&mut template_arg.expression, name, value)?;
                }
            }
        }
        let first_name = path.first().unwrap().name.clone();

        if name == &first_name.value {
            if let Ok(mut front) =
                TryInto::<Spanned<Vec<PathPart>>>::try_into(value.expression.value.clone())
            {
                path.remove(0);
                front.append(&mut path.value);
                path.value = front.value;
            }
        }
        Ok(())
    }

    fn match_and_drain(
        template_params: &mut Vec<Spanned<FormalTemplateParameter>>,
        with: PathPart,
    ) -> Vec<(Spanned<FormalTemplateParameter>, Spanned<TemplateArg>)> {
        return template_params
            .drain(..)
            .map(|x| {
                let name: Option<Spanned<String>> = Some(x.name.clone());
                println!("{with}");
                (
                    x,
                    with.template_args
                        .iter()
                        .flatten()
                        .find(|y| y.arg_name == name)
                        .cloned()
                        .unwrap_or_else(|| panic!("EXPECTED {:?}", name)),
                )
            })
            .collect();
    }

    fn specialize_alias(alias: &mut Alias, with: PathPart) -> Result<(), CompilerPassError> {
        for (param, arg) in Self::match_and_drain(&mut alias.template_parameters, with) {
            let name: &String = &param.name.value;
            Self::substitute_path(&mut alias.typ.path, name, &arg)?;
        }
        Ok(())
    }

    fn specialize_declaration(
        declaration: &mut Declaration,
        with: PathPart,
    ) -> Result<(), CompilerPassError> {
        for (param, arg) in Self::match_and_drain(&mut declaration.template_parameters, with) {
            let name = &param.name.value;
            if let Some(typ) = declaration.typ.as_mut() {
                Self::substitute_path(&mut typ.path, name, &arg)?;
            }

            if let Some(init) = declaration.initializer.as_mut() {
                Self::substitute_expression(init, name, &arg)?;
            }

            for expr in declaration
                .attributes
                .iter_mut()
                .flat_map(|x| x.arguments.iter_mut().flatten())
            {
                Self::substitute_expression(expr.as_mut(), name, &arg)?;
            }
        }
        Ok(())
    }

    fn specialize_const_assert(
        const_assert: &mut ConstAssert,
        with: PathPart,
    ) -> Result<(), CompilerPassError> {
        for (param, arg) in Self::match_and_drain(&mut const_assert.template_parameters, with) {
            let name = &param.name.value;
            Self::substitute_expression(&mut const_assert.expression, name, &arg)?;
        }

        Ok(())
    }

    fn specialize_function(
        function: &mut Function,
        with: PathPart,
    ) -> Result<(), CompilerPassError> {
        for (param, arg) in Self::match_and_drain(&mut function.template_parameters, with) {
            let name: &String = &param.name.value;
            Self::substitute_compound_statement(&mut function.body, name, &arg)?;
            for expr in function
                .attributes
                .iter_mut()
                .chain(function.return_attributes.iter_mut())
                .chain(
                    function
                        .parameters
                        .iter_mut()
                        .flat_map(|x| x.attributes.iter_mut()),
                )
                .flat_map(|x| x.arguments.iter_mut().flatten())
            {
                Self::substitute_expression(expr.as_mut(), name, &arg)?;
            }

            if let Some(ret) = function.return_type.as_mut() {
                Self::substitute_path(&mut ret.path, name, &arg)?;
            }

            for func_param in function.parameters.iter_mut() {
                Self::substitute_path(&mut func_param.typ.path, name, &arg)?;
            }
        }

        Ok(())
    }

    fn specialize_struct(strct: &mut Struct, with: PathPart) -> Result<(), CompilerPassError> {
        for (param, arg) in Self::match_and_drain(&mut strct.template_parameters, with) {
            let name: &String = &param.name.value;
            for member in strct.members.iter_mut() {
                Self::substitute_path(&mut member.typ.path, name, &arg)?;
                for expr in member
                    .attributes
                    .iter_mut()
                    .flat_map(|x| x.arguments.iter_mut())
                    .flatten()
                {
                    Self::substitute_expression(expr, name, &arg)?;
                }
            }
        }

        Ok(())
    }

    fn push_down(&mut self) -> Result<(), CompilerPassError> {
        assert!(self.requires_push_down());
        let module = match self {
            OwnedMember::Global(Spanned {
                value: GlobalDeclaration::Module(m),
                ..
            }) => Some(m),
            OwnedMember::Module(Spanned {
                value: ModuleMemberDeclaration::Module(m),
                ..
            }) => Some(m),
            _ => None,
        }
        .unwrap();

        let params: Vec<Spanned<FormalTemplateParameter>> =
            module.template_parameters.drain(..).collect();

        let mut new_members = vec![];
        for mut member in module.members.drain(..) {
            if matches!(&member.value, ModuleMemberDeclaration::Module(_)) {
                let template_params = member.template_parameters_mut().unwrap();
                let mut params = params.clone();
                params.append(template_params);
                *template_params = params;
            } else {
                let borrowed = BorrowedMember::Module {
                    declaration: &mut member,
                    is_initialized: true,
                };
                let mut usages: Usages = Usages::new();
                borrowed.collect_usages(&mut usages)?;
                let usages = usages
                    .set
                    .into_iter()
                    .filter_map(|x| x.head().map(|x| x.name.value.to_string()))
                    .collect::<HashSet<String>>();
                let mut used_params = params
                    .iter()
                    .filter(|x: &&Spanned<FormalTemplateParameter>| usages.contains(&x.name.value))
                    .cloned()
                    .collect::<Vec<Spanned<FormalTemplateParameter>>>();
                if let Some(template_params) = member.template_parameters_mut() {
                    used_params.append(template_params);
                    *template_params = used_params;
                }
            }

            new_members.push(member);
        }
        module.members = new_members;
        Ok(())
    }
}

type SymbolMap = HashMap<SymbolPath, OwnedMember>;

impl<'a> BorrowedMember<'a> {
    fn try_add_alias_usage(
        &self,
        remaining_path: im::Vector<PathPart>,
        usages: &mut Usages,
    ) -> bool {
        match self {
            BorrowedMember::Global {
                declaration:
                    Spanned {
                        value: GlobalDeclaration::Alias(alias),
                        ..
                    },
                ..
            }
            | BorrowedMember::Module {
                declaration:
                    Spanned {
                        value: ModuleMemberDeclaration::Alias(alias),
                        ..
                    },
                ..
            } => {
                // Precondition is that this alias needs to be fully resolved
                assert!(&alias.template_parameters.is_empty());
                let mut path: im::Vector<PathPart> = alias
                    .typ
                    .path
                    .iter()
                    .cloned()
                    .collect::<im::Vector<PathPart>>();
                path.append(remaining_path);
                usages.insert(path);
                true
            }
            _ => false,
        }
    }

    fn collect_usages(&self, usages: &mut Usages) -> Result<(), CompilerPassError> {
        match self {
            BorrowedMember::Global {
                declaration: decl,
                is_initialized: _,
            } => Self::collect_usages_from_global_decl(decl, usages)?,
            BorrowedMember::Module {
                declaration: decl,
                is_initialized: _,
            } => Self::collect_usages_from_module_member_decl(decl, usages)?,
        }
        Ok(())
    }

    fn name(&self) -> Option<Spanned<String>> {
        match self {
            BorrowedMember::Global { declaration, .. } => declaration.name(),
            BorrowedMember::Module { declaration, .. } => declaration.name(),
        }
    }

    fn collect_usages_from_global_decl(
        decl: &GlobalDeclaration,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        match decl {
            GlobalDeclaration::Void => {}
            GlobalDeclaration::Declaration(declaration) => {
                Self::collect_usages_from_declaration(declaration, usages)?
            }
            GlobalDeclaration::Alias(alias) => Self::collect_usages_from_alias(alias, usages)?,
            GlobalDeclaration::Struct(strct) => Self::collect_usages_from_struct(strct, usages)?,
            GlobalDeclaration::Function(function) => {
                Self::collect_usages_from_function(function, usages)?
            }
            GlobalDeclaration::ConstAssert(const_assert) => {
                Self::collect_usages_from_const_assert(const_assert, usages)?
            }
            GlobalDeclaration::Module(module) => {
                // We're not recursing here
                for arg in module
                    .attributes
                    .iter()
                    .flat_map(|x| x.arguments.iter().flatten())
                {
                    Self::collect_usages_from_expression(arg, usages)?;
                }
            }
        }
        Ok(())
    }

    fn collect_usages_from_module_member_decl(
        decl: &ModuleMemberDeclaration,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        match decl {
            ModuleMemberDeclaration::Void => {}
            ModuleMemberDeclaration::Declaration(declaration) => {
                Self::collect_usages_from_declaration(declaration, usages)?
            }
            ModuleMemberDeclaration::Alias(alias) => {
                Self::collect_usages_from_alias(alias, usages)?
            }
            ModuleMemberDeclaration::Struct(strct) => {
                Self::collect_usages_from_struct(strct, usages)?
            }
            ModuleMemberDeclaration::Function(function) => {
                Self::collect_usages_from_function(function, usages)?
            }
            ModuleMemberDeclaration::ConstAssert(const_assert) => {
                Self::collect_usages_from_const_assert(const_assert, usages)?
            }
            ModuleMemberDeclaration::Module(module) => {
                for arg in module
                    .attributes
                    .iter()
                    .flat_map(|x| x.arguments.iter().flatten())
                {
                    Self::collect_usages_from_expression(arg, usages)?;
                }
                // We're not recursing here
            }
        }
        Ok(())
    }

    fn collect_usages_from_struct(
        strct: &Struct,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        for member in strct.members.iter() {
            Self::collect_usages_from_type(&member.typ, usages)?;
        }

        for arg in strct
            .members
            .iter()
            .flat_map(|x| x.attributes.iter())
            .flat_map(|x| x.arguments.iter().flatten())
        {
            Self::collect_usages_from_expression(arg, usages)?;
        }

        Ok(())
    }

    fn collect_usages_from_function(
        function: &Function,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        for attr in function
            .attributes
            .iter()
            .chain(function.return_attributes.iter())
            .chain(
                function
                    .parameters
                    .iter()
                    .flat_map(|x| &x.as_ref().attributes),
            )
        {
            for arg in attr.arguments.iter().flatten() {
                Self::collect_usages_from_expression(arg.as_ref(), usages)?;
            }
        }

        for param in function.parameters.iter() {
            Self::collect_usages_from_type(&param.typ, usages)?;
        }

        if let Some(ret) = function.return_type.as_ref() {
            Self::collect_usages_from_type(ret, usages)?;
        }

        Self::collect_usages_from_compound_statement(&function.body, usages)?;
        Ok(())
    }

    fn collect_usages_from_type(
        typ: &TypeExpression,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        Self::collect_usages_from_path(&typ.path, usages)?;
        Ok(())
    }

    fn collect_usages_from_path(
        path: &[PathPart],
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        for part in path.iter() {
            if let Some(args) = part.template_args.as_ref() {
                for arg in args.iter() {
                    Self::collect_usages_from_expression(&arg.expression, usages)?;
                }
            }
        }
        usages.insert(path.into());
        Ok(())
    }

    fn collect_usages_from_declaration(
        declaration: &Declaration,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        for attribute in declaration.attributes.iter() {
            for arg in attribute.arguments.iter().flatten() {
                Self::collect_usages_from_expression(arg, usages)?;
            }
        }

        if let Some(typ) = declaration.typ.as_ref() {
            Self::collect_usages_from_type(typ, usages)?;
        }

        if let Some(init) = declaration.initializer.as_ref() {
            Self::collect_usages_from_expression(init.as_ref(), usages)?;
        }

        Ok(())
    }

    fn collect_usages_from_alias(
        alias: &Alias,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        Self::collect_usages_from_type(&alias.typ, usages)?;
        Ok(())
    }

    fn collect_usages_from_const_assert(
        const_assert: &ConstAssert,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        Self::collect_usages_from_expression(&const_assert.expression, usages)?;
        Ok(())
    }

    fn collect_usages_from_expression(
        expression: &Expression,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        match expression {
            Expression::Literal(_) => Ok(()),
            Expression::Parenthesized(spanned) => {
                Self::collect_usages_from_expression(spanned, usages)
            }
            Expression::NamedComponent(named_component_expression) => {
                Self::collect_usages_from_expression(&named_component_expression.base, usages)?;
                Ok(())
            }
            Expression::Indexing(indexing_expression) => {
                Self::collect_usages_from_expression(&indexing_expression.base, usages)?;
                Self::collect_usages_from_expression(&indexing_expression.index, usages)?;
                Ok(())
            }
            Expression::Unary(unary_expression) => {
                Self::collect_usages_from_expression(&unary_expression.operand, usages)
            }
            Expression::Binary(binary_expression) => {
                Self::collect_usages_from_expression(&binary_expression.left, usages)?;

                Self::collect_usages_from_expression(&binary_expression.right, usages)?;
                Ok(())
            }
            Expression::FunctionCall(function_call_expression) => {
                Self::collect_usages_from_path(&function_call_expression.path, usages)?;
                for arg in function_call_expression.arguments.iter() {
                    Self::collect_usages_from_expression(arg, usages)?;
                }
                Ok(())
            }
            Expression::Identifier(IdentifierExpression { path })
            | Expression::Type(TypeExpression { path }) => {
                Self::collect_usages_from_path(path, usages)
            }
        }
    }

    fn collect_usages_from_statement(
        statement: &Statement,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        match statement {
            Statement::Void => Ok(()),
            Statement::Compound(compound_statement) => {
                Self::collect_usages_from_compound_statement(compound_statement, usages)
            }
            Statement::Assignment(assignment_statement) => {
                Self::collect_usages_from_expression(&assignment_statement.lhs, usages)?;
                Self::collect_usages_from_expression(&assignment_statement.rhs, usages)?;
                Ok(())
            }
            Statement::Increment(expression) => {
                Self::collect_usages_from_expression(expression, usages)
            }
            Statement::Decrement(expression) => {
                Self::collect_usages_from_expression(expression, usages)
            }
            Statement::If(if_statement) => {
                for expr in if_statement
                    .attributes
                    .iter()
                    .flat_map(|x| x.arguments.iter().flatten())
                {
                    Self::collect_usages_from_expression(expr.as_ref(), usages)?;
                }

                Self::collect_usages_from_compound_statement(&if_statement.if_clause.1, usages)?;
                Self::collect_usages_from_expression(&if_statement.if_clause.0, usages)?;

                for elif in if_statement.else_if_clauses.iter() {
                    Self::collect_usages_from_compound_statement(&elif.1, usages)?;
                    Self::collect_usages_from_expression(&elif.0, usages)?;
                }

                if let Some(els) = if_statement.else_clause.as_ref() {
                    Self::collect_usages_from_compound_statement(els, usages)?;
                }

                Ok(())
            }
            Statement::Switch(switch_statement) => {
                for arg in switch_statement
                    .attributes
                    .iter()
                    .chain(switch_statement.body_attributes.iter())
                    .flat_map(|x| x.arguments.iter().flatten())
                {
                    Self::collect_usages_from_expression(arg, usages)?;
                }
                Self::collect_usages_from_expression(&switch_statement.expression, usages)?;
                for clause in switch_statement.clauses.iter() {
                    Self::collect_usages_from_compound_statement(&clause.body, usages)?;
                    for case_seletor in clause.case_selectors.iter() {
                        if let CaseSelector::Expression(expr) = case_seletor.as_ref() {
                            Self::collect_usages_from_expression(expr, usages)?;
                        }
                    }
                }
                Ok(())
            }
            Statement::Loop(loop_statement) => {
                for arg in loop_statement
                    .attributes
                    .iter()
                    .flat_map(|x| x.arguments.iter().flatten())
                {
                    Self::collect_usages_from_expression(arg, usages)?;
                }

                Self::collect_usages_from_compound_statement(&loop_statement.body, usages)?;

                if let Some(continuing) = loop_statement.continuing.as_ref() {
                    Self::collect_usages_from_compound_statement(&continuing.body, usages)?;
                    if let Some(break_if) = continuing.break_if.as_ref() {
                        Self::collect_usages_from_expression(break_if, usages)?;
                    }
                }
                Ok(())
            }
            Statement::For(for_statement) => {
                for arg in for_statement
                    .attributes
                    .iter()
                    .flat_map(|x| x.arguments.iter().flatten())
                {
                    Self::collect_usages_from_expression(arg, usages)?;
                }

                Self::collect_usages_from_compound_statement(&for_statement.body, usages)?;

                if let Some(cond) = for_statement.condition.as_ref() {
                    Self::collect_usages_from_expression(cond, usages)?;
                }

                if let Some(statement) = for_statement.initializer.as_ref() {
                    Self::collect_usages_from_statement(statement, usages)?;
                }

                if let Some(statement) = for_statement.update.as_ref() {
                    Self::collect_usages_from_statement(statement, usages)?;
                }

                Ok(())
            }
            Statement::While(while_statement) => {
                for arg in while_statement
                    .attributes
                    .iter()
                    .flat_map(|x| x.arguments.iter().flatten())
                {
                    Self::collect_usages_from_expression(arg, usages)?;
                }

                Self::collect_usages_from_compound_statement(&while_statement.body, usages)?;
                Self::collect_usages_from_expression(&while_statement.condition, usages)?;
                Ok(())
            }
            Statement::Break => Ok(()),
            Statement::Continue => Ok(()),
            Statement::Return(spanned) => {
                if let Some(expr) = spanned.as_ref() {
                    Self::collect_usages_from_expression(expr, usages)
                } else {
                    Ok(())
                }
            }
            Statement::Discard => Ok(()),
            Statement::FunctionCall(function_call_expression) => {
                Self::collect_usages_from_path(&function_call_expression.path, usages)?;
                for arg in function_call_expression.arguments.iter() {
                    Self::collect_usages_from_expression(arg.as_ref(), usages)?;
                }
                Ok(())
            }
            Statement::ConstAssert(const_assert) => {
                Self::collect_usages_from_const_assert(const_assert, usages)
            }
            Statement::Declaration(declaration_statement) => {
                Self::collect_usages_from_declaration(&declaration_statement.declaration, usages)?;
                for statement in declaration_statement.statements.iter() {
                    Self::collect_usages_from_statement(statement, usages)?;
                }
                Ok(())
            }
        }
    }

    fn collect_usages_from_compound_statement(
        compound_statement: &CompoundStatement,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        for attribute in compound_statement.attributes.iter() {
            for arg in attribute.arguments.iter().flatten() {
                Self::collect_usages_from_expression(arg, usages)?;
            }
        }
        for statement in compound_statement.statements.iter() {
            Self::collect_usages_from_statement(statement.as_ref(), usages)?;
        }
        Ok(())
    }
}

impl From<OwnedMember> for Spanned<GlobalDeclaration> {
    fn from(val: OwnedMember) -> Self {
        match val {
            OwnedMember::Global(spanned) => spanned,
            OwnedMember::Module(Spanned { value, span }) => Spanned::new(value.into(), span),
        }
    }
}

impl From<OwnedMember> for Spanned<ModuleMemberDeclaration> {
    fn from(val: OwnedMember) -> Self {
        match val {
            OwnedMember::Module(spanned) => spanned,
            OwnedMember::Global(Spanned { value, span }) => Spanned::new(value.into(), span),
        }
    }
}
#[derive(Debug)]
enum Parent<'a> {
    TranslationUnit(&'a mut TranslationUnit),
    Module {
        module: &'a mut Module,
        is_initialized: bool,
    },
}

impl<'a> BorrowedMember<'a> {
    fn set_initialized(&mut self) {
        match self {
            BorrowedMember::Global {
                declaration: _,
                is_initialized,
            } => {
                *is_initialized = true;
            }
            BorrowedMember::Module {
                declaration: _,
                is_initialized,
            } => {
                *is_initialized = true;
            }
        }
    }

    fn try_into_parent(self) -> Result<Parent<'a>, BorrowedMember<'a>> {
        match self {
            BorrowedMember::Global {
                declaration:
                    Spanned {
                        span: _,
                        value: GlobalDeclaration::Module(m),
                    },
                is_initialized,
            } => Ok(Parent::Module {
                module: m,
                is_initialized,
            }),
            BorrowedMember::Module {
                declaration:
                    Spanned {
                        span: _,
                        value: ModuleMemberDeclaration::Module(m),
                    },
                is_initialized,
            } => Ok(Parent::Module {
                module: m,
                is_initialized,
            }),
            other => Err(other),
        }
    }
}

impl<'a> From<&'a mut TranslationUnit> for Parent<'a> {
    fn from(value: &'a mut TranslationUnit) -> Self {
        Self::TranslationUnit(value)
    }
}

impl<'a> Parent<'a> {
    fn is_initialized(&self) -> bool {
        match self {
            Parent::TranslationUnit(_) => true,
            Parent::Module {
                module: _,
                is_initialized,
            } => *is_initialized,
        }
    }

    fn add_member(&mut self, member: OwnedMember) -> BorrowedMember<'_> {
        match self {
            Parent::TranslationUnit(t) => {
                t.global_declarations.push(member.into());
                BorrowedMember::Global {
                    declaration: t.global_declarations.last_mut().unwrap(),
                    is_initialized: false,
                }
            }
            Parent::Module {
                module: m,
                is_initialized,
            } => {
                assert!(*is_initialized);
                m.members.push(member.into());
                BorrowedMember::Module {
                    declaration: m.members.last_mut().unwrap(),
                    is_initialized: false,
                }
            }
        }
    }

    fn add_alias(&mut self, path_part: &PathPart, concrete_path: ConcreteSymbolPath) {
        let _ = self.add_member(OwnedMember::Global(Spanned::new(
            GlobalDeclaration::Alias(Self::make_alias(
                path_part,
                concrete_path,
                path_part.name.span(),
            )),
            path_part.name.span(),
        )));
    }

    fn find_child<'b>(&'b mut self, path_part: &PathPart) -> Option<BorrowedMember<'b>> {
        let name = mangle_template_args(path_part);
        match self {
            Parent::TranslationUnit(x) => {
                for item in x.global_declarations.iter_mut() {
                    if matches!(item.name(), Some(n) if n.value == name) {
                        assert!(item.template_parameters().is_none());
                        return Some(BorrowedMember::Global {
                            declaration: item,
                            is_initialized: true,
                        });
                    }
                }
                None
            }
            Parent::Module {
                module,
                is_initialized,
            } => {
                assert!(*is_initialized);

                for item in module.members.iter_mut() {
                    if matches!(item.name(), Some(n) if n.value == name) {
                        assert!(item.template_parameters().is_none());
                        return Some(BorrowedMember::Module {
                            declaration: item,
                            is_initialized: true,
                        });
                    }
                }
                None
            }
        }
    }

    fn remove_child(&mut self, path_part: &PathPart) {
        let name = mangle_template_args(path_part);
        match self {
            Parent::TranslationUnit(x) => {
                x.global_declarations
                    .retain(|x| !matches!(x.name(), Some(n) if n.value == name));
            }
            Parent::Module {
                module,
                is_initialized,
            } => {
                assert!(*is_initialized);
                module
                    .members
                    .retain(|x: &Spanned<ModuleMemberDeclaration>| !matches!(x.name(), Some(n) if n.value == name));
            }
        }
    }

    fn is_entry_point(function: &Function) -> bool {
        function
            .attributes
            .iter()
            .any(|x| matches!(x.name.as_ref().as_str(), "vertex" | "fragment" | "compute"))
    }

    fn add_module(&mut self, path_part: PathPart) -> BorrowedMember<'_> {
        let module = Module {
            name: Spanned::new(mangle_template_args(&path_part), path_part.name.span()),
            ..Default::default()
        };

        let mut borrowed = self.add_member(OwnedMember::Global(Spanned::new(
            GlobalDeclaration::Module(module),
            path_part.name.span(),
        )));

        borrowed.set_initialized();

        borrowed
    }

    fn make_alias(path_part: &PathPart, concrete_path: ConcreteSymbolPath, span: Span) -> Alias {
        let path = concrete_path
            .into_iter()
            .map(|x| PathPart {
                name: Spanned::new(x, 0..0),
                template_args: None,
                inline_template_args: None,
            })
            .collect();
        Alias {
            name: Spanned::new(mangle_template_args(path_part), path_part.name.span()),
            typ: Spanned::new(
                TypeExpression {
                    path: Spanned::new(path, span.clone()),
                },
                span.clone(),
            ),
            template_parameters: vec![],
        }
    }

    fn initialize(
        &mut self,
        symbol_path: ConcreteSymbolPath,
        symbol_map: &mut SymbolMap,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        match self {
            Parent::TranslationUnit(t) => {
                let mut entrypoints = vec![];
                for declaration in t.global_declarations.drain(..) {
                    if let GlobalDeclaration::Function(f) = declaration.as_ref() {
                        if Self::is_entry_point(f) && f.template_parameters.is_empty() {
                            entrypoints.push(declaration);
                            continue;
                        }
                    } else if let GlobalDeclaration::ConstAssert(assrt) = declaration.as_ref() {
                        if assrt.template_parameters.is_empty() {
                            entrypoints.push(declaration);
                            continue;
                        }
                    } else if let GlobalDeclaration::Alias(alias) = declaration.as_ref() {
                        if alias.template_parameters.is_empty() {
                            entrypoints.push(declaration);
                            continue;
                        }
                    }
                    if let Some(name) = declaration.name() {
                        let mut symbol_path = symbol_path.clone();
                        symbol_path.push_back(name.value);
                        symbol_map.insert(symbol_path, OwnedMember::Global(declaration));
                    } else {
                        entrypoints.push(declaration);
                    }
                }
                t.global_declarations.append(&mut entrypoints);

                for member in t.global_declarations.iter_mut() {
                    let member: BorrowedMember<'_> = BorrowedMember::Global {
                        declaration: member,
                        is_initialized: true,
                    };
                    member.collect_usages(usages)?;
                }

                Ok(())
            }
            Parent::Module {
                module,
                is_initialized,
            } if !*is_initialized => {
                let mut others = vec![];
                for mut declaration in module.members.drain(..) {
                    if let Some(name) = declaration.name() {
                        let mut symbol_path: im::Vector<String> = symbol_path.clone();
                        symbol_path.push_back(name.value.clone());
                        symbol_map.insert(symbol_path, OwnedMember::Module(declaration));
                    } else {
                        let member: BorrowedMember<'_> = BorrowedMember::Module {
                            declaration: &mut declaration,
                            is_initialized: true,
                        };
                        member.collect_usages(usages)?;
                        others.push(declaration);
                    }
                }
                module.members.append(&mut others);

                *is_initialized = true;
                Ok(())
            }
            _ => Ok(()),
        }?;
        Ok(())
    }
}
type SymbolPath = im::Vector<String>;

impl Specializer {
    fn specialize_translation_unit<'a>(
        &self,
        translation_unit: &'a mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let mut symbol_map: SymbolMap = HashMap::new();
        let mut usages: Usages = Usages::new();
        if let Some(entrypoint) = self.entrypoint.as_ref() {
            usages.insert(entrypoint.iter().cloned().collect());
        }
        let mut parent: Parent<'a> = Parent::TranslationUnit(translation_unit);
        parent.initialize(im::Vector::new(), &mut symbol_map, &mut usages)?;

        while let Some(remaining_path) = usages.pop() {
            assert!(!remaining_path.is_empty());
            let current_path = im::Vector::new();
            if let Some(concrete_path) = Self::specialize(
                &mut parent,
                &mut usages,
                &mut symbol_map,
                remaining_path.clone(),
                current_path,
            )? {
                Self::alias(&mut parent, remaining_path, concrete_path)?;
            }
        }
        Ok(())
    }

    fn alias<'a, 'b: 'a>(
        parent: &'a mut Parent<'b>,
        mut remaining_path: im::Vector<PathPart>,
        concrete_path: ConcreteSymbolPath,
    ) -> Result<(), CompilerPassError> {
        assert!(!remaining_path.is_empty());
        let part: PathPart = remaining_path.pop_front().unwrap();
        let current: BorrowedMember<'_>;

        if let Some(m) = parent.find_child(&part) {
            if remaining_path.is_empty() {
                return Ok(());
            } else {
                current = m;
            }
        } else if remaining_path.is_empty() {
            parent.add_alias(&part, concrete_path);
            return Ok(());
        } else {
            current = parent.add_module(part);
        }

        match current.try_into_parent() {
            Ok(mut p) => Self::alias(&mut p, remaining_path, concrete_path),
            Err(_) => Ok(()),
        }
    }

    fn specialize<'a, 'b: 'a>(
        parent: &'a mut Parent<'b>,
        usages: &mut Usages,
        symbol_map: &mut SymbolMap,
        mut remaining_path: im::Vector<PathPart>,
        mut current_path: ConcreteSymbolPath,
    ) -> Result<Option<ConcreteSymbolPath>, CompilerPassError> {
        assert!(parent.is_initialized());
        if remaining_path.is_empty() {
            return Ok(None);
        }
        let mut part: PathPart = remaining_path.pop_front().unwrap();
        let current;
        let mut unparamaterized_part = part.clone();
        unparamaterized_part.template_args = None;

        let mut symbol_path = current_path.clone();
        symbol_path.push_back(part.name.value.clone());

        if let Some(mut member) = symbol_map.remove(&symbol_path) {
            if member.requires_push_down() {
                member.push_down()?;
            } else if member.requires_specialization() {
                // Add back symbol so we can specialize again
                symbol_map.insert(symbol_path, member.clone());
                member.specialize(part.clone())?;
            }
            current = parent.add_member(member);
        } else if let Some(m) = parent.find_child(&unparamaterized_part) {
            current = m;
        } else {
            return Ok(None);
        }

        if let Some(mut head) = remaining_path.pop_front() {
            let mut parent_args = part.template_args.take().unwrap_or_default();
            if let Some(mut args) = head.template_args.take() {
                parent_args.append(&mut args);
            }
            if !parent_args.is_empty() {
                head.template_args = Some(parent_args);
            }
            remaining_path.push_front(head);
        } else {
            current.collect_usages(usages)?;
        }
        current_path.push_back(current.name().unwrap().value);

        match current.try_into_parent() {
            Ok(mut p) => {
                p.initialize(current_path.clone(), symbol_map, usages)?;
                Self::specialize(&mut p, usages, symbol_map, remaining_path, current_path)
            }
            Err(borrowed) => {
                if borrowed.try_add_alias_usage(remaining_path.clone(), usages)
                    || remaining_path.is_empty()
                {
                    return Ok(Some(current_path));
                }
                return Err(CompilerPassError::UnableToResolvePath(
                    current_path
                        .iter()
                        .cloned()
                        .map(|x| PathPart {
                            name: Spanned::new(x, 0..0),
                            template_args: None,
                            inline_template_args: None,
                        })
                        .take(current_path.len() - 1)
                        .chain(remaining_path.clone())
                        .collect(),
                ));
            }
        }
    }
}

impl CompilerPass for Specializer {
    fn apply_mut(
        &mut self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        self.specialize_translation_unit(translation_unit)?;
        Ok(())
    }
}
