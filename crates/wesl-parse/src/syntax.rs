//! A syntax tree for WGSL files. The root of the tree is a [`TranslationUnit`].
//!
//! Follwing the spec at this date:
//! [2024-07-31](https://www.w3.org/TR/2024/WD-WGSL-20240731/).
//! The syntax tree closely mirrors wgsl structure while allowing language extensions.
//!
//! ## Strictness
//!
//! This syntax tree is rather strict, meaning it cannot represent most syntaxically
//! incorrect programs. But it is only syntactic, meaning it doesn't perform many
//! contextual checks: for example, certain attributes can only appear in certain places,
//! or declarations have different constraints depending on where they appear.
//! stricter checking is TODO and will be optional.
//!
//! ## Extensions
//!
//! TODO, the syntax tree can be mutated to allow well-defined language extensions with
//! feature flags (wgsl-tooling-imports, wgsl-tooling-generics, ...).
//!
//! ## Design considerations
//!
//! The parsing is not designed to be primarily efficient, but flexible and correct.
//! It is made with the ultimate goal to implement spec-compliant language extensions.
//! This is why this parser doesn't borrow strings.

use std::{hash::Hash, ops::Deref};

use crate::span::S;

pub struct WithSource<'s, T> {
    syntax: T,
    source: &'s str,
}

impl<'s, T> WithSource<'s, T> {
    fn new(syntax: T, source: &'s str) -> Self {
        Self { syntax, source }
    }
    pub fn source(&self) -> &str {
        self.source
    }
}

impl<'s, T> Deref for WithSource<'s, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.syntax
    }
}

impl<'s, T> AsRef<T> for WithSource<'s, T> {
    fn as_ref(&self) -> &T {
        &self.syntax
    }
}

pub trait SpannedSyntax {
    fn with_source<'s>(&self, source: &'s str) -> WithSource<'s, &Self> {
        WithSource::new(self, source)
    }
}

