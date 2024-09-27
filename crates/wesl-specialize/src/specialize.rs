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

type SymbolDeclarations = HashMap<SymbolPath, OwnedMember>;

#[derive(Debug, Clone, PartialEq, Hash)]
enum OwnedMember {
    Global(Spanned<GlobalDeclaration>),
    Module(Spanned<ModuleMemberDeclaration>),
}

#[derive(Debug, PartialEq, Hash)]
enum BorrowedMember<'a> {
    Global {
        global: &'a mut Spanned<GlobalDeclaration>,
        is_initialized: bool,
    },
    Module {
        module: &'a mut Spanned<ModuleMemberDeclaration>,
        is_initialized: bool,
    },
}

type SymbolMap = HashMap<SymbolPath, OwnedMember>;

impl<'a> BorrowedMember<'a> {
    fn collect_usages(&self, usages: &mut Usages) -> Result<(), CompilerPassError> {
        match self {
            BorrowedMember::Global {
                global: decl,
                is_initialized: _,
            } => Self::collect_usages_from_global_decl(decl, usages)?,
            BorrowedMember::Module {
                module: decl,
                is_initialized: _,
            } => Self::collect_usages_from_module_member_decl(decl, usages)?,
        }
        Ok(())
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
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(&arg, usages)?;
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
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(&arg, usages)?;
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
        assert!(strct.template_parameters.is_empty());
        for member in strct.members.iter() {
            Self::collect_usages_from_typ(&member.typ, usages)?;
        }
        for arg in strct
            .members
            .iter()
            .map(|x| x.attributes.iter())
            .flatten()
            .map(|x| x.arguments.iter().flatten())
            .flatten()
        {
            Self::collect_usages_from_expression(arg, usages)?;
        }
        Ok(())
    }

