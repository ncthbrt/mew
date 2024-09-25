use wesl_parse::{
    span::Spanned,
    syntax::{self, *},
};
use wesl_types::{CompilerPass, CompilerPassError};

#[derive(Debug, Default, Clone, Copy)]
struct Specializer;

type SymbolPath = im::Vector<PathPart>;

#[derive(Debug, Clone, PartialEq, Hash)]
enum ModuleOrGlobalDeclaration {
    Global(GlobalDeclaration),
    Module(ModuleMemberDeclaration),
}

#[derive(Debug, Clone, PartialEq, Hash)]
struct GenericPath {
    parent: SymbolPath,
    declaration: ModuleOrGlobalDeclaration,
}

impl GenericPath {
    fn name(&self) -> Spanned<String> {
        match &self.declaration {
            ModuleOrGlobalDeclaration::Global(global_declaration) => {
                global_declaration.name().unwrap()
            }
            ModuleOrGlobalDeclaration::Module(module_member_declaration) => {
                module_member_declaration.name().unwrap()
            }
        }
    }

    fn template_parameters(&self) -> Vec<Spanned<FormalTemplateParameter>> {
        match &self.declaration {
            ModuleOrGlobalDeclaration::Global(global_declaration) => {
                global_declaration.template_parameters()
            }
            ModuleOrGlobalDeclaration::Module(module_member_declaration) => {
                module_member_declaration.template_parameters()
            }
        }
    }
}

impl Specializer {
    fn specialize_module(
        module: &mut Module,
        mut parent_path: SymbolPath,
    ) -> Result<(), CompilerPassError> {
        Ok(())
    }

    fn specialize_translation_unit(
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), CompilerPassError> {
        let mut generic_symbols: Vec<Spanned<GlobalDeclaration>> = vec![];
        let mut realised_symbols: Vec<Spanned<GlobalDeclaration>> = vec![];
        for declaration in translation_unit.global_declarations.drain(..) {}

        translation_unit
            .global_declarations
            .append(&mut realised_symbols);

        let symbol_path = SymbolPath::new();

        for declaration in translation_unit.global_declarations.iter_mut() {}

        Ok(())
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
