use std::{collections::VecDeque, hash::Hash};

use im::{HashMap, HashSet};
use wesl_parse::{span::Spanned, syntax::*};
use wesl_types::{CompilerPass, CompilerPassError};

#[derive(Debug, Default, Clone, Copy)]
struct Specializer;

type ConcreteSymbolPath = im::Vector<PathPart>;
type AliasCache = HashMap<ConcreteSymbolPath, ConcreteSymbolPath>;

#[derive(Debug, Default, Clone)]
struct Usages {
    set: HashSet<ConcreteSymbolPath>,
    queue: VecDeque<ConcreteSymbolPath>,
}

impl Usages {
    fn new() -> Usages {
        Default::default()
    }

    fn insert(&mut self, path: ConcreteSymbolPath) {
        if self.set.insert(path.clone()).is_none() {
            self.queue.push_back(path);
        }
    }

    fn pop(&mut self) -> Option<ConcreteSymbolPath> {
        self.queue.pop_front()
    }

    fn front(&mut self) -> Option<ConcreteSymbolPath> {
        self.queue.front().cloned()
    }
}

type SymbolDeclarations = HashMap<SymbolPath, Member>;

#[derive(Debug, Clone, PartialEq, Hash)]
enum Member {
    Global(Spanned<GlobalDeclaration>),
    Module(Spanned<ModuleMemberDeclaration>),
}

impl Into<Spanned<GlobalDeclaration>> for Member {
    fn into(self) -> Spanned<GlobalDeclaration> {
        match self {
            Member::Global(spanned) => spanned,
            Member::Module(Spanned { value, span }) => Spanned::new(value.into(), span),
        }
    }
}

impl Into<Spanned<ModuleMemberDeclaration>> for Member {
    fn into(self) -> Spanned<ModuleMemberDeclaration> {
        match self {
            Member::Module(spanned) => spanned,
            Member::Global(Spanned { value, span }) => Spanned::new(value.into(), span),
        }
    }
}

