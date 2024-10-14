use mew_bundle::Bundler;
use mew_parse::{
    span::{Span, Spanned},
    syntax::{
        Alias, FormalTemplateParameter, GlobalDeclaration, PathPart, TranslationUnit,
        TypeExpression,
    },
};
use mew_types::{mangling::mangle_path, CompilerPass, CompilerPassError, InternalCompilerError};

#[derive(Default, Debug)]
pub struct MewApi {
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
pub enum MewErrorInner {
    ModuleNotFound,
    SymbolNotFound(Vec<PathPart>),
    MissingRequiredTemplateArgument(FormalTemplateParameter),
    InternalError(InternalCompilerError),
    MalformedTemplateArgument,
    ParseError(String),
}

#[derive(Debug)]
pub struct MewError {
    pub span: Option<Span>,
    pub module_name: Option<String>,
    pub error: MewErrorInner,
}

pub enum Path {
    Parsed(Vec<PathPart>),
    Text(String),
}

type Result<T = ()> = std::result::Result<T, MewError>;

impl From<CompilerPassError> for MewError {
    fn from(value: CompilerPassError) -> Self {
        match value {
            CompilerPassError::SymbolNotFound(vec, range) => MewError {
                span: Some(range),
                module_name: None,
                error: MewErrorInner::SymbolNotFound(vec),
            },
            CompilerPassError::UnableToResolvePath(vec) => MewError {
                span: None,
                module_name: None,
                error: MewErrorInner::SymbolNotFound(vec),
            },
            CompilerPassError::MissingRequiredTemplateArgument(spanned, range) => MewError {
                span: Some(range),
                module_name: None,
                error: MewErrorInner::MissingRequiredTemplateArgument(spanned.value),
            },
            CompilerPassError::InternalError(internal_compiler_error) => MewError {
                span: None,
                module_name: None,
                error: MewErrorInner::InternalError(internal_compiler_error),
            },
            CompilerPassError::MalformedTemplateArgument(range) => MewError {
                span: Some(range),
                module_name: None,
                error: MewErrorInner::MalformedTemplateArgument,
            },
            CompilerPassError::ParseError(parse_err, span) => MewError {
                span: Some(span),
                module_name: None,
                error: MewErrorInner::ParseError(parse_err),
            },
        }
    }
}

impl MewApi {
    pub fn remove_module(&mut self, module_name: &String) -> Result {
        let prev_len = self.translation_unit.global_declarations.len();
        self.translation_unit.global_declarations.retain(|x| {
            !matches!(x.value, GlobalDeclaration::Module(_))
                || x.name().as_ref().map(|x| &x.value) != Some(module_name)
        });

        if prev_len < self.translation_unit.global_declarations.len() {
            Ok(())
        } else {
            Err(MewError {
                span: None,
                module_name: Some(module_name.clone()),
                error: MewErrorInner::ModuleNotFound,
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
                mew_parse::Parser::parse_path(path)
                    .map_err(|err| CompilerPassError::ParseError(format!("{}", err), err.span()))?
                    .path
                    .value
            }
        };

        let mut alias_name_path = path.clone();
        mangle_path(&mut alias_name_path);

        let mut resolver = mew_resolve::Resolver;
        let mut result = self.translation_unit.clone();

        let alias = Alias {
            name: Spanned::new(
                alias_name_path
                    .into_iter()
                    .map(|x| x.name.value)
                    .collect::<Vec<String>>()
                    .join("_"),
                0..0,
            ),
            typ: Spanned::new(
                TypeExpression {
                    path: Spanned::new(path, 0..0),
                },
                0..0,
            ),
            template_parameters: vec![],
        };

        let entry_path = vec![PathPart {
            name: alias.name.clone(),
            template_args: None,
            inline_template_args: None,
        }];

        result
            .global_declarations
            .push(Spanned::new(GlobalDeclaration::Alias(alias), 0..0));

        resolver.apply_mut(&mut result)?;

        let mut inliner = mew_inline::Inliner;
        inliner.apply_mut(&mut result)?;

        let mut normalizer = mew_template_normalize::TemplateNormalizer;
        normalizer.apply_mut(&mut result)?;

        let mut specializer = mew_specialize::Specializer {
            entrypoint: Some(entry_path),
        };

        specializer.apply_mut(&mut result)?;

        let mut dealiaser = mew_dealias::Dealiaser;

        dealiaser.apply_mut(&mut result)?;

        let mut mangler = mew_mangle::Mangler;

        mangler.apply_mut(&mut result)?;

        let mut flattener = mew_flatten::Flattener;
        flattener.apply_mut(&mut result)?;

        Ok(format!("{result}"))
    }

    // pub fn format_error(&self, _: MewError) -> String {
    //     todo!();
    // }
}
