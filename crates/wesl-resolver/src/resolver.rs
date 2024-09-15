use std::collections::VecDeque;

use wesl_parse::syntax::{
    CompoundStatement, ContinuingStatement, DeclarationStatement, Expression, ForStatement,
    FormalParameter, Function, GlobalDeclaration, LoopStatement, Module, ModuleMemberDeclaration,
    Statement, TranslationUnit,
};

#[derive(Debug, Default)]
pub struct Resolver;

#[derive(Debug)]
pub enum ResolverError {
    SymbolNotFound(Vec<String>),
    AmbiguousScope(String),
}

#[derive(Debug, PartialEq, Clone)]
struct ModulePath(im::Vector<String>);

// #[derive(Debug, PartialEq, Clone)]
// struct ScopeMemberInner<T: Clone>(T);

// #[derive(Debug, PartialEq, Clone)]
// enum ScopeMember {
//     LocalDeclaration(ScopeMemberInner<DeclarationStatement>),
//     ModuleMemberDeclaration(ScopeMemberInner<ModuleMemberDeclaration>, ModulePath),
//     GlobalDeclaration(ScopeMemberInner<GlobalDeclaration>),
//     FormalFunctionParameter(ScopeMemberInner<FormalParameter>),
// }

// #[derive(Debug, PartialEq, Clone)]
// struct ScopeInner<'syntax, T: 'syntax>(&'syntax T);

// #[derive(Debug, PartialEq)]
// enum Scope<'syntax> {
//     Global(ScopeInner<'syntax, TranslationUnit>),
//     LocalDeclaration(ScopeInner<'syntax, DeclarationStatement>),
//     Compound(ScopeInner<'syntax, CompoundStatement>),
//     Function(ScopeInner<'syntax, Function>),
//     Module(ScopeInner<'syntax, Module>, ModulePath),
// }

// impl<'syntax> Scope<'syntax> {
//     fn direct_child_statements(&self) -> Option<Vec<Statement>> {
//         match self {
//             Scope::LocalDeclaration(l) => Some(l.0.statements.clone()),
//             Scope::Compound(c) => Some(c.0.statements.clone()),
//             Scope::Function(f) => Some(f.0.body.statements.clone()),
//             _ => None,
//         }
//     }
// }

// impl ScopeMember {
//     fn name(&self) -> Option<String> {
//         match self {
//             ScopeMember::LocalDeclaration(local) => Some(local.0.declaration.name.clone()),
//             ScopeMember::ModuleMemberDeclaration(m, _) => m.0.name().clone(),
//             ScopeMember::GlobalDeclaration(g) => g.0.name().clone(),
//             ScopeMember::FormalFunctionParameter(f) => Some(f.0.name.clone()),
//         }
//     }
// }

// #[derive(Debug)]
// struct ScopeWithMembers {
//     scope: Scope,
//     parent_members: im::HashMap<String, ScopeMember>,
// }

// impl ScopeWithMembers {
//     fn new(scope: Scope, parent_members: im::HashMap<String, ScopeMember>) -> ScopeWithMembers {
//         return ScopeWithMembers {
//             scope,
//             parent_members,
//         };
//     }

//     fn members(&self) -> im::HashMap<String, ScopeMember> {
//         let parent_members = self.parent_members.clone();
//         let new_members = match &self.scope {
//             Scope::Global(translation_unit) => translation_unit
//                 .0
//                 .global_declarations
//                 .iter()
//                 .filter_map(|x: &GlobalDeclaration| {
//                     if let Some(name) = x.name() {
//                         Some((
//                             name,
//                             ScopeMember::GlobalDeclaration(ScopeMemberInner(x.clone())),
//                         ))
//                     } else {
//                         None
//                     }
//                 })
//                 .collect::<im::HashMap<String, ScopeMember>>(),
//             Scope::LocalDeclaration(decl) => {
//                 let name: String = decl.0.declaration.name.clone();
//                 im::HashMap::unit(
//                     name,
//                     ScopeMember::LocalDeclaration(ScopeMemberInner(decl.0.clone())),
//                 )
//             }
//             Scope::Compound(_) => im::HashMap::new(),
//             Scope::Function(f) => {
//                 f.0.parameters
//                     .iter()
//                     .map(|x| {
//                         (
//                             x.name.clone(),
//                             ScopeMember::FormalFunctionParameter(ScopeMemberInner(x.clone())),
//                         )
//                     })
//                     .collect()
//             }
//             Scope::Module(module, path) => module
//                 .0
//                 .members
//                 .iter()
//                 .filter_map(|x| {
//                     if let Some(name) = x.name() {
//                         let mut path: ModulePath = path.clone();
//                         path.0.push_back(module.0.name.clone());
//                         Some((
//                             name,
//                             ScopeMember::ModuleMemberDeclaration(ScopeMemberInner(x.clone()), path),
//                         ))
//                     } else {
//                         None
//                     }
//                 })
//                 .collect(),
//         };
//         new_members.union(parent_members)
//     }