enum Parent<'a> {
    TranslationUnit(&'a mut TranslationUnit),
    Module(&'a mut Module),
}

impl<'a> Parent<'a> {
    fn add_member(&'a mut self, member: Member) {
        match self {
            Parent::TranslationUnit(t) => {
                t.global_declarations.push(member.into());
            }
            Parent::Module(m) => {
                m.members.push(member.into());
            }
        }
    }

    fn find_child(&mut self, name: &str) -> Option<&'a mut Member> {
        match self {
            Parent::TranslationUnit(x) => None,
            Parent::Module(x) => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SymbolPath {
    parent: ConcreteSymbolPath,
    name: Spanned<String>,
}

impl Specializer {
    fn collect_usages_from_struct(
        strct: &Struct,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        assert!(strct.template_parameters.is_empty());
        for member in strct.members.iter() {
            Self::collect_usages_from_typ(&member.typ, usages, alias_cache, template_declarations)?;
        }
        for arg in strct
            .members
            .iter()
            .map(|x| x.attributes.iter())
            .flatten()
            .map(|x| x.arguments.iter().flatten())
            .flatten()
        {
            Self::collect_usages_from_expression(arg, usages, alias_cache, template_declarations)?;
        }
        Ok(())
    }

    fn collect_usages_from_function(
        function: &Function,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        assert!(function.template_parameters.is_empty());
        for attr in function
            .attributes
            .iter()
            .chain(function.return_attributes.iter())
            .chain(
                function
                    .parameters
                    .iter()
                    .map(|x| &x.as_ref().attributes)
                    .flatten(),
            )
        {
            for arg in attr.arguments.iter().flatten() {
                Self::collect_usages_from_expression(
                    &arg.as_ref(),
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
            }
        }
        for param in function.parameters.iter() {
            Self::collect_usages_from_typ(&param.typ, usages, alias_cache, template_declarations)?;
        }

        Self::collect_usages_from_compound_statement(
            &function.body,
            usages,
            alias_cache,
            template_declarations,
        )?;
        Ok(())
    }

    fn collect_usages_from_typ(
        typ: &TypeExpression,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        Self::collect_usages_from_path(&typ.path, usages, alias_cache, template_declarations)?;
        Ok(())
    }

    fn collect_usages_from_path(
        path: &Vec<PathPart>,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        for part in path.iter() {
            for arg in part.template_args.iter().flatten() {
                Self::collect_usages_from_expression(
                    &arg.expression,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
            }
        }
        usages.insert(path.clone().into());
        Ok(())
    }

    fn collect_usages_from_declaration(
        declaration: &Declaration,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        assert!(declaration.template_parameters.is_empty());
        for attribute in declaration.attributes.iter() {
            for arg in attribute.arguments.iter().flatten() {
                Self::collect_usages_from_expression(
                    arg,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
            }
        }

        if let Some(typ) = declaration.typ.as_ref() {
            Self::collect_usages_from_typ(typ, usages, alias_cache, template_declarations)?;
        }

        if let Some(init) = declaration.initializer.as_ref() {
            Self::collect_usages_from_expression(
                init.as_ref(),
                usages,
                alias_cache,
                template_declarations,
            )?;
        }
        Ok(())
    }

    fn collect_usages_from_alias(
        alias: &Alias,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        assert!(alias.template_parameters.is_empty());
        Self::collect_usages_from_typ(&alias.typ, usages, alias_cache, template_declarations)?;
        Ok(())
    }

    fn collect_usages_from_const_assert(
        const_assert: &ConstAssert,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        assert!(const_assert.template_parameters.is_empty());
        Self::collect_usages_from_expression(
            &const_assert.expression,
            usages,
            alias_cache,
            template_declarations,
        )?;
        Ok(())
    }

    fn collect_usages_from_expression(
        expression: &Expression,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        match expression {
            Expression::Literal(_) => Ok(()),
            Expression::Parenthesized(spanned) => Self::collect_usages_from_expression(
                &spanned,
                usages,
                alias_cache,
                template_declarations,
            ),
            Expression::NamedComponent(named_component_expression) => {
                Self::collect_usages_from_expression(
                    &named_component_expression.base,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Ok(())
            }
            Expression::Indexing(indexing_expression) => {
                Self::collect_usages_from_expression(
                    &indexing_expression.base,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Self::collect_usages_from_expression(
                    &indexing_expression.index,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Ok(())
            }
            Expression::Unary(unary_expression) => Self::collect_usages_from_expression(
                &unary_expression.operand,
                usages,
                alias_cache,
                template_declarations,
            ),
            Expression::Binary(binary_expression) => {
                Self::collect_usages_from_expression(
                    &binary_expression.left,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;

                Self::collect_usages_from_expression(
                    &binary_expression.right,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Ok(())
            }
            Expression::FunctionCall(function_call_expression) => {
                Self::collect_usages_from_path(
                    &function_call_expression.path,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                for arg in function_call_expression.arguments.iter() {
                    Self::collect_usages_from_expression(
                        &arg,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }
                Ok(())
            }
            Expression::Identifier(identifier_expression) => Self::collect_usages_from_path(
                &identifier_expression.path,
                usages,
                alias_cache,
                template_declarations,
            ),
            Expression::Type(type_expression) => Self::collect_usages_from_typ(
                type_expression,
                usages,
                alias_cache,
                template_declarations,
            ),
        }
    }

    fn collect_usages_from_statement(
        statement: &Statement,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        match statement {
            Statement::Void => Ok(()),
            Statement::Compound(compound_statement) => {
                Self::collect_usages_from_compound_statement(
                    compound_statement,
                    usages,
                    alias_cache,
                    template_declarations,
                )
            }
            Statement::Assignment(assignment_statement) => {
                Self::collect_usages_from_expression(
                    &assignment_statement.lhs,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Self::collect_usages_from_expression(
                    &assignment_statement.rhs,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Ok(())
            }
            Statement::Increment(expression) => Self::collect_usages_from_expression(
                expression,
                usages,
                alias_cache,
                template_declarations,
            ),
            Statement::Decrement(expression) => Self::collect_usages_from_expression(
                expression,
                usages,
                alias_cache,
                template_declarations,
            ),
            Statement::If(if_statement) => {
                for expr in if_statement
                    .attributes
                    .iter()
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(
                        expr.as_ref(),
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                Self::collect_usages_from_compound_statement(
                    &if_statement.if_clause.1,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Self::collect_usages_from_expression(
                    &if_statement.if_clause.0,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;

                for elif in if_statement.else_if_clauses.iter() {
                    Self::collect_usages_from_compound_statement(
                        &elif.1,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                    Self::collect_usages_from_expression(
                        &elif.0,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                if let Some(els) = if_statement.else_clause.as_ref() {
                    Self::collect_usages_from_compound_statement(
                        &els,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                Ok(())
            }
            Statement::Switch(switch_statement) => {
                for arg in switch_statement
                    .attributes
                    .iter()
                    .chain(switch_statement.body_attributes.iter())
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(
                        &arg,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }
                Self::collect_usages_from_expression(
                    &switch_statement.expression,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                for clause in switch_statement.clauses.iter() {
                    Self::collect_usages_from_compound_statement(
                        &clause.body,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                    for case_seletor in clause.case_selectors.iter() {
                        if let CaseSelector::Expression(expr) = case_seletor.as_ref() {
                            Self::collect_usages_from_expression(
                                expr,
                                usages,
                                alias_cache,
                                template_declarations,
                            )?;
                        }
                    }
                }
                Ok(())
            }
            Statement::Loop(loop_statement) => {
                for arg in loop_statement
                    .attributes
                    .iter()
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(
                        &arg,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                Self::collect_usages_from_compound_statement(
                    &loop_statement.body,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;

                if let Some(continuing) = loop_statement.continuing.as_ref() {
                    Self::collect_usages_from_compound_statement(
                        &continuing.body,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                    if let Some(break_if) = continuing.break_if.as_ref() {
                        Self::collect_usages_from_expression(
                            &break_if,
                            usages,
                            alias_cache,
                            template_declarations,
                        )?;
                    }
                }
                Ok(())
            }
            Statement::For(for_statement) => {
                for arg in for_statement
                    .attributes
                    .iter()
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(
                        &arg,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                Self::collect_usages_from_compound_statement(
                    &for_statement.body,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;

                if let Some(cond) = for_statement.condition.as_ref() {
                    Self::collect_usages_from_expression(
                        cond,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                if let Some(statement) = for_statement.initializer.as_ref() {
                    Self::collect_usages_from_statement(
                        statement,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                if let Some(statement) = for_statement.update.as_ref() {
                    Self::collect_usages_from_statement(
                        statement,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                Ok(())
            }
            Statement::While(while_statement) => {
                for arg in while_statement
                    .attributes
                    .iter()
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(
                        &arg,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }

                Self::collect_usages_from_compound_statement(
                    &while_statement.body,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Self::collect_usages_from_expression(
                    &while_statement.condition,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                Ok(())
            }
            Statement::Break => Ok(()),
            Statement::Continue => Ok(()),
            Statement::Return(spanned) => {
                if let Some(expr) = spanned.as_ref() {
                    Self::collect_usages_from_expression(
                        expr,
                        usages,
                        alias_cache,
                        template_declarations,
                    )
                } else {
                    Ok(())
                }
            }
            Statement::Discard => Ok(()),
            Statement::FunctionCall(function_call_expression) => {
                Self::collect_usages_from_path(
                    &function_call_expression.path,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                for arg in function_call_expression.arguments.iter() {
                    Self::collect_usages_from_expression(
                        arg.as_ref(),
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }
                Ok(())
            }
            Statement::ConstAssert(const_assert) => Self::collect_usages_from_const_assert(
                const_assert,
                usages,
                alias_cache,
                template_declarations,
            ),
            Statement::Declaration(declaration_statement) => {
                Self::collect_usages_from_declaration(
                    &declaration_statement.declaration,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
                for statement in declaration_statement.statements.iter() {
                    Self::collect_usages_from_statement(
                        statement,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }
                Ok(())
            }
        }
    }

    fn collect_usages_from_compound_statement(
        compound_statement: &CompoundStatement,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        assert!(compound_statement.directives.is_empty());
        for attribute in compound_statement.attributes.iter() {
            for arg in attribute.arguments.iter().flatten() {
                Self::collect_usages_from_expression(
                    arg,
                    usages,
                    alias_cache,
                    template_declarations,
                )?;
            }
        }
        for statement in compound_statement.statements.iter() {
            Self::collect_usages_from_statement(
                statement.as_ref(),
                usages,
                alias_cache,
                template_declarations,
            )?;
        }
        Ok(())
    }

    fn collect_usages_from_translation_unit(
        translation_unit: &mut TranslationUnit,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
        template_declarations: &mut SymbolDeclarations,
    ) -> Result<(), CompilerPassError> {
        for global_decl in translation_unit.global_declarations.iter() {
            match global_decl.as_ref() {
                GlobalDeclaration::Function(function) => {
                    Self::collect_usages_from_function(
                        function,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }
                GlobalDeclaration::ConstAssert(const_assert) => {
                    Self::collect_usages_from_const_assert(
                        const_assert,
                        usages,
                        alias_cache,
                        template_declarations,
                    )?;
                }
                _ => {
                    panic!("INVARIANT FAILURE NO OTHER ELEMENTS SHOULD BE PRESENT AT THIS POINT OF THE ALGORITHM");
                }
            }
        }
        Ok(())
    }

    fn is_entry_point(function: &Function) -> bool {
        function
            .attributes
            .iter()
            .any(|x| match x.name.as_ref().as_str() {
                "vertex" | "fragment" | "compute" => true,
                _ => false,
            })
    }

    fn specialize_translation_unit(
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let mut symbol_map = HashMap::new();
        let mut entrypoints: Vec<Spanned<GlobalDeclaration>> = vec![];

        let symbol_path = ConcreteSymbolPath::new();

        for declaration in translation_unit.global_declarations.drain(..) {
            if let GlobalDeclaration::Function(f) = declaration.as_ref() {
                if Self::is_entry_point(f) && f.template_parameters.is_empty() {
                    entrypoints.push(declaration);
                    continue;
                }
            } else if let GlobalDeclaration::ConstAssert(_) = declaration.as_ref() {
                entrypoints.push(declaration);
                continue;
            }
            symbol_map.insert(
                SymbolPath {
                    parent: symbol_path.clone(),
                    name: declaration.name().unwrap(),
                },
                Member::Global(declaration),
            );
        }

        translation_unit
            .global_declarations
            .append(&mut entrypoints);

        let mut usages = Usages::new();
        let mut alias_cache = AliasCache::new();
        Self::collect_usages_from_translation_unit(
            translation_unit,
            &mut usages,
            &mut alias_cache,
            &mut symbol_map,
        )?;
        while let Some(front) = usages.pop() {
            assert!(front.len() > 0);
            let mut parent = Parent::TranslationUnit(translation_unit);
            let last = front.last().unwrap().clone();
            let mut path = im::Vector::new();
            'outer: for part in front.take(front.len() - 1) {
                if let Some(m) = parent.find_child(&part.name) {
                } else if let Some(mut m) = symbol_map.remove(&SymbolPath {
                    parent: path.clone(),
                    name: part.name.clone(),
                }) {
                } else {
                    panic!("THIS CASE SHOULD NEVER BE REACHED. SYMBOLS SHOULD HAVE ALREADY BEEN CHECKED");
                }
                match parent {
                    Parent::TranslationUnit(_) => panic!("THIS CASE SHOULD NEVER BE REACHED"),
                    Parent::Module(_) => {}
                }
                path.push_back(part.clone());
            }
        }

        Ok(())
    }
}

impl CompilerPass for Specializer {
    fn apply_mut(
        &mut self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        Self::specialize_translation_unit(translation_unit)?;
        Ok(())
    }
}

// Perform the following using the translation unit
// Remove global declarations (excluding entry points) and add to symbol map
// For each entry point add usages of symbols to usages set.

// For each unique, unprocessed usage, create a new symbol path and for each path part  in the usage path do the following:
// add symbol name to symbol path
// using symbol path look up in symbol map
// If symbol has no template parameters add declaration to parent module or translation unit
// Else specialise symbol using the template arguments in the path part.
// Add symbol and its associated productions to parent module or translation unit
// If module, remove module member declarations excluding entry points and aliases from the translation unit  and add to symbol map
// Once the leaf of the path has been reached, add usages found in leaf to usages set
// The problem is @Mathis that unused generics need to be dropped as is.
// Most of my passes do have dependencies on other passes. Mainly name resolution
// But the passes are quite granular in a sense
// E.g. templates has two different passes
