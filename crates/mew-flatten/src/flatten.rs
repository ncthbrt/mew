use mew_parse::{
    span::Spanned,
    syntax::{GlobalDeclaration, Module, ModuleMemberDeclaration, TranslationUnit},
};
use mew_types::CompilerPass;
use std::convert::Into;

#[derive(Debug, Default, Clone, Copy)]
pub struct Flattener;

impl Flattener {
    fn flatten_module(translation_unit: &mut TranslationUnit, mut module: Module) {
        for decl in module.members.drain(..) {
            let span = decl.span();
            match decl.value {
                ModuleMemberDeclaration::Module(m) => {
                    Self::flatten_module(translation_unit, m);
                }
                other => translation_unit
                    .global_declarations
                    .push(Spanned::new(other.into(), span)),
            }
        }
    }

    pub fn flatten_mut(&self, translation_unit: &mut TranslationUnit) {
        let mut modules = vec![];
        let mut others = vec![];
        for decl in translation_unit.global_declarations.drain(..) {
            if let GlobalDeclaration::Module(m) = decl.value {
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
    ) -> mew_types::CompilerPassResult {
        self.flatten_mut(translation_unit);
        Ok(())
    }
}
