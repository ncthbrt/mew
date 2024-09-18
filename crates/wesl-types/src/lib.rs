use wesl_parse::syntax::{CompoundDirective, GlobalDirective, ModuleDirective, TranslationUnit};

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerPassError {
    SymbolNotFound(Vec<String>),
    AmbiguousScope(String),
    UnexpectedGlobalDirective(GlobalDirective),
    UnexpectedModuleDirective(ModuleDirective),
    UnexpectedCompoundDirective(CompoundDirective),
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