//     fn statement_to_child_scopes(
//         &self,
//         statement: &mut Statement,
//     ) -> Result<Vec<ScopeWithMembers>, ResolverError> {
//         let members = self.members();
//         match statement {
//             Statement::Void
//             | Statement::Increment(_)
//             | Statement::Assignment(_)
//             | Statement::Decrement(_)
//             | Statement::Break
//             | Statement::Continue
//             | Statement::Return(_)
//             | Statement::Discard
//             | Statement::FunctionCall(_)
//             | Statement::ConstAssert(_) => Ok(vec![]),
//             Statement::Compound(c) => Ok(vec![ScopeWithMembers::new(
//                 Scope::Compound(ScopeInner(c.clone())),
//                 members,
//             )]),
//             Statement::If(iff) => {
//                 let mut result = vec![ScopeWithMembers::new(
//                     Scope::Compound(ScopeInner(iff.if_clause.1.clone())),
//                     members.clone(),
//                 )];
//                 for else_if_clause in iff.else_if_clauses.iter_mut() {
//                     result.push(ScopeWithMembers::new(
//                         Scope::Compound(ScopeInner(else_if_clause.1.clone())),
//                         members.clone(),
//                     ));
//                 }
//                 if let Some(else_clause) = iff.else_clause.as_mut() {
//                     result.push(ScopeWithMembers::new(
//                         Scope::Compound(ScopeInner(else_clause.clone())),
//                         members.clone(),
//                     ));
//                 }
//                 Ok(result)
//             }
//             Statement::Switch(s) => Ok(s
//                 .clauses
//                 .iter_mut()
//                 .map(|x| {
//                     ScopeWithMembers::new(
//                         Scope::Compound(ScopeInner(x.body.clone())),
//                         members.clone(),
//                     )
//                 })
//                 .collect()),
//             Statement::Loop(LoopStatement {
//                 attributes: _,
//                 body,
//                 continuing,
//             }) => {
//                 if let Some(ContinuingStatement {
//                     body: continuing_body,
//                     break_if: _,
//                 }) = continuing.as_mut()
//                 {
//                     let compound = ScopeWithMembers::new(
//                         Scope::Compound(ScopeInner(body.clone())),
//                         members.clone(),
//                     );
//                     let mut c = vec![compound];
//                     let members = c.get_mut(0).unwrap().members();

//                     c.push(ScopeWithMembers::new(
//                         Scope::Compound(ScopeInner(continuing_body.clone())),
//                         members,
//                     ));
//                     return Ok(c);
//                 } else {
//                     return Ok(vec![ScopeWithMembers::new(
//                         Scope::Compound(ScopeInner(body.clone())),
//                         members.clone(),
//                     )]);
//                 }
//             }
//             Statement::For(ForStatement {
//                 attributes: _,
//                 initializer,
//                 condition: _,
//                 update,
//                 body,
//             }) => {
//                 let mut result = vec![];
//                 let members = if let Some(initializer) = initializer.as_mut() {
//                     let mut children = self.statement_to_child_scopes(initializer)?;
//                     if children.len() > 1 {
//                         return Err(ResolverError::AmbiguousScope(format!("AMBIGUOUS SCOPE")));
//                     }
//                     if children.len() == 1 {
//                         let fst = children.remove(0);
//                         let members = fst.members();
//                         result.push(fst);
//                         Ok(members)
//                     } else {
//                         Ok(members.clone())
//                     }
//                 } else {
//                     Ok(members.clone())
//                 }?;

//                 result.push(ScopeWithMembers::new(
//                     Scope::Compound(ScopeInner(body.clone())),
//                     members.clone(),
//                 ));

//                 if let Some(update) = update.as_mut() {
//                     let rest = self.statement_to_child_scopes(update)?;
//                     result.extend(rest);
//                 };
//                 Ok(result)
//             }
//             Statement::While(w) => Ok(vec![ScopeWithMembers::new(
//                 Scope::Compound(ScopeInner(w.body.clone())),
//                 members.clone(),
//             )]),
//             Statement::Declaration(d) => Ok(vec![ScopeWithMembers::new(
//                 Scope::LocalDeclaration(ScopeInner(d.clone())),
//                 members.clone(),
//             )]),
//         }
//     }

