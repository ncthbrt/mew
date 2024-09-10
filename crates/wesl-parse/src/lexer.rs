//! Prefer using [`Parser::parse_str`]. You shouldn't need to manipulate the lexer.

use crate::{error::ParseError, parser::Parser, span::Span};
use logos::{Logos, SpannedIter};
use std::{fmt::Display, num::NonZeroU8};

fn maybe_template_end(
    lex: &mut logos::Lexer<Token>,
    current: Token,
    lookahead: Option<Token>,
) -> Token {
    if let Some(depth) = lex.extras.template_depths.last() {
        if lex.extras.depth == *depth {
            // found a ">" on the same nesting level as the opening "<"
            lex.extras.template_depths.pop();
            lex.extras.lookahead = lookahead;
            return Token::TemplateArgsEnd;
        }
    }

    current
}

// operators && and || have lower precedence than < and >.
// therefore, this is not a template: a < b || c > d
fn maybe_fail_template(lex: &mut logos::Lexer<Token>) -> bool {
    if let Some(depth) = lex.extras.template_depths.last() {
        if lex.extras.depth == *depth {
            return false;
        }
    }
    true
}

fn incr_depth(lex: &mut logos::Lexer<Token>) {
    lex.extras.depth += 1;
}

fn decr_depth(lex: &mut logos::Lexer<Token>) {
    lex.extras.depth -= 1;
}

// don't have to be super strict, the lexer regex already did the heavy lifting
const DEC_FORMAT: u128 = lexical::NumberFormatBuilder::new().build();

// don't have to be super strict, the lexer regex already did the heavy lifting
const HEX_FORMAT: u128 = lexical::NumberFormatBuilder::new()
    .mantissa_radix(16)
    .base_prefix(NonZeroU8::new(b'x'))
    .exponent_base(NonZeroU8::new(16))
    .exponent_radix(NonZeroU8::new(10))
    .build();

fn parse_dec_abstract_int(lex: &mut logos::Lexer<Token>) -> Option<i64> {
    let options = &lexical::parse_integer_options::STANDARD;
    let str = lex.slice();
    lexical::parse_with_options::<i64, _, DEC_FORMAT>(str, options).ok()
}

fn parse_hex_abstract_int(lex: &mut logos::Lexer<Token>) -> Option<i64> {
    let options = &lexical::parse_integer_options::STANDARD;
    let str = lex.slice();
    lexical::parse_with_options::<i64, _, HEX_FORMAT>(str, options).ok()
}

