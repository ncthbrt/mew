use mew_parse::{
    span::{Span, Spanned},
    syntax::{
        CompoundDirective, FormalTemplateParameter, GlobalDirective, ModuleDirective, PathPart,
        TranslationUnit,
    },
};

pub mod builtins;
pub mod mangling;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerPassError {
    SymbolNotFound(Vec<PathPart>, Span),
    UnableToResolvePath(Vec<PathPart>),
    MissingRequiredTemplateArgument(Spanned<FormalTemplateParameter>, Span),
    InternalError(InternalCompilerError),
    MalformedTemplateArgument(Span),
    ParseError(String, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InternalCompilerError {
    UnexpectedGlobalDirective(GlobalDirective, Span),
    UnexpectedModuleDirective(ModuleDirective, Span),
    UnexpectedCompoundDirective(CompoundDirective, Span),
    UnexpectedMember,
}

pub type CompilerPassResult<T = ()> = std::result::Result<T, Box<CompilerPassError>>;

pub trait CompilerPass {
    fn apply_mut(&mut self, translation_unit: &mut TranslationUnit) -> CompilerPassResult;

    fn apply(&mut self, translation_unit: &TranslationUnit) -> CompilerPassResult<TranslationUnit> {
        let mut clone = translation_unit.clone();
        self.apply_mut(&mut clone)?;
        Ok(clone)
    }
}
