use wesl_types::CompilerPass;

#[derive(Debug, Default, Clone, Copy)]
struct TemplateSpecializer;

impl CompilerPass for TemplateSpecializer {
    fn apply_mut(
        &mut self,
        _translation_unit: &mut wesl_parse::syntax::TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        Ok(())
    }
}
