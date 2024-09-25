use super::{error::ParseError, syntax::*};
use crate::span::*;
use std::{collections::VecDeque, str::FromStr};

impl FromStr for DiagnosticSeverity {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(Self::Error),
            "warning" => Ok(Self::Warning),
            "info" => Ok(Self::Info),
            "off" => Ok(Self::Off),
            _ => Err(ParseError::ParseDiagnosticSeverity),
        }
    }
}

impl From<GlobalDeclaration> for ModuleMemberDeclaration {
    fn from(value: GlobalDeclaration) -> Self {
        match value {
            GlobalDeclaration::Void => ModuleMemberDeclaration::Void,
            GlobalDeclaration::Declaration(decl) => ModuleMemberDeclaration::Declaration(decl),
            GlobalDeclaration::Alias(alias) => ModuleMemberDeclaration::Alias(alias),
            GlobalDeclaration::Struct(strct) => ModuleMemberDeclaration::Struct(strct),
            GlobalDeclaration::Function(func) => ModuleMemberDeclaration::Function(func),
            GlobalDeclaration::ConstAssert(ass) => ModuleMemberDeclaration::ConstAssert(ass),
            GlobalDeclaration::Module(module) => ModuleMemberDeclaration::Module(module),
        }
    }
}

impl From<ModuleMemberDeclaration> for GlobalDeclaration {
    fn from(value: ModuleMemberDeclaration) -> Self {
        match value {
            ModuleMemberDeclaration::Void => GlobalDeclaration::Void,
            ModuleMemberDeclaration::Declaration(decl) => GlobalDeclaration::Declaration(decl),
            ModuleMemberDeclaration::Alias(alias) => GlobalDeclaration::Alias(alias),
            ModuleMemberDeclaration::Struct(strct) => GlobalDeclaration::Struct(strct),
            ModuleMemberDeclaration::Function(func) => GlobalDeclaration::Function(func),
            ModuleMemberDeclaration::ConstAssert(ass) => GlobalDeclaration::ConstAssert(ass),
            ModuleMemberDeclaration::Module(module) => GlobalDeclaration::Module(module),
        }
    }
}

impl DeclarationStatement {
    pub fn construct_scope_tree(&mut self, queue: &mut VecDeque<S<Statement>>) {
        while let Some(statement) = queue.pop_front() {
            let span = statement.span();
            match statement.value {
                Statement::Declaration(mut decl) => {
                    decl.construct_scope_tree(queue);
                    self.statements
                        .push(S::new(Statement::Declaration(decl), span));
                }
                other => self.statements.push(S::new(other, span)),
            }
        }
    }
}

impl CompoundStatement {
    pub fn construct_scope_tree(&mut self) {
        let mut queue: VecDeque<S<Statement>> = self.statements.drain(..).collect();
        while let Some(statement) = queue.pop_front() {
            let span = statement.span();
            match statement.value {
                Statement::Declaration(mut decl) => {
                    decl.construct_scope_tree(&mut queue);
                    self.statements
                        .push(S::new(Statement::Declaration(decl), span));
                }
                other => self.statements.push(S::new(other, span)),
            }
        }
    }
}

impl ModuleMemberDeclaration {
    pub fn name(&self) -> Option<S<String>> {
        match self {
            ModuleMemberDeclaration::Declaration(d) => Some(d.name.clone()),
            ModuleMemberDeclaration::Alias(a) => Some(a.name.clone()),
            ModuleMemberDeclaration::Struct(s) => Some(s.name.clone()),
            ModuleMemberDeclaration::Function(f) => Some(f.name.clone()),
            ModuleMemberDeclaration::Module(m) => Some(m.name.clone()),
            _ => None,
        }
    }
    pub fn template_parameters(&self) -> Vec<S<FormalTemplateParameter>> {
        match self {
            ModuleMemberDeclaration::Struct(s) => s.template_parameters.clone(),
            ModuleMemberDeclaration::Function(f) => f.template_parameters.clone(),
            ModuleMemberDeclaration::Module(m) => m.template_parameters.clone(),
            ModuleMemberDeclaration::Declaration(_) => vec![],
            ModuleMemberDeclaration::Alias(_) => vec![],
            ModuleMemberDeclaration::Void => vec![],
            ModuleMemberDeclaration::ConstAssert(_) => vec![],
        }
    }
}

impl GlobalDeclaration {
    pub fn name(&self) -> Option<S<String>> {
        match self {
            GlobalDeclaration::Declaration(d) => Some(d.name.clone()),
            GlobalDeclaration::Alias(a) => Some(a.name.clone()),
            GlobalDeclaration::Struct(s) => Some(s.name.clone()),
            GlobalDeclaration::Function(f) => Some(f.name.clone()),
            GlobalDeclaration::Module(m) => Some(m.name.clone()),
            _ => None,
        }
    }
    pub fn template_parameters(&self) -> Vec<S<FormalTemplateParameter>> {
        match self {
            GlobalDeclaration::Struct(s) => s.template_parameters.clone(),
            GlobalDeclaration::Function(f) => f.template_parameters.clone(),
            GlobalDeclaration::Module(m) => m.template_parameters.clone(),
            GlobalDeclaration::Declaration(_) => vec![],
            GlobalDeclaration::Alias(_) => vec![],
            GlobalDeclaration::Void => vec![],
            GlobalDeclaration::ConstAssert(_) => vec![],
        }
    }
}

impl From<TemplateElaboratedIdent> for TypeExpression {
    fn from(
        TemplateElaboratedIdent {
            path: S { value, span },
        }: TemplateElaboratedIdent,
    ) -> Self {
        TypeExpression {
            path: S::new(value.into_iter().map(|x| x.into()).collect(), span),
        }
    }
}

impl From<TemplateElaboratedIdent> for IdentifierExpression {
    fn from(
        TemplateElaboratedIdent {
            path: S { value, span },
        }: TemplateElaboratedIdent,
    ) -> Self {
        IdentifierExpression {
            path: S::new(value.into_iter().map(|x| x.into()).collect(), span),
        }
    }
}

impl From<TemplateElaboratedIdentPart> for PathPart {
    fn from(value: TemplateElaboratedIdentPart) -> Self {
        PathPart {
            name: value.name,
            template_args: value.template_args,
        }
    }
}