impl SpannedSyntax for TranslationUnit {}
impl SpannedSyntax for GlobalDirective {}
impl SpannedSyntax for DiagnosticDirective {}
impl SpannedSyntax for EnableDirective {}
impl SpannedSyntax for RequiresDirective {}
impl SpannedSyntax for GlobalDeclaration {}
impl SpannedSyntax for Declaration {}
impl SpannedSyntax for Alias {}
impl SpannedSyntax for Struct {}
impl SpannedSyntax for StructMember {}
impl SpannedSyntax for Function {}
impl SpannedSyntax for FormalParameter {}
impl SpannedSyntax for ConstAssert {}
impl SpannedSyntax for Attribute {}
impl SpannedSyntax for Expression {}
impl SpannedSyntax for NamedComponentExpression {}
impl SpannedSyntax for IndexingExpression {}
impl SpannedSyntax for UnaryExpression {}
impl SpannedSyntax for BinaryExpression {}
impl SpannedSyntax for FunctionCallExpression {}
impl SpannedSyntax for TypeExpression {}
impl SpannedSyntax for Statement {}
impl SpannedSyntax for CompoundStatement {}
impl SpannedSyntax for AssignmentStatement {}
impl SpannedSyntax for AssignmentOperator {}
impl SpannedSyntax for IfStatement {}
impl SpannedSyntax for SwitchStatement {}
impl SpannedSyntax for SwitchClause {}
impl SpannedSyntax for CaseSelector {}
impl SpannedSyntax for LoopStatement {}
impl SpannedSyntax for ContinuingStatement {}
impl SpannedSyntax for ForStatement {}
impl SpannedSyntax for WhileStatement {}
impl SpannedSyntax for IdentifierExpression {}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TranslationUnit {
    pub global_directives: Vec<S<GlobalDirective>>,
    pub global_declarations: Vec<S<GlobalDeclaration>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GlobalDirective {
    Diagnostic(DiagnosticDirective),
    Enable(EnableDirective),
    Requires(RequiresDirective),
    Use(UseDirective),
    Extend(ExtendDirective),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleDirective {
    Use(UseDirective),
    Extend(ExtendDirective),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExtendDirective {
    pub attributes: Vec<S<Attribute>>,
    pub path: S<Vec<PathPart>>,
}

type UseDirective = Use;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticDirective {
    pub severity: S<DiagnosticSeverity>,
    pub rule_name: S<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Off,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnableDirective {
    pub extensions: Vec<S<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RequiresDirective {
    pub extensions: Vec<S<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GlobalDeclaration {
    Void,
    Declaration(Declaration),
    Alias(Alias),
    Struct(Struct),
    Function(Function),
    ConstAssert(ConstAssert),
    Module(Module),
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
pub struct Module {
    pub attributes: Vec<S<Attribute>>,
    pub name: S<String>,
    pub directives: Vec<S<ModuleDirective>>,
    pub members: Vec<S<ModuleMemberDeclaration>>,
    pub template_parameters: Vec<S<FormalTemplateParameter>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
pub struct FormalTemplateParameter {
    pub name: S<String>,
    pub default_value: Option<S<Expression>>,
}

pub struct TemplateElaboratedIdent {
    pub path: S<Vec<TemplateElaboratedIdentPart>>,
}

pub struct TemplateElaboratedIdentPart {
    pub name: S<String>,
    pub template_args: Option<Vec<S<TemplateArg>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleMemberDeclaration {
    Void,
    Declaration(Declaration),
    Alias(Alias),
    Struct(Struct),
    Function(Function),
    ConstAssert(ConstAssert),
    Module(Module),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Declaration {
    pub attributes: Vec<S<Attribute>>,
    pub kind: S<DeclarationKind>,
    pub template_args: Option<Vec<S<TemplateArg>>>,
    pub name: S<String>,
    pub typ: Option<S<TypeExpression>>,
    pub initializer: Option<S<Expression>>,
    pub template_parameters: Vec<S<FormalTemplateParameter>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeclarationKind {
    Const,
    Override,
    Let,
    Var,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Alias {
    pub name: S<String>,
    pub typ: S<TypeExpression>,
    pub template_parameters: Vec<S<FormalTemplateParameter>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Struct {
    pub name: S<String>,
    pub members: Vec<S<StructMember>>,
    pub template_parameters: Vec<S<FormalTemplateParameter>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructMember {
    pub attributes: Vec<S<Attribute>>,
    pub name: S<String>,
    pub typ: S<TypeExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CompoundDirective {
    Use(UseDirective),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Function {
    pub attributes: Vec<S<Attribute>>,
    pub name: S<String>,
    pub parameters: Vec<S<FormalParameter>>,
    pub return_attributes: Vec<S<Attribute>>,
    pub return_type: Option<S<TypeExpression>>,
    pub body: S<CompoundStatement>,
    pub template_parameters: Vec<S<FormalTemplateParameter>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormalParameter {
    pub attributes: Vec<S<Attribute>>,
    pub name: S<String>,
    pub typ: S<TypeExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstAssert {
    pub expression: S<Expression>,
    pub template_parameters: Vec<S<FormalTemplateParameter>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub name: S<String>,
    pub arguments: Option<Vec<S<Expression>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    Literal(S<LiteralExpression>),
    Parenthesized(ParenthesizedExpression),
    NamedComponent(NamedComponentExpression),
    Indexing(IndexingExpression),
    Unary(UnaryExpression),
    Binary(BinaryExpression),
    FunctionCall(FunctionCallExpression),
    Identifier(IdentifierExpression),
    Type(TypeExpression),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiteralExpression {
    True,
    False,
    AbstractInt(String),
    AbstractFloat(String),
    I32(i32),
    U32(u32),
    F32(String),
    F16(String),
}

pub type ParenthesizedExpression = Box<S<Expression>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedComponentExpression {
    pub base: Box<S<Expression>>,
    pub component: S<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IndexingExpression {
    pub base: Box<S<Expression>>,
    pub index: Box<S<Expression>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnaryExpression {
    pub operator: S<UnaryOperator>,
    pub operand: Box<S<Expression>>, // TODO maybe rename rhs
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    LogicalNegation,
    Negation,
    BitwiseComplement,
    AddressOf,
    Indirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BinaryExpression {
    pub operator: S<BinaryOperator>,
    pub left: Box<S<Expression>>, // TODO: rename lhs rhs
    pub right: Box<S<Expression>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    ShortCircuitOr,
    ShortCircuitAnd,
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Remainder,
    Equality,
    Inequality,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionCallExpression {
    pub path: S<Vec<PathPart>>,
    pub arguments: Vec<S<Expression>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PathPart {
    pub name: S<String>,
    pub template_args: Option<Vec<S<TemplateArg>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdentifierExpression {
    pub path: S<Vec<PathPart>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeExpression {
    pub path: S<Vec<PathPart>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TemplateArg {
    pub expression: S<Expression>,
    pub arg_name: Option<S<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Statement {
    Void,
    Compound(CompoundStatement),
    Assignment(AssignmentStatement),
    Increment(IncrementStatement),
    Decrement(DecrementStatement),
    If(IfStatement),
    Switch(SwitchStatement),
    Loop(LoopStatement),
    For(ForStatement),
    While(WhileStatement),
    Break,
    Continue,
    Return(ReturnStatement),
    Discard,
    FunctionCall(FunctionCallStatement),
    ConstAssert(ConstAssertStatement),
    Declaration(DeclarationStatement),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompoundStatement {
    pub attributes: Vec<S<Attribute>>,
    pub directives: Vec<S<CompoundDirective>>,
    pub statements: Vec<S<Statement>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssignmentStatement {
    pub operator: S<AssignmentOperator>,
    pub lhs: S<Expression>,
    pub rhs: S<Expression>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AssignmentOperator {
    Equal,
    PlusEqual,
    MinusEqual,
    TimesEqual,
    DivisionEqual,
    ModuloEqual,
    AndEqual,
    OrEqual,
    XorEqual,
    ShiftRightAssign,
    ShiftLeftAssign,
}

pub type IncrementStatement = Expression;

pub type DecrementStatement = Expression;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IfStatement {
    pub attributes: Vec<S<Attribute>>,
    pub if_clause: (S<Expression>, S<CompoundStatement>),
    pub else_if_clauses: Vec<(S<Expression>, S<CompoundStatement>)>,
    pub else_clause: Option<S<CompoundStatement>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SwitchStatement {
    pub attributes: Vec<S<Attribute>>,
    pub expression: S<Expression>,
    pub body_attributes: Vec<S<Attribute>>,
    pub clauses: Vec<S<SwitchClause>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SwitchClause {
    pub case_selectors: Vec<S<CaseSelector>>,
    pub body: S<CompoundStatement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CaseSelector {
    Default,
    Expression(Expression),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LoopStatement {
    pub attributes: Vec<S<Attribute>>,
    pub body: S<CompoundStatement>,
    // a ContinuingStatement can only appear inside a LoopStatement body, therefore it is
    // not part of the Statement enum. it appears here instead, but consider it part of
    // body as the last statement of the CompoundStatement.
    pub continuing: Option<S<ContinuingStatement>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContinuingStatement {
    pub body: S<CompoundStatement>,
    // a BreakIfStatement can only appear inside a ContinuingStatement body, therefore it
    // not part of the Statement enum. it appears here instead, but consider it part of
    // body as the last statement of the CompoundStatement.
    pub break_if: Option<S<BreakIfStatement>>,
}

pub type BreakIfStatement = Expression;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ForStatement {
    pub attributes: Vec<S<Attribute>>,
    pub initializer: Option<Box<S<Statement>>>,
    pub condition: Option<S<Expression>>,
    pub update: Option<Box<S<Statement>>>,
    pub body: S<CompoundStatement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WhileStatement {
    pub attributes: Vec<S<Attribute>>,
    pub condition: S<Expression>,
    pub body: S<CompoundStatement>,
}

pub type ReturnStatement = Option<S<Expression>>;

pub type FunctionCallStatement = FunctionCallExpression;

pub type ConstAssertStatement = ConstAssert;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeclarationStatement {
    pub declaration: S<Declaration>,
    pub statements: Vec<S<Statement>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Use {
    pub attributes: Vec<S<Attribute>>,
    pub path: S<Vec<PathPart>>,
    pub content: S<UseContent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UseContent {
    Item(UseItem),
    Collection(Vec<S<Use>>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UseItem {
    pub name: S<String>,
    pub rename: Option<S<String>>,
    pub template_args: Option<Vec<S<TemplateArg>>>,
}
