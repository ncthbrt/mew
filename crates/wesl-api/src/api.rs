use wesl_bundle::Bundler;
use wesl_parse::{
    span::Span,
    syntax::{FormalTemplateParameter, GlobalDeclaration, PathPart, TranslationUnit},
};
use wesl_types::{CompilerPass, CompilerPassError, InternalCompilerError};

#[derive(Default, Debug)]
pub struct WeslApi {
    pub translation_unit: TranslationUnit,
}

pub enum Source<'a> {
    Ast(&'a mut TranslationUnit),
    Text(&'a str),
}

pub struct ModuleDescriptor<'a> {
    pub module_name: &'a str,
    pub source: Source<'a>,
}

#[derive(Debug)]
pub enum WeslErrorInner {
    ModuleNotFound,
    SymbolNotFound(Vec<PathPart>),
    MissingRequiredTemplateArgument(FormalTemplateParameter),
    InternalError(InternalCompilerError),
    MalformedTemplateArgument,
    ParseError(String),
}

#[derive(Debug)]
pub struct WeslError {
    pub span: Option<Span>,
    pub module_name: Option<String>,
    pub error: WeslErrorInner,
}

pub enum Path {
    Parsed(Vec<PathPart>),
    Text(String),
}

type Result<T = ()> = std::result::Result<T, WeslError>;

impl From<CompilerPassError> for WeslError {
    fn from(value: CompilerPassError) -> Self {
        match value {
            CompilerPassError::SymbolNotFound(vec, range) => WeslError {
                span: Some(range),
                module_name: None,
                error: WeslErrorInner::SymbolNotFound(vec),
            },
            CompilerPassError::UnableToResolvePath(vec) => WeslError {
                span: None,
                module_name: None,
                error: WeslErrorInner::SymbolNotFound(vec),
            },
            CompilerPassError::MissingRequiredTemplateArgument(spanned, range) => WeslError {
                span: Some(range),
                module_name: None,
                error: WeslErrorInner::MissingRequiredTemplateArgument(spanned.value),
            },
            CompilerPassError::InternalError(internal_compiler_error) => WeslError {
                span: None,
                module_name: None,
                error: WeslErrorInner::InternalError(internal_compiler_error),
            },
            CompilerPassError::MalformedTemplateArgument(range) => WeslError {
                span: Some(range),
                module_name: None,
                error: WeslErrorInner::MalformedTemplateArgument,
            },
            CompilerPassError::ParseError(parse_err, span) => WeslError {
                span: Some(span),
                module_name: None,
                error: WeslErrorInner::ParseError(parse_err),
            },
        }
    }
}

impl WeslApi {
    pub fn remove_module(&mut self, module_name: &String) -> Result {
        let prev_len = self.translation_unit.global_declarations.len();
        self.translation_unit.global_declarations.retain(|x| {
            !matches!(x.value, GlobalDeclaration::Module(_))
                || x.name().as_ref().map(|x| &x.value) != Some(module_name)
        });

        if prev_len < self.translation_unit.global_declarations.len() {
            Ok(())
        } else {
            Err(WeslError {
                span: None,
                module_name: Some(module_name.clone()),
                error: WeslErrorInner::ModuleNotFound,
            })
        }
    }

    pub fn add_module(&mut self, module: ModuleDescriptor<'_>) -> Result {
        match module.source {
            Source::Ast(translation_unit) => {
                self.translation_unit
                    .global_declarations
                    .append(&mut translation_unit.global_declarations);
                self.translation_unit
                    .global_directives
                    .append(&mut translation_unit.global_directives);
                Ok(())
            }
            Source::Text(text) => {
                let mut bundler = Bundler {
                    sources: vec![text],
                    enclosing_module_name: Some(module.module_name.to_string()),
                };
                bundler.apply_mut(&mut self.translation_unit)?;
                Ok(())
            }
        }
    }

    pub fn compile(&self, path: &Path) -> Result<String> {
        let path = match path {
            Path::Parsed(path) => path.clone(),
            Path::Text(path) => {
                wesl_parse::Parser::parse_path(path)
                    .map_err(|err| CompilerPassError::ParseError(format!("{}", err), err.span()))?
                    .path
                    .value
            }
        };

        let mut resolver = wesl_resolve::Resolver;
        let mut result = resolver.apply(&self.translation_unit)?;

        let mut inliner = wesl_inline::Inliner;
        inliner.apply_mut(&mut result)?;

        let mut normalizer = wesl_template_normalize::TemplateNormalizer;
        normalizer.apply_mut(&mut result)?;

        let mut specializer = wesl_specialize::Specializer {
            entrypoint: Some(path),
        };

        specializer.apply_mut(&mut result)?;

        let mut dealiaser = wesl_dealias::Dealiaser;

        dealiaser.apply_mut(&mut result)?;

        let mut mangler = wesl_mangle::Mangler;

        mangler.apply_mut(&mut result)?;

        let mut flattener = wesl_flatten::Flattener;
        flattener.apply_mut(&mut result)?;

        Ok(format!("{result}"))
    }

    pub fn format_error(&self, _: WeslError) -> String {
        todo!();
    }
}
