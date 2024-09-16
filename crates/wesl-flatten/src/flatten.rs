use std::convert::Into;
use wesl_parse::syntax::{GlobalDeclaration, Module, ModuleMemberDeclaration, TranslationUnit};
use wesl_types::CompilerPass;

#[derive(Debug, Default, Clone, Copy)]
pub struct Flattener;

impl Flattener {
    fn flatten_module(translation_unit: &mut TranslationUnit, mut module: Module) {
        for decl in module.members.drain(0..) {
            match decl {
                ModuleMemberDeclaration::Module(m) => {
                    Self::flatten_module(translation_unit, m);
                }
                other => translation_unit.global_declarations.push(other.into()),
            }
        }
    }

    pub fn flatten_mut(&self, translation_unit: &mut TranslationUnit) {
        let mut modules = vec![];
        let mut others = vec![];
        for decl in translation_unit.global_declarations.drain(0..) {
            if let GlobalDeclaration::Module(m) = decl {
                modules.push(m);
            } else {
                others.push(decl);
            }
        }
        translation_unit.global_declarations.append(&mut others);
        for m in modules {
            Self::flatten_module(translation_unit, m);
        }
    }
}

impl CompilerPass for Flattener {
    fn apply_mut(
        &mut self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), wesl_types::CompilerPassError> {
        self.flatten_mut(translation_unit);
        Ok(())
    }
}