    fn collect_usages_from_function(
        function: &Function,
        usages: &mut Usages,
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
                Self::collect_usages_from_expression(&arg.as_ref(), usages)?;
            }
        }
        for param in function.parameters.iter() {
            Self::collect_usages_from_typ(&param.typ, usages)?;
        }

        Self::collect_usages_from_compound_statement(&function.body, usages)?;
        Ok(())
    }

    fn collect_usages_from_typ(
        typ: &TypeExpression,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        Self::collect_usages_from_path(&typ.path, usages)?;
        Ok(())
    }

    fn collect_usages_from_path(
        path: &Vec<PathPart>,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        for part in path.iter() {
            for arg in part.template_args.iter().flatten() {
                Self::collect_usages_from_expression(&arg.expression, usages)?;
            }
        }
        usages.insert(path.clone().into());
        Ok(())
    }

    fn collect_usages_from_declaration(
        declaration: &Declaration,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        assert!(declaration.template_parameters.is_empty());
        for attribute in declaration.attributes.iter() {
            for arg in attribute.arguments.iter().flatten() {
                Self::collect_usages_from_expression(arg, usages)?;
            }
        }

        if let Some(typ) = declaration.typ.as_ref() {
            Self::collect_usages_from_typ(typ, usages)?;
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
        assert!(alias.template_parameters.is_empty());
        Self::collect_usages_from_typ(&alias.typ, usages)?;
        Ok(())
    }

    fn collect_usages_from_const_assert(
        const_assert: &ConstAssert,
        usages: &mut Usages,
    ) -> Result<(), CompilerPassError> {
        assert!(const_assert.template_parameters.is_empty());
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
                Self::collect_usages_from_expression(&spanned, usages)
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
                    Self::collect_usages_from_expression(&arg, usages)?;
                }
                Ok(())
            }
            Expression::Identifier(identifier_expression) => {
                Self::collect_usages_from_path(&identifier_expression.path, usages)
            }
            Expression::Type(type_expression) => {
                Self::collect_usages_from_typ(type_expression, usages)
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
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
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
                    Self::collect_usages_from_compound_statement(&els, usages)?;
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
                    Self::collect_usages_from_expression(&arg, usages)?;
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
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(&arg, usages)?;
                }

                Self::collect_usages_from_compound_statement(&loop_statement.body, usages)?;

                if let Some(continuing) = loop_statement.continuing.as_ref() {
                    Self::collect_usages_from_compound_statement(&continuing.body, usages)?;
                    if let Some(break_if) = continuing.break_if.as_ref() {
                        Self::collect_usages_from_expression(&break_if, usages)?;
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
                    Self::collect_usages_from_expression(&arg, usages)?;
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
                    .map(|x| x.arguments.iter().flatten())
                    .flatten()
                {
                    Self::collect_usages_from_expression(&arg, usages)?;
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
        assert!(compound_statement.directives.is_empty());
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

impl Into<Spanned<GlobalDeclaration>> for OwnedMember {
    fn into(self) -> Spanned<GlobalDeclaration> {
        match self {
            OwnedMember::Global(spanned) => spanned,
            OwnedMember::Module(Spanned { value, span }) => Spanned::new(value.into(), span),
        }
    }
}

impl Into<Spanned<ModuleMemberDeclaration>> for OwnedMember {
    fn into(self) -> Spanned<ModuleMemberDeclaration> {
        match self {
            OwnedMember::Module(spanned) => spanned,
            OwnedMember::Global(Spanned { value, span }) => Spanned::new(value.into(), span),
        }
    }
}

enum Parent<'a> {
    TranslationUnit(&'a mut TranslationUnit),
    Module {
        module: &'a mut Module,
        is_initialized: bool,
    },
}

fn mangle_expression(expr: &Expression) -> String {
    let data = format!("{expr}").replace(' ', "").replace('\n', "");
    let mut result = String::new();
    for c in data.chars() {
        if c.is_alphanumeric() {
            result.push(c);
        } else {
            let mut buf = Vec::new();
            buf.resize(c.len_utf8(), 0b0);
            let _ = c.encode_utf8(&mut buf);
            result.push_str("__");
            for item in buf {
                let str = item.to_string();
                result.push_str(&str.len().to_string());
                result.push_str(&str);
            }
        }
    }
    result
}

fn mangle_path_part(path_part: &PathPart) -> String {
    let name = path_part.name.replace('_', "__");
    let mut template_args = String::new();
    for template_arg in path_part.template_args.iter().flatten() {
        template_args.push('_');
        mangle_expression(&template_arg.expression);
    }
    format!("{name}{template_args}")
}

impl<'a> BorrowedMember<'a> {
    fn try_into_parent(self) -> Result<Parent<'a>, ()> {
        match self {
            BorrowedMember::Global {
                global:
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
                module:
                    Spanned {
                        span: _,
                        value: ModuleMemberDeclaration::Module(m),
                    },
                is_initialized,
            } => Ok(Parent::Module {
                module: m,
                is_initialized,
            }),
            _ => Err(()),
        }
    }
}

impl<'a> From<&'a mut TranslationUnit> for Parent<'a> {
    fn from(value: &'a mut TranslationUnit) -> Self {
        Self::TranslationUnit(value)
    }
}

impl<'a> Parent<'a> {
    fn add_member<'b>(&'b mut self, member: OwnedMember) -> BorrowedMember<'b> {
        match self {
            Parent::TranslationUnit(t) => {
                t.global_declarations.push(member.into());
                BorrowedMember::Global {
                    global: t.global_declarations.last_mut().unwrap(),
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
                    module: m.members.last_mut().unwrap(),
                    is_initialized: false,
                }
            }
        }
    }

    fn find_child<'b>(&'b mut self, path_part: &PathPart) -> Option<BorrowedMember<'b>> {
        let name = Some(mangle_path_part(path_part));
        match self {
            Parent::TranslationUnit(x) => {
                for item in x.global_declarations.iter_mut() {
                    if item.name().map(|x| x.value) == name {
                        assert!(item.template_parameters().is_empty());
                        return Some(BorrowedMember::Global {
                            global: item,
                            is_initialized: true,
                        });
                    }
                }
                return None;
            }
            Parent::Module {
                module: x,
                is_initialized,
            } => {
                assert!(*is_initialized);
                for item in x.members.iter_mut() {
                    if item.name().map(|x| x.value) == name {
                        assert!(item.template_parameters().is_empty());
                        return Some(BorrowedMember::Module {
                            module: item,
                            is_initialized: true,
                        });
                    }
                }
                return None;
            }
        }
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

    fn initialize<'b>(
        &'b mut self,
        symbol_path: ConcreteSymbolPath,
        symbol_map: &mut SymbolMap,
        usages: &mut Usages,
        alias_cache: &mut AliasCache,
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
                    } else if let GlobalDeclaration::ConstAssert(_) = declaration.as_ref() {
                        entrypoints.push(declaration);
                        continue;
                    } else if let GlobalDeclaration::Alias(alias) = declaration.as_ref() {
                        assert!(alias.template_parameters.is_empty());
                        let mut alias_path = symbol_path.clone();
                        alias_path.push_back(PathPart {
                            name: alias.name.clone(),
                            template_args: None,
                        });
                        alias_cache.insert(alias_path, alias.typ.path.iter().cloned().collect());
                        continue;
                    }
                    symbol_map.insert(
                        SymbolPath {
                            parent: symbol_path.clone(),
                            name: declaration.name().unwrap(),
                        },
                        OwnedMember::Global(declaration),
                    );
                }

                t.global_declarations.append(&mut entrypoints);
                for member in t.global_declarations.iter_mut() {
                    let member = BorrowedMember::Global {
                        global: member,
                        is_initialized: true,
                    };
                    member.collect_usages(usages)?;
                }
                Ok(())
            }
            Parent::Module {
                module,
                is_initialized: false,
            } => {
                for declaration in module.members.drain(..) {
                    symbol_map.insert(
                        SymbolPath {
                            parent: symbol_path.clone(),
                            name: declaration.name().unwrap(),
                        },
                        OwnedMember::Module(declaration),
                    );
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SymbolPath {
    parent: ConcreteSymbolPath,
    name: Spanned<String>,
}

impl Specializer {
    fn specialize_translation_unit<'a>(
        translation_unit: &'a mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let mut symbol_map: SymbolMap = HashMap::new();
        let mut usages: Usages = Usages::new();
        let mut alias_cache = AliasCache::new();

        let mut parent: Parent<'a> = Parent::TranslationUnit(translation_unit);
        parent.initialize(
            im::Vector::new(),
            &mut symbol_map,
            &mut usages,
            &mut alias_cache,
        )?;

        while let Some(remaining_path) = usages.pop() {
            assert!(remaining_path.len() > 0);
            let current_path = im::Vector::new();
            Self::specialize(
                &mut parent,
                &mut usages,
                &mut symbol_map,
                &mut alias_cache,
                remaining_path,
                current_path,
            )?;
        }

        Ok(())
    }

    fn specialize<'a, 'b: 'a>(
        parent: &'a mut Parent<'b>,
        usages: &mut Usages,
        symbol_map: &mut SymbolMap,
        alias_cache: &mut AliasCache,
        mut remaining_path: ConcreteSymbolPath,
        mut current_path: ConcreteSymbolPath,
    ) -> Result<(), CompilerPassError> {
        assert!(!remaining_path.is_empty());
        let part = remaining_path.pop_front().unwrap();
        let current;
        if let Some(m) = parent.find_child(&part) {
            current = m;
        } else if let Some(m) = symbol_map.remove(&SymbolPath {
            parent: current_path.clone(),
            name: part.name.clone(),
        }) {
            // TODO: Specialize member
            current = parent.add_member(m);
        } else {
            return Err(CompilerPassError::UnableToResolvePath(
                current_path.iter().cloned().collect(),
            ));
        }
        current_path.push_back(part.clone());
        if remaining_path.is_empty() {
            return current.collect_usages(usages);
        } else if let Ok(mut p) = current.try_into_parent() {
            p.initialize(current_path.clone(), symbol_map, usages, alias_cache)?;
            return Self::specialize(
                &mut p,
                usages,
                symbol_map,
                alias_cache,
                current_path,
                remaining_path,
            );
        } else {
            return Err(CompilerPassError::UnableToResolvePath(
                current_path.iter().cloned().collect(),
            ));
        }
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
