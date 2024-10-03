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
    pub fn name_mut(&mut self) -> Option<&mut S<String>> {
        match self {
            ModuleMemberDeclaration::Declaration(d) => Some(&mut d.name),
            ModuleMemberDeclaration::Alias(a) => Some(&mut a.name),
            ModuleMemberDeclaration::Struct(s) => Some(&mut s.name),
            ModuleMemberDeclaration::Function(f) => Some(&mut f.name),
            ModuleMemberDeclaration::Module(m) => Some(&mut m.name),
            ModuleMemberDeclaration::Void => None,
            ModuleMemberDeclaration::ConstAssert(_) => None,
        }
    }

    pub fn template_parameters_mut(&mut self) -> Option<&mut Vec<S<FormalTemplateParameter>>> {
        match self {
            ModuleMemberDeclaration::Struct(decl) => Some(&mut decl.template_parameters),
            ModuleMemberDeclaration::Function(decl) => Some(&mut decl.template_parameters),
            ModuleMemberDeclaration::Module(decl) => Some(&mut decl.template_parameters),
            ModuleMemberDeclaration::Declaration(decl) => Some(&mut decl.template_parameters),
            ModuleMemberDeclaration::Alias(decl) => Some(&mut decl.template_parameters),
            ModuleMemberDeclaration::ConstAssert(decl) => Some(&mut decl.template_parameters),
            ModuleMemberDeclaration::Void => None,
        }
    }

    pub fn template_parameters(&self) -> Option<&Vec<S<FormalTemplateParameter>>> {
        match self {
            ModuleMemberDeclaration::Struct(decl) => Some(&decl.template_parameters),
            ModuleMemberDeclaration::Function(decl) => Some(&decl.template_parameters),
            ModuleMemberDeclaration::Module(decl) => Some(&decl.template_parameters),
            ModuleMemberDeclaration::Declaration(decl) => Some(&decl.template_parameters),
            ModuleMemberDeclaration::Alias(decl) => Some(&decl.template_parameters),
            ModuleMemberDeclaration::Void => None,
            ModuleMemberDeclaration::ConstAssert(decl) => Some(&decl.template_parameters),
        }
        .and_then(|x| if x.is_empty() { None } else { Some(x) })
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
            GlobalDeclaration::Void => None,
            GlobalDeclaration::ConstAssert(_) => None,
        }
    }

    pub fn name_mut(&mut self) -> Option<&mut S<String>> {
        match self {
            GlobalDeclaration::Declaration(d) => Some(&mut d.name),
            GlobalDeclaration::Alias(a) => Some(&mut a.name),
            GlobalDeclaration::Struct(s) => Some(&mut s.name),
            GlobalDeclaration::Function(f) => Some(&mut f.name),
            GlobalDeclaration::Module(m) => Some(&mut m.name),
            GlobalDeclaration::Void => None,
            GlobalDeclaration::ConstAssert(_) => None,
        }
    }

    pub fn template_parameters_mut(&mut self) -> Option<&mut Vec<S<FormalTemplateParameter>>> {
        match self {
            GlobalDeclaration::Struct(s) => Some(&mut s.template_parameters),
            GlobalDeclaration::Function(f) => Some(&mut f.template_parameters),
            GlobalDeclaration::Module(m) => Some(&mut m.template_parameters),
            GlobalDeclaration::Declaration(decl) => Some(&mut decl.template_parameters),
            GlobalDeclaration::Alias(alias) => Some(&mut alias.template_parameters),
            GlobalDeclaration::Void => None,
            GlobalDeclaration::ConstAssert(assrt) => Some(&mut assrt.template_parameters),
        }
    }

    pub fn template_parameters(&self) -> Option<&Vec<S<FormalTemplateParameter>>> {
        match self {
            GlobalDeclaration::Struct(s) => Some(&s.template_parameters),
            GlobalDeclaration::Function(f) => Some(&f.template_parameters),
            GlobalDeclaration::Module(m) => Some(&m.template_parameters),
            GlobalDeclaration::Declaration(decl) => Some(&decl.template_parameters),
            GlobalDeclaration::Alias(alias) => Some(&alias.template_parameters),
            GlobalDeclaration::Void => None,
            GlobalDeclaration::ConstAssert(assrt) => Some(&assrt.template_parameters),
        }
        .and_then(|x| if x.is_empty() { None } else { Some(x) })
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

impl TryInto<S<Vec<PathPart>>> for Expression {
    type Error = ();
    fn try_into(self) -> Result<S<Vec<PathPart>>, Self::Error> {
        match self {
            Expression::Literal(_) => Err(()),
            Expression::Parenthesized(spanned) => spanned.into_inner().try_into(),
            Expression::NamedComponent(_) => Err(()),
            Expression::Indexing(_) => Err(()),
            Expression::Unary(_) => Err(()),
            Expression::Binary(_) => Err(()),
            Expression::FunctionCall(_) => Err(()),
            Expression::Identifier(identifier_expression) => Ok(identifier_expression.path),
            Expression::Type(type_expression) => Ok(type_expression.path),
        }
    }
}

impl TryInto<S<ModuleDirective>> for S<GlobalDirective> {
    type Error = S<GlobalDirective>;
    fn try_into(self) -> Result<S<ModuleDirective>, Self::Error> {
        let span = self.span();
        match self.value {
            GlobalDirective::Diagnostic(diagnostic_directive) => Err(S::new(
                GlobalDirective::Diagnostic(diagnostic_directive),
                span,
            )),
            GlobalDirective::Enable(enable_directive) => {
                Err(S::new(GlobalDirective::Enable(enable_directive), span))
            }
            GlobalDirective::Requires(requires_directive) => {
                Err(S::new(GlobalDirective::Requires(requires_directive), span))
            }
            GlobalDirective::Use(usage) => Ok(S::new(ModuleDirective::Use(usage), span)),
            GlobalDirective::Extend(extend_directive) => {
                Ok(S::new(ModuleDirective::Extend(extend_directive), span))
            }
        }
    }
}