//     fn child_scopes(&mut self) -> Result<Vec<ScopeWithMembers>, ResolverError> {
//         let members = self.members();
//         match &mut self.scope {
//             Scope::Global(translation_unit) => Ok(translation_unit
//                 .0
//                 .global_declarations
//                 .clone()
//                 .iter_mut()
//                 .filter_map(|x| {
//                     if let GlobalDeclaration::Module(m) = x {
//                         let path = ModulePath(im::Vector::new());
//                         Some(ScopeWithMembers::new(
//                             Scope::Module(ScopeInner(m.clone()), path),
//                             members.clone(),
//                         ))
//                     } else if let GlobalDeclaration::Function(f) = x {
//                         Some(ScopeWithMembers::new(
//                             Scope::Function(ScopeInner(f.clone())),
//                             members.clone(),
//                         ))
//                     } else {
//                         None
//                     }
//                 })
//                 .collect()),
//             Scope::Module(outer_m, path) => Ok(outer_m
//                 .0
//                 .members
//                 .clone()
//                 .iter_mut()
//                 .filter_map(|x| {
//                     if let ModuleMemberDeclaration::Module(inner_m) = x {
//                         let mut path: ModulePath = path.clone();
//                         path.0.push_back(outer_m.0.name.clone());
//                         Some(ScopeWithMembers::new(
//                             Scope::Module(ScopeInner(inner_m.clone()), path),
//                             members.clone(),
//                         ))
//                     } else if let ModuleMemberDeclaration::Function(f) = x {
//                         Some(ScopeWithMembers::new(
//                             Scope::Function(ScopeInner(f.clone())),
//                             members.clone(),
//                         ))
//                     } else {
//                         None
//                     }
//                 })
//                 .collect()),
//             rest => {
//                 if let Some(mut rest) = rest.direct_child_statements() {
//                     let mut result = vec![];
//                     for item in rest.iter_mut() {
//                         let mut child_scope = self.statement_to_child_scopes(item)?;
//                         result.append(&mut child_scope);
//                     }
//                     Ok(result)
//                 } else {
//                     Ok(vec![])
//                 }
//             }
//         }
//     }

//     fn get_symbol(&mut self, symbol: &str) -> Option<ScopeMember> {
//         let mut members: im::HashMap<String, ScopeMember> = self.members();
//         members.remove(symbol)
//     }

//     fn relative_path_to_absolute_path(
//         mut members: im::HashMap<String, ScopeMember>,
//         path: &mut Vec<String>,
//     ) -> Result<(), ResolverError> {
//         if let Some(symbol) = members.remove(path.first().unwrap().as_str()) {
//             match symbol {
//                 ScopeMember::LocalDeclaration(_) => {
//                     // No action required
//                 }
//                 ScopeMember::ModuleMemberDeclaration(_, module_path) => {
//                     let mut new_path = module_path
//                         .0
//                         .iter()
//                         .map(|x| x.to_string())
//                         .collect::<Vec<String>>();
//                     new_path.extend(path.iter().cloned());
//                     *path = new_path;
//                 }
//                 ScopeMember::GlobalDeclaration(_) => {
//                     // No action required
//                 }
//                 ScopeMember::FormalFunctionParameter(_) => {
//                     // No action required
//                 }
//             }
//         } else {
//             return Err(ResolverError::SymbolNotFound(path.clone().to_owned()));
//         };

//         Ok(())
//     }

//     fn expression_to_absolute_paths(
//         members: im::HashMap<String, ScopeMember>,
//         expression: &mut Expression,
//     ) -> Result<(), ResolverError> {
//         match expression {
//             Expression::Literal(_) => {}
//             Expression::Parenthesized(p) => {
//                 Self::expression_to_absolute_paths(members, p.as_mut())?
//             }
//             Expression::NamedComponent(n) => {
//                 Self::expression_to_absolute_paths(members, &mut n.base)?
//             }
//             Expression::Indexing(idx) => {
//                 Self::expression_to_absolute_paths(members, &mut idx.base)?
//             }
//             Expression::Unary(u) => Self::expression_to_absolute_paths(members, &mut u.operand)?,
//             Expression::Binary(b) => {
//                 Self::expression_to_absolute_paths(members.clone(), &mut b.left);
//                 Self::expression_to_absolute_paths(members, &mut b.right);
//             }
//             Expression::FunctionCall(f) => {
//                 Self::relative_path_to_absolute_path(members, &mut f.path)?;
//             }
//             Expression::Identifier(ident) => {
//                 Self::relative_path_to_absolute_path(members, &mut ident.path.clone())?;
//             }
//             Expression::Type(typ) => {
//                 Self::relative_path_to_absolute_path(members, &mut typ.path.clone())?;
//             }
//         };
//         Ok(())
//     }

