use std::path::PathBuf;

use wesl_parse::{
    span::Spanned,
    syntax::{self, Module, ModuleDirective, TranslationUnit},
};

use crate::file_system::ReadonlyFilesystem;

#[derive(Debug, Default, Clone, Copy)]
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

        let mut ws: String = String::new();

        for item in ctx.entry_points.iter() {
            let file = self
                .file_system
                .read(item)
                .await
                .map_err(BundlerError::FileSystemError)?;
            let file_len = file.len();
            let mut file_with_starting_ws = ws.clone();
            file_with_starting_ws.push_str(&file);
            ws.extend((0..file_len).map(|_| ' '));
            let mut local_translation_unit = wesl_parse::Parser::parse_str(&file_with_starting_ws)
                .map_err(|err| BundlerError::ParseError(format!("{}", err)))?;
            result
                .global_declarations
                .append(&mut local_translation_unit.global_declarations);
            result
                .global_directives
                .append(&mut local_translation_unit.global_directives);
        }

        if let Some(module_name) = &ctx.enclosing_module_name {
            let mut encapsulated_result: TranslationUnit = TranslationUnit::default();
            let mut module = Module {
                name: Spanned::new(module_name.to_owned(), 0..0),
                ..Module::default()
            };
            let mut module_span = 0..0;
            for declaration in result.global_declarations {
                let span = declaration.span();
                module_span.start = usize::min(span.start, module_span.start);
                module_span.end = usize::max(span.end, module_span.end);
                module
                    .members
                    .push(Spanned::new(declaration.value.into(), span));
            }
            for directive in result.global_directives {
                match TryInto::<Spanned<ModuleDirective>>::try_into(directive) {
                    Ok(dir) => {
                        module_span.start = usize::min(dir.span().start, module_span.start);
                        module_span.end = usize::max(dir.span().end, module_span.end);
                        module.directives.push(dir);
                    }
                    Err(directive) => {
                        encapsulated_result.global_directives.push(directive);
                    }
                };
            }
            encapsulated_result.global_declarations.push(Spanned::new(
                syntax::GlobalDeclaration::Module(module),
                module_span,
            ));
            Ok(encapsulated_result)
        } else {
            Ok(result)
        }
    }
}