fn parse_dec_i32(lex: &mut logos::Lexer<Token>) -> Option<i32> {
    let options = &lexical::parse_integer_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<i32, _, DEC_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

fn parse_hex_i32(lex: &mut logos::Lexer<Token>) -> Option<i32> {
    let options = &lexical::parse_integer_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<i32, _, HEX_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

fn parse_dec_u32(lex: &mut logos::Lexer<Token>) -> Option<u32> {
    let options = &lexical::parse_integer_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<u32, _, DEC_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

fn parse_hex_u32(lex: &mut logos::Lexer<Token>) -> Option<u32> {
    let options = &lexical::parse_integer_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<u32, _, HEX_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

fn parse_dec_abs_float(lex: &mut logos::Lexer<Token>) -> Option<f64> {
    let options = &lexical::parse_float_options::STANDARD;
    let str = lex.slice();
    lexical::parse_with_options::<f64, _, DEC_FORMAT>(str, options).ok()
}

fn parse_hex_abs_float(lex: &mut logos::Lexer<Token>) -> Option<f64> {
    let options = &lexical::parse_float_options::STANDARD;
    let str = lex.slice();
    lexical::parse_with_options::<f64, _, HEX_FORMAT>(str, options).ok()
}

fn parse_dec_f32(lex: &mut logos::Lexer<Token>) -> Option<f32> {
    let options = &lexical::parse_float_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<f32, _, DEC_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

fn parse_hex_f32(lex: &mut logos::Lexer<Token>) -> Option<f32> {
    let options = &lexical::parse_float_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<f32, _, HEX_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

fn parse_dec_f16(lex: &mut logos::Lexer<Token>) -> Option<f32> {
    let options = &lexical::parse_float_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<f32, _, DEC_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

fn parse_hex_f16(lex: &mut logos::Lexer<Token>) -> Option<f32> {
    let options = &lexical::parse_float_options::STANDARD;
    let str = lex.slice();
    lexical::parse_partial_with_options::<f32, _, HEX_FORMAT>(str, options)
        .ok()
        .map(|(x, _)| x)
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct LexerState {
    depth: i32,
    template_depths: Vec<i32>,
    lookahead: Option<Token>,
}

// follwing the spec at this date: https://www.w3.org/TR/2024/WD-WGSL-20240731/
#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(
    skip r"\s+",
    skip r"//[^\n\r]*[\n\r]*", // line comment
    skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", // block comment
    extras = LexerState,
    error = ParseError)]
pub enum Token {
    // syntactic tokens
    // https://www.w3.org/TR/WGSL/#syntactic-tokens
    #[token("&")]
    SymAnd,
    #[token("&&", maybe_fail_template)]
    SymAndAnd,
    #[token("->")]
    SymArrow,
    #[token("@")]
    SymAttr,
    #[token("/")]
    SymForwardSlash,
    #[token("!")]
    SymBang,
    #[token("[", incr_depth)]
    SymBracketLeft,
    #[token("]", decr_depth)]
    SymBracketRight,
    #[token("{")]
    SymBraceLeft,
    #[token("}")]
    SymBraceRight,
    // BEGIN WESL SYNTACTIC TOKENS
    #[token("::")]
    SymDoubleColon,
    #[token("..")]
    SymDoublePeriod,
    // END WESL SYNTACTIC TOKENS
    #[token(":")]
    SymColon,
    #[token(",")]
    SymComma,
    #[token("=")]
    SymEqual,
    #[token("==")]
    SymEqualEqual,
    #[token("!=")]
    SymNotEqual,
    #[token(">", |lex| maybe_template_end(lex, Token::SymGreaterThan, None))]
    SymGreaterThan,
    #[token(">=", |lex| maybe_template_end(lex, Token::SymGreaterThanEqual, Some(Token::SymEqual)))]
    SymGreaterThanEqual,
    #[token(">>", |lex| maybe_template_end(lex, Token::SymShiftRight, Some(Token::SymGreaterThan)))]
    SymShiftRight,
    #[token("<")]
    SymLessThan,
    #[token("<=")]
    SymLessThanEqual,
    #[token("<<")]
    SymShiftLeft,
    #[token("%")]
    SymModulo,
    #[token("-")]
    SymMinus,
    #[token("--")]
    SymMinusMinus,
    #[token(".")]
    SymPeriod,
    #[token("+")]
    SymPlus,
    #[token("++")]
    SymPlusPlus,
    #[token("|")]
    SymOr,
    #[token("||", maybe_fail_template)]
    SymOrOr,
    #[token("(", incr_depth)]
    SymParenLeft,
    #[token(")", decr_depth)]
    SymParenRight,
    #[token(";")]
    SymSemicolon,
    #[token("*")]
    SymStar,
    #[token("~")]
    SymTilde,
    #[token("_")]
    SymUnderscore,
    #[token("^")]
    SymXor,
    #[token("+=")]
    SymPlusEqual,
    #[token("-=")]
    SymMinusEqual,
    #[token("*=")]
    SymTimesEqual,
    #[token("/=")]
    SymDivisionEqual,
    #[token("%=")]
    SymModuloEqual,
    #[token("&=")]
    SymAndEqual,
    #[token("|=")]
    SymOrEqual,
    #[token("^=")]
    SymXorEqual,
    #[token(">>=", |lex| maybe_template_end(lex, Token::SymShiftRightAssign, Some(Token::SymGreaterThanEqual)))]
    SymShiftRightAssign,
    #[token("<<=")]
    SymShiftLeftAssign,

    // keywords
    // https://www.w3.org/TR/WGSL/#keyword-summary
    #[token("alias")]
    KwAlias,
    #[token("break")]
    KwBreak,
    #[token("case")]
    KwCase,
    #[token("const")]
    KwConst,
    #[token("const_assert")]
    KwConstAssert,
    #[token("continue")]
    KwContinue,
    #[token("continuing")]
    KwContinuing,
    #[token("default")]
    KwDefault,
    #[token("diagnostic")]
    KwDiagnostic,
    #[token("discard")]
    KwDiscard,
    #[token("else")]
    KwElse,
    #[token("enable")]
    KwEnable,
    #[token("false")]
    KwFalse,
    #[token("fn")]
    KwFn,
    #[token("for")]
    KwFor,
    #[token("if")]
    KwIf,
    #[token("let")]
    KwLet,
    #[token("loop")]
    KwLoop,
    #[token("override")]
    KwOverride,
    #[token("requires")]
    KwRequires,
    #[token("return")]
    KwReturn,
    #[token("struct")]
    KwStruct,
    #[token("switch")]
    KwSwitch,
    #[token("true")]
    KwTrue,
    #[token("var")]
    KwVar,
    #[token("while")]
    KwWhile,

    // BEGIN WESL KEYWORDS
    #[token("load")]
    KwLoad,
    #[token("mod")]
    KwMod,
    #[token("include")]
    KwInclude,
    #[token("sig")]
    KwSig,
    // END WESL KEYWORDS

    // XXX: should we also register reserved words as tokens?
    #[regex(r#"([_\p{XID_Start}][\p{XID_Continue}]+)|([\p{XID_Start}])"#, |lex| lex.slice().to_string())]
    Ident(String),
    #[regex(r#"0|[1-9][0-9]*"#, parse_dec_abstract_int)]
    #[regex(r#"0[xX][0-9a-fA-F]+"#, parse_hex_abstract_int)]
    AbstractInt(i64),
    #[regex(r#"(\d+\.\d*|\.\d+)([eE][+-]?\d+)?"#, parse_dec_abs_float)]
    #[regex(r#"\d+[eE][+-]?\d+"#, parse_dec_abs_float)]
    #[regex(r#"0[xX][\da-fA-F]*\.[\da-fA-F]*([pP][+-]?\d+)?"#, parse_hex_abs_float)]
    #[regex(r#"0[xX]\.[\da-fA-F]+([pP][+-]?\d+)?"#, parse_hex_abs_float)]
    #[regex(r#"0[xX][\da-fA-F]+[pP][+-]?\d+"#, parse_hex_abs_float)]
    // hex
    AbstractFloat(f64),
    #[regex(r#"(0|[1-9][0-9]*)i"#, parse_dec_i32)]
    #[regex(r#"0[xX][0-9a-fA-F]+i"#, parse_hex_i32)]
    // hex
    I32(i32),
    #[regex(r#"(0|[1-9][0-9]*)u"#, parse_dec_u32)]
    #[regex(r#"0[xX][0-9a-fA-F]+u"#, parse_hex_u32)]
    // hex
    U32(u32),
    #[regex(r#"(\d+\.\d*|\.\d+)([eE][+-]?\d+)?f"#, parse_dec_f32)]
    #[regex(r#"\d+([eE][+-]?\d+)?f"#, parse_dec_f32)]
    #[regex(r#"0[xX][\da-fA-F]*\.[\da-fA-F]*[pP][+-]?\d+f"#, parse_hex_f32)]
    #[regex(r#"0[xX]\.[\da-fA-F]+[pP][+-]?\d+f"#, parse_hex_f32)]
    #[regex(r#"0[xX][\da-fA-F]+[pP][+-]?\d+f"#, parse_hex_f32)]
    F32(f32),
    #[regex(r#"(\d+\.\d*|\.\d+)([eE][+-]?\d+)?h"#, parse_dec_f16)]
    #[regex(r#"\d+([eE][+-]?\d+)?h"#, parse_dec_f16)]
    #[regex(r#"0[xX][\da-fA-F]*\.[\da-fA-F]*[pP][+-]?\d+h"#, parse_hex_f16)]
    #[regex(r#"0[xX]\.[\da-fA-F]+[pP][+-]?\d+h"#, parse_hex_f16)]
    #[regex(r#"0[xX][\da-fA-F]+[pP][+-]?\d+h"#, parse_hex_f16)]
    F16(f32),
    TemplateArgsStart,
    TemplateArgsEnd,
}

impl Token {
    #[allow(unused)]
    pub fn is_symbol(&self) -> bool {
        matches!(
            self,
            Token::SymAnd
                | Token::SymAndAnd
                | Token::SymArrow
                | Token::SymAttr
                | Token::SymForwardSlash
                | Token::SymBang
                | Token::SymBracketLeft
                | Token::SymBracketRight
                | Token::SymBraceLeft
                | Token::SymBraceRight
                | Token::SymColon
                | Token::SymComma
                | Token::SymEqual
                | Token::SymEqualEqual
                | Token::SymNotEqual
                | Token::SymGreaterThan
                | Token::SymGreaterThanEqual
                | Token::SymShiftRight
                | Token::SymLessThan
                | Token::SymLessThanEqual
                | Token::SymShiftLeft
                | Token::SymModulo
                | Token::SymMinus
                | Token::SymMinusMinus
                | Token::SymPeriod
                | Token::SymPlus
                | Token::SymPlusPlus
                | Token::SymOr
                | Token::SymOrOr
                | Token::SymParenLeft
                | Token::SymParenRight
                | Token::SymSemicolon
                | Token::SymStar
                | Token::SymTilde
                | Token::SymUnderscore
                | Token::SymXor
                | Token::SymPlusEqual
                | Token::SymMinusEqual
                | Token::SymTimesEqual
                | Token::SymDivisionEqual
                | Token::SymModuloEqual
                | Token::SymAndEqual
                | Token::SymOrEqual
                | Token::SymXorEqual
                | Token::SymShiftRightAssign
                | Token::SymShiftLeftAssign
        )
    }

    #[allow(unused)]
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::KwAlias
                | Token::KwBreak
                | Token::KwCase
                | Token::KwConst
                | Token::KwConstAssert
                | Token::KwContinue
                | Token::KwContinuing
                | Token::KwDefault
                | Token::KwDiagnostic
                | Token::KwDiscard
                | Token::KwElse
                | Token::KwEnable
                | Token::KwFalse
                | Token::KwFn
                | Token::KwFor
                | Token::KwIf
                | Token::KwLet
                | Token::KwLoop
                | Token::KwOverride
                | Token::KwRequires
                | Token::KwReturn
                | Token::KwStruct
                | Token::KwSwitch
                | Token::KwTrue
                | Token::KwVar
                | Token::KwWhile
                | Token::KwLoad
                | Token::KwMod
                | Token::KwInclude
                | Token::KwSig
        )
    }

    #[allow(unused)]
    pub fn is_numeric_literal(&self) -> bool {
        matches!(
            self,
            Token::AbstractInt(_)
                | Token::AbstractFloat(_)
                | Token::I32(_)
                | Token::U32(_)
                | Token::F32(_)
                | Token::F16(_)
        )
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::SymAnd => f.write_str("&"),
            Token::SymAndAnd => f.write_str("&&"),
            Token::SymArrow => f.write_str("->"),
            Token::SymAttr => f.write_str("@"),
            Token::SymForwardSlash => f.write_str("/"),
            Token::SymBang => f.write_str("!"),
            Token::SymBracketLeft => f.write_str("["),
            Token::SymBracketRight => f.write_str("]"),
            Token::SymBraceLeft => f.write_str("{"),
            Token::SymBraceRight => f.write_str("}"),
            Token::SymColon => f.write_str(":"),
            Token::SymComma => f.write_str(","),
            Token::SymEqual => f.write_str("="),
            Token::SymEqualEqual => f.write_str("=="),
            Token::SymNotEqual => f.write_str("!="),
            Token::SymGreaterThan => f.write_str(">"),
            Token::SymGreaterThanEqual => f.write_str(">="),
            Token::SymShiftRight => f.write_str(">>"),
            Token::SymLessThan => f.write_str("<"),
            Token::SymLessThanEqual => f.write_str("<="),
            Token::SymShiftLeft => f.write_str("<<"),
            Token::SymModulo => f.write_str("%"),
            Token::SymMinus => f.write_str("-"),
            Token::SymMinusMinus => f.write_str("--"),
            Token::SymPeriod => f.write_str("."),
            Token::SymPlus => f.write_str("+"),
            Token::SymPlusPlus => f.write_str("++"),
            Token::SymOr => f.write_str("|"),
            Token::SymOrOr => f.write_str("||"),
            Token::SymParenLeft => f.write_str("("),
            Token::SymParenRight => f.write_str(")"),
            Token::SymSemicolon => f.write_str(";"),
            Token::SymStar => f.write_str("*"),
            Token::SymTilde => f.write_str("~"),
            Token::SymUnderscore => f.write_str("_"),
            Token::SymXor => f.write_str("^"),
            Token::SymPlusEqual => f.write_str("+="),
            Token::SymMinusEqual => f.write_str("-="),
            Token::SymTimesEqual => f.write_str("*="),
            Token::SymDivisionEqual => f.write_str("/="),
            Token::SymModuloEqual => f.write_str("%="),
            Token::SymAndEqual => f.write_str("&="),
            Token::SymOrEqual => f.write_str("|="),
            Token::SymXorEqual => f.write_str("^="),
            Token::SymShiftRightAssign => f.write_str(">>="),
            Token::SymShiftLeftAssign => f.write_str("<<="),
            Token::KwAlias => f.write_str("alias"),
            Token::KwBreak => f.write_str("break"),
            Token::KwCase => f.write_str("case"),
            Token::KwConst => f.write_str("const"),
            Token::KwConstAssert => f.write_str("const_assert"),
            Token::KwContinue => f.write_str("continue"),
            Token::KwContinuing => f.write_str("continuing"),
            Token::KwDefault => f.write_str("default"),
            Token::KwDiagnostic => f.write_str("diagnostic"),
            Token::KwDiscard => f.write_str("discard"),
            Token::KwElse => f.write_str("else"),
            Token::KwEnable => f.write_str("enable"),
            Token::KwFalse => f.write_str("false"),
            Token::KwFn => f.write_str("fn"),
            Token::KwFor => f.write_str("for"),
            Token::KwIf => f.write_str("if"),
            Token::KwLet => f.write_str("let"),
            Token::KwLoop => f.write_str("loop"),
            Token::KwOverride => f.write_str("override"),
            Token::KwRequires => f.write_str("requires"),
            Token::KwReturn => f.write_str("return"),
            Token::KwStruct => f.write_str("struct"),
            Token::KwSwitch => f.write_str("switch"),
            Token::KwTrue => f.write_str("true"),
            Token::KwVar => f.write_str("var"),
            Token::KwWhile => f.write_str("while"),
            Token::KwLoad => f.write_str("load"),
            Token::KwMod => f.write_str("mod"),
            Token::KwInclude => f.write_str("include"),
            Token::KwSig => f.write_str("sig"),
            Token::Ident(s) => write!(f, "identifier `{s}`"),
            Token::AbstractInt(n) => write!(f, "{n}"),
            Token::AbstractFloat(n) => write!(f, "{n}"),
            Token::I32(n) => write!(f, "{n}i"),
            Token::U32(n) => write!(f, "{n}u"),
            Token::F32(n) => write!(f, "{n}f"),
            Token::F16(n) => write!(f, "{n}h"),
            Token::TemplateArgsStart => f.write_str("<"),
            Token::TemplateArgsEnd => f.write_str(">"),
            Token::SymDoubleColon => f.write_str("::"),
            Token::SymDoublePeriod => f.write_str(".."),
        }
    }
}

pub type Spanned<Tok, Loc, ParseError> = Result<(Loc, Tok, Loc), (Loc, ParseError, Loc)>;

#[derive(Clone)]
pub struct Lexer<'s> {
    source: &'s str,
    token_stream: SpannedIter<'s, Token>,
    next_token: Option<(Result<Token, ParseError>, Span)>,
    parsing_template: bool,
    opened_templates: u32,
}

impl<'s> Lexer<'s> {
    pub fn new(source: &'s str) -> Self {
        let mut token_stream = Token::lexer_with_extras(source, LexerState::default()).spanned();
        let next_token = token_stream.next();
        Self {
            source,
            token_stream,
            next_token,
            parsing_template: false,
            opened_templates: 0,
        }
    }

    pub fn source(&self) -> &str {
        self.source
    }
}

/// Returns `true` if the source starts with a valid template list.
///
/// # Bugs
///
/// There might be a bug in the spec algorithm, which rejects the following:
/// `assert_eq!(recognize_template_list("<X<Y<Z>>>"), false);`
/// This implementation doesn't.
///
/// ## Examples
///
/// ```rust
/// # use wesl_parse::lexer::recognize_template_list;
/// // examples from the spec:
/// assert_eq!(recognize_template_list("<i32,select(2,3,a>b)>"), true);
/// assert_eq!(recognize_template_list("<d]>"), false);
/// assert_eq!(recognize_template_list("<B<<C>"), true);
/// assert_eq!(recognize_template_list("<B<=C>"), true);
/// assert_eq!(recognize_template_list("<(B>=C)>"), true);
/// assert_eq!(recognize_template_list("<(B!=C)>"), true);
/// assert_eq!(recognize_template_list("<(B==C)>"), true);
///
/// // false cases
/// assert_eq!(recognize_template_list(""), false);
/// assert_eq!(recognize_template_list("<>"), false);
/// assert_eq!(recognize_template_list("<b || c>d"), false);
/// ```
///
/// ## Specification
///
/// [3.9. Template Lists](https://www.w3.org/TR/WGSL/#template-lists-sec)
///
/// Contrary to the specification [template list discovery algorithm], this function also
/// checks that the template is syntactically valid (syntax: [*template_list*]).
///
/// [template list discovery algorigthm]: https://www.w3.org/TR/WGSL/#template-list-discovery
/// [*template_list*]: https://www.w3.org/TR/WGSL/#syntax-template_list
pub fn recognize_template_list(source: &str) -> bool {
    let mut lexer = Lexer::new(source);
    match lexer.next_token {
        Some((Ok(ref mut t), _)) if *t == Token::SymLessThan => *t = Token::TemplateArgsStart,
        _ => return false,
    };
    lexer.parsing_template = true;
    lexer.opened_templates = 1;
    lexer.token_stream.extras.template_depths.push(0);
    let parse = Parser::recognize_template_list(&mut lexer);
    parse.is_ok()
}

impl<'s> Iterator for Lexer<'s> {
    type Item = Spanned<Token, usize, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_token = &self.next_token;

        let lookahead = self.token_stream.extras.lookahead.take();
        let mut next_token = match lookahead {
            Some(next_token) => {
                let (_, span) = cur_token.as_ref().unwrap(); // safety: lookahead implies lexer looked at a token
                let span = (span.start + 1)..span.end;
                Some((Ok(next_token), span))
            }
            None => self.token_stream.next(),
        };

        if let (Some((Ok(cur_tok), _)), Some((Ok(next_tok), span))) = (cur_token, &mut next_token) {
            if (matches!(cur_tok, Token::Ident(_)) || cur_tok.is_keyword())
                && *next_tok == Token::SymLessThan
            {
                let source = &self.source[span.start..];
                if recognize_template_list(source) {
                    *next_tok = Token::TemplateArgsStart;
                    let cur_depth = self.token_stream.extras.depth;
                    self.token_stream.extras.template_depths.push(cur_depth);
                    self.opened_templates += 1;
                }
            }
        }

        if self.parsing_template && matches!(cur_token, Some((Ok(Token::TemplateArgsEnd), _))) {
            self.opened_templates -= 1;
            if self.opened_templates == 0 {
                next_token = None; // push eof after end of template
            }
        }

        std::mem::swap(&mut self.next_token, &mut next_token);

        next_token.map(|(token, span)| match token {
            Ok(tok) => Ok((span.start, tok, span.end)),
            Err(err) => Err((span.start, err, span.end)),
        })
    }
}
