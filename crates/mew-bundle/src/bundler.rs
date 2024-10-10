use mew_parse::{
    span::Spanned,
    syntax::{self, Module, ModuleDirective, TranslationUnit},
};
use mew_types::CompilerPass;

#[derive(Debug, Default)]
pub struct Bundler<'a> {
    pub sources: Vec<&'a str>,
    pub enclosing_module_name: Option<String>,
}

impl<'a> CompilerPass for Bundler<'a> {
    fn apply_mut(
        &mut self,
        translation_unit: &mut TranslationUnit,
    ) -> Result<(), mew_types::CompilerPassError> {
        let mut result: TranslationUnit = TranslationUnit::default();

        let mut ws: String = String::new();

        for file in self.sources.iter() {
            let file_len = file.len();
            let mut file_with_starting_ws = ws.clone();
            file_with_starting_ws.push_str(file);
            ws.extend((0..file_len).map(|_| ' '));
            let mut local_translation_unit = mew_parse::Parser::parse_str(&file_with_starting_ws)
                .map_err(|err| {
                    mew_types::CompilerPassError::ParseError(format!("{}", err), err.span())
                })?;
            result
                .global_declarations
                .append(&mut local_translation_unit.global_declarations);
            result
                .global_directives
                .append(&mut local_translation_unit.global_directives);
        }

        if let Some(module_name) = &self.enclosing_module_name {
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
                        translation_unit.global_directives.push(directive);
                    }
                };
            }
            translation_unit.global_declarations.push(Spanned::new(
                syntax::GlobalDeclaration::Module(module),
                module_span,
            ));
            Ok(())
        } else {
            translation_unit
                .global_declarations
                .append(&mut result.global_declarations);
            translation_unit
                .global_directives
                .append(&mut result.global_directives);

            Ok(())
        }
    }
}
