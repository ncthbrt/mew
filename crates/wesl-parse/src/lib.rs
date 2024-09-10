//! A parser for WGSL files, written directly from the [specification] with [lalrpop].
//!
//! # Parsing a source file
//!
//! ```rust
//! # use wesl_parse::syntax::*;
//! let source = "@fragment fn frag_main() -> @location(0) vec4f { return vec4(1); }";
//! let parsed = wesl_parse::Parser::parse_str(source).unwrap();
//!
//! let compare = TranslationUnit {
//!     global_directives: vec![],
//!     global_declarations: vec![GlobalDeclaration::Function(Function {
//!         attributes: vec![Attribute {
//!             name: "fragment".to_string(),
//!             arguments: None
//!         }],
//!         name: "frag_main".to_string(),
//!         parameters: vec![],
//!         return_attributes: vec![Attribute {
//!             name: "location".to_string(),
//!             arguments: Some(vec![Expression::Literal(LiteralExpression::AbstractInt(0))])
//!         }],
//!         return_type: Some(TypeExpression {
//!             path: vec!["vec4f".to_string()],
//!             template_args: None
//!         }),
//!         body: CompoundStatement {
//!             attributes: vec![],
//!             statements: vec![Statement::Return(Some(Expression::FunctionCall(
//!                 FunctionCallExpression {
//!                     path: vec!["vec4".to_string()],
//!                     template_args: None,
//!                     arguments: vec![Expression::Literal(LiteralExpression::AbstractInt(1))]
//!                 }
//!             )))]
//!         }
//!     })]
//! };
//!
//! assert_eq!(parsed, compare);
//! ```
//!
//! # Syntax tree
//!
//! See [syntax tree].
//!
//! Modifying the syntax tree:
//! ```rust
//!     let source = "const hello = 0u;";
//!     let mut module = wesl_parse::Parser::parse_str(source).unwrap();
//!
//!     // modify the module as needed...
//!     let decl = &mut module
//!         .global_declarations
//!         .iter_mut()
//!         .find_map(|decl| match decl {
//!             wesl_parse::syntax::GlobalDeclaration::Declaration(decl) => Some(decl),
//!             _ => None,
//!         })
//!         .unwrap();
//!     decl.name = "world".to_string();
//!
//!     assert_eq!(format!("{module}").trim(), "const world = 0u;");
//! ```
//!
//! # Stringification
//!
//! The syntax tree elements implement [`Display`][std::fmt::Display].
//! The display is always pretty-printed.
//!
//! TODO: implement :# for pretty vs. inline formatting.
//!
//! ```rust
//! let source = "@fragment fn frag_main() -> @location(0) vec4f { return vec4(1); }";
//! let mut module = wesl_parse::Parser::parse_str(source).unwrap();
//!
//! // modify the module as needed...
//!
//! println!("{module}");
//! ```
//!
//! [lalrpop]: https://lalrpop.github.io/lalrpop/
//! [specification]: https://www.w3.org/TR/WGSL/
//! [syntax tree]: syntax
//! [spanned syntax tree]: syntax_spanned

pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod syntax;
pub mod syntax_spanned;

mod parser_support;
mod parser_support_spanned;
mod syntax_display;
mod syntax_display_spanned;
mod syntax_impl;

pub use lexer::Lexer;
pub use parser::Parser;

// pub fn parse_recognize(
//     source: &str,
// ) -> Result<(), ParseError<usize, Token, (usize, Error, usize)>> {
//     let lexer = Lexer::new(&source);
//     let parser = wgsl_recognize::TranslationUnitParser::new();
//     parser.parse(lexer)
// }

// pub fn parse_spanned(
//     source: &str,
// ) -> Result<TranslationUnit, ParseError<usize, Token, (usize, Error, usize)>> {
//     let lexer = Lexer::new(&source);
//     let parser = wgsl_spanned::TranslationUnitParser::new();
//     parser.parse(lexer)
// }
