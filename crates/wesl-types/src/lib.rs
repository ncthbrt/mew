use wesl_parse::{
    span::{Span, Spanned},
    syntax::{
        CompoundDirective, FormalTemplateParameter, GlobalDirective, ModuleDirective, PathPart,
        TranslationUnit,
    },
};

pub mod builtins;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerPassError {
    SymbolNotFound(Vec<PathPart>, Span),
    MissingRequiredTemplateArgument(Spanned<FormalTemplateParameter>, Span),
    UnknownTemplateArgument(Span),
    InternalError(InternalCompilerError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InternalCompilerError {
    UnexpectedGlobalDirective(GlobalDirective, Span),
    UnexpectedModuleDirective(ModuleDirective, Span),
    UnexpectedCompoundDirective(CompoundDirective, Span),
}

pub trait CompilerPass {
    fn apply_mut(
        &mut self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError>;

    fn apply(
        &mut self,
        translation_unit: &TranslationUnit,
    ) -> Result<TranslationUnit, CompilerPassError> {
        let mut clone = translation_unit.clone();
        self.apply_mut(&mut clone)?;
        Ok(clone)
    }
}
