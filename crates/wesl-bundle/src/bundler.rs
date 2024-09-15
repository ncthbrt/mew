use std::path::PathBuf;

use wesl_parse::syntax::{self, Module, TranslationUnit};

use crate::file_system::ReadonlyFilesystem;

pub struct Bundler<Fs: ReadonlyFilesystem> {
    pub file_system: Fs,
}

#[derive(Debug)]
pub enum BundlerError<FileSystemError> {
    FileSystemError(FileSystemError),
    ParseError(String),
}

#[derive(Default, Debug)]
pub struct BundleContext {
    pub entry_points: Vec<PathBuf>,
    pub enclosing_module_name: Option<String>,
}

impl<Fs: ReadonlyFilesystem> Bundler<Fs> {
    pub async fn bundle(
        &self,
        ctx: &BundleContext,
    ) -> Result<TranslationUnit, BundlerError<Fs::Error>> {
        let mut result: TranslationUnit = TranslationUnit::default();

        for item in ctx.entry_points.iter() {
            let file = self
                .file_system
                .read(item)
                .await
                .map_err(BundlerError::FileSystemError)?;
            let mut local_translation_unit = wesl_parse::Parser::parse_str(&file)
                .map_err(|err| BundlerError::ParseError(format!("{}", err)))?;

            result
                .global_declarations
                .append(&mut local_translation_unit.global_declarations);
        }

        if let Some(module_name) = &ctx.enclosing_module_name {
            let mut encapsulated_result: TranslationUnit = TranslationUnit::default();
            encapsulated_result
                .global_directives
                .append(&mut result.global_directives);
            let mut module = Module {
                name: module_name.to_owned(),
                ..Module::default()
            };
            for declaration in result.global_declarations {
                module.members.push(declaration.into());
            }
            encapsulated_result
                .global_declarations
                .push(syntax::GlobalDeclaration::Module(module));
            Ok(encapsulated_result)
        } else {
            Ok(result)
        }
    }
}
