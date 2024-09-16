use wesl_parse::syntax::TranslationUnit;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerPassError {
    SymbolNotFound(Vec<String>),
    AmbiguousScope(String),
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
