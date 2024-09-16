//! The [`Parser`] takes WGSL source code and returns a [syntax tree].
//!
//! [syntax tree]: syntax

lalrpop_mod!(
    #[allow(clippy::type_complexity)]
    wgsl
);
lalrpop_mod!(
    #[allow(clippy::type_complexity)]
    wgsl_spanned
);
lalrpop_mod!(
    #[allow(clippy::type_complexity)]
    wgsl_recognize
);

use lalrpop_util::lalrpop_mod;

use crate::{error::SpannedError, lexer::Lexer, syntax, syntax_spanned};

#[derive(Debug, Default, Clone, Copy)]
pub struct Parser;

impl Parser {
    pub fn parse_str(source: &str) -> Result<syntax::TranslationUnit, SpannedError> {
        let lexer = Lexer::new(source);
        let parser = wgsl::TranslationUnitParser::new();
        let res = parser.parse(lexer);
        res.map_err(|e| SpannedError::new(e, source))
    }
    pub fn parse<'s>(
        mut lexer: &'s mut Lexer,
    ) -> Result<syntax::TranslationUnit, SpannedError<'s>> {
        let parser = wgsl::TranslationUnitParser::new();
        let res = parser.parse(&mut lexer);
        res.map_err(|e| SpannedError::new(e, lexer.source()))
    }
}
impl Parser {
    pub fn parse_str_spanned(
        source: &str,
    ) -> Result<syntax_spanned::TranslationUnit, SpannedError> {
        let lexer = Lexer::new(source);
        let parser = wgsl_spanned::TranslationUnitParser::new();
        let res = parser.parse(lexer);
        res.map_err(|e| SpannedError::new(e, source))
    }
    pub fn parse_spanned<'s>(
        mut lexer: &'s mut Lexer,
    ) -> Result<syntax_spanned::TranslationUnit, SpannedError<'s>> {
        let parser = wgsl_spanned::TranslationUnitParser::new();
        let res = parser.parse(&mut lexer);
        res.map_err(|e| SpannedError::new(e, lexer.source()))
    }
}

impl Parser {
    pub fn recognize_str(source: &str) -> Result<(), SpannedError> {
        let lexer = Lexer::new(source);
        let parser = wgsl_recognize::TranslationUnitParser::new();
        let res = parser.parse(lexer);
        res.map_err(|e| SpannedError::new(e, source))
    }
    pub fn recognize<'s>(mut lexer: &'s mut Lexer) -> Result<(), SpannedError<'s>> {
        let parser = wgsl_recognize::TranslationUnitParser::new();
        let res = parser.parse(&mut lexer);
        res.map_err(|e| SpannedError::new(e, lexer.source()))
    }
    pub fn recognize_template_list<'s>(mut lexer: &'s mut Lexer) -> Result<(), SpannedError<'s>> {
        let parser = wgsl_recognize::TryTemplateListParser::new();
        let res = parser.parse(&mut lexer);
        res.map_err(|e| SpannedError::new(e, lexer.source()))
    }
}