//     fn statement_to_absolute_paths(statement: &mut Statement) -> Result<(), ResolverError> {
//         match statement {
//             Statement::Void => {}
//             Statement::Compound(c) => {
//                 for c in c.statements.iter_mut() {
//                     Self::statement_to_absolute_paths(c)?;
//                 }
//             }
//             Statement::Assignment(a) => {}
//             Statement::Increment(_) => todo!(),
//             Statement::Decrement(_) => todo!(),
//             Statement::If(_) => todo!(),
//             Statement::Switch(_) => todo!(),
//             Statement::Loop(_) => todo!(),
//             Statement::For(_) => todo!(),
//             Statement::While(_) => todo!(),
//             Statement::Break => todo!(),
//             Statement::Continue => todo!(),
//             Statement::Return(_) => todo!(),
//             Statement::Discard => todo!(),
//             Statement::FunctionCall(_) => todo!(),
//             Statement::ConstAssert(_) => todo!(),
//             Statement::Declaration(_) => todo!(),
//         };
//         Ok(())
//     }

//     fn to_absolute_paths(&mut self) -> Result<(), ResolverError> {
//         let members = self.members();
//         match &mut self.scope {
//             Scope::LocalDeclaration(decl) => {
//                 for statement in decl.0.statements.iter_mut() {
//                     Self::statement_to_absolute_paths(statement)?;
//                 }
//             }
//             Scope::Compound(c) => {
//                 for statement in c.0.statements.iter_mut() {
//                     Self::statement_to_absolute_paths(statement)?;
//                 }
//             }
//             Scope::Function(f) => {
//                 if let Some(return_type) = f.0.return_type.as_mut() {
//                     ScopeWithMembers::relative_path_to_absolute_path(
//                         members.clone(),
//                         &mut return_type.path,
//                     )?;
//                 };
//                 for s in f.0.parameters.iter_mut() {
//                     ScopeWithMembers::relative_path_to_absolute_path(
//                         members.clone(),
//                         &mut s.typ.path,
//                     )?;
//                 }
//                 // ADD BODY
//             }
//             Scope::Module(m, p) => {}
//             Scope::Global(_) => {}
//         };
//         Ok(())
//     }
// }

impl Resolver {
    fn module_to_absolute_path(
        module: &mut Module,
        module_path: ModulePath,
        scope: im::HashMap<String, String>,
    ) -> Result<(), ResolverError> {
    }

    fn translation_unit_to_absolute_path(
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), ResolverError> {
        for decl in translation_unit.global_declarations.iter_mut() {
            match decl {
                GlobalDeclaration::Void => {}
                GlobalDeclaration::Declaration(_) => {}
                GlobalDeclaration::Alias(_) => {}
                GlobalDeclaration::Struct(_) => {}
                GlobalDeclaration::Function(_) => {}
                GlobalDeclaration::ConstAssert(_) => {}
                GlobalDeclaration::Module(m) => {
                    Self::module_to_absolute_path(m, ModulePath(im::Vector::new()))?;
                }
            }
        }
        // let members = self.members();
        // match &mut self.scope {
        //     Scope::LocalDeclaration(decl) => {
        //         for statement in decl.0.statements.iter_mut() {
        //             Self::statement_to_absolute_paths(statement)?;
        //         }
        //     }
        //     Scope::Compound(c) => {
        //         for statement in c.0.statements.iter_mut() {
        //             Self::statement_to_absolute_paths(statement)?;
        //         }
        //     }
        //     Scope::Function(f) => {
        //         if let Some(return_type) = f.0.return_type.as_mut() {
        //             ScopeWithMembers::relative_path_to_absolute_path(
        //                 members.clone(),
        //                 &mut return_type.path,
        //             )?;
        //         };
        //         for s in f.0.parameters.iter_mut() {
        //             ScopeWithMembers::relative_path_to_absolute_path(
        //                 members.clone(),
        //                 &mut s.typ.path,
        //             )?;
        //         }
        //         // ADD BODY
        //     }
        //     Scope::Module(m, p) => {}
        //     Scope::Global(_) => {}
        // };
        Ok(())
    }
    pub fn resolve_mut(&self, translation_unit: &mut TranslationUnit) -> Result<(), ResolverError> {
        let scope_members = im::HashMap::<String, ScopeMember>::new();

        ScopeWithMembers::new(Scope::Global(ScopeInner(translation_unit)), scope_members)
            .to_absolute_paths()?;

        Ok(())
    }
}
