use std::str::FromStr;

use super::{error::ParseError, syntax::*};

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
