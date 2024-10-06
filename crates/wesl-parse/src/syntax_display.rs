use crate::{span::S, syntax::*};
use core::fmt;
use std::fmt::{Display, Formatter, Write};

use itertools::Itertools;

struct Indent<T: Display>(pub T);

impl<T: Display> Display for Indent<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent = "    ";
        let inner_display = self.0.to_string();
        let fmt = inner_display
            .lines()
            .map(|l| format!("{}{}", indent, l))
            .format("\n");
        write!(f, "{}", fmt)?;
        Ok(())
    }
}

impl Display for TranslationUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let directives = self.global_directives.iter().format("\n");
        let declarations = self
            .global_declarations
            .iter()
            // .filter(|decl| !matches!(decl, GlobalDeclaration::Void))
            .format("\n\n");
        writeln!(f, "{directives}\n\n{declarations}")
    }
}

impl Display for GlobalDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GlobalDirective::Diagnostic(print) => write!(f, "{}", print),
            GlobalDirective::Enable(print) => write!(f, "{}", print),
            GlobalDirective::Requires(print) => write!(f, "{}", print),
            GlobalDirective::Use(print) if matches!(print.content.value, UseContent::Item(_)) => {
                write!(f, "use {};", print)
            }
            GlobalDirective::Use(print) => write!(f, "use {}", print),
            GlobalDirective::Extend(print) => write!(f, "{}", print),
        }
    }
}

impl Display for DiagnosticDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let severity = &self.severity;
        let rule = &self.rule_name;
        writeln!(f, "diagnostic ({severity}, {rule});")
    }
}

impl Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
            Self::Off => write!(f, "off"),
        }
    }
}

impl Display for EnableDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let exts = self.extensions.iter().format(", ");
        writeln!(f, "enable {exts};")
    }
}

impl Display for RequiresDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let exts = self.extensions.iter().format(", ");
        writeln!(f, "requires {exts};")
    }
}

impl Display for GlobalDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GlobalDeclaration::Void => write!(f, ";"),
            GlobalDeclaration::Declaration(print) => write!(f, "{}", print),
            GlobalDeclaration::Alias(print) => write!(f, "{}", print),
            GlobalDeclaration::Struct(print) => write!(f, "{}", print),
            GlobalDeclaration::Function(print) => write!(f, "{}", print),
            GlobalDeclaration::ConstAssert(print) => write!(f, "{}", print),
            GlobalDeclaration::Module(print) => write!(f, "{}", print),
        }
    }
}

impl Display for Declaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let kind = &self.kind;
        let tplt_args = fmt_template_args(&self.template_args);
        let tplt_params = fmt_template_params(&self.template_parameters);
        let name = &self.name;
        let typ = self
            .typ
            .as_ref()
            .map(|typ| format!(": {}", typ))
            .unwrap_or_default();
        let init = self
            .initializer
            .as_ref()
            .map(|typ| format!(" = {}", typ))
            .unwrap_or_default();

        write!(
            f,
            "{attrs}{kind}{tplt_args} {name}{tplt_params}{typ}{init};"
        )
    }
}

impl Display for DeclarationKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DeclarationKind::Const => write!(f, "const"),
            DeclarationKind::Override => write!(f, "override"),
            DeclarationKind::Let => write!(f, "let"),
            DeclarationKind::Var => write!(f, "var"),
        }
    }
}

impl Display for Alias {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let typ = &self.typ;
        let template_params = fmt_template_params(&self.template_parameters);
        write!(f, "alias {name}{template_params} = {typ};")
    }
}

fn fmt_template_params(params: &[S<FormalTemplateParameter>]) -> String {
    let mut result = String::new();
    if !params.is_empty() {
        result.push('<');
        result.push_str(&params.iter().format(", ").to_string());
        result.push('>');
    }
    result
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let members = Indent(self.members.iter().format(",\n"));
        let template_params = fmt_template_params(&self.template_parameters);
        write!(f, "struct {name}{template_params} {{\n{members}\n}}")
    }
}

impl Display for StructMember {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let name = &self.name;
        let typ = &self.typ;
        write!(f, "{attrs}{name}: {typ}")
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let name = &self.name;
        let params = self.parameters.iter().format(", ");
        let ret_attrs = fmt_attrs(&self.return_attributes, true);
        let ret_typ = self
            .return_type
            .as_ref()
            .map(|typ| format!("-> {ret_attrs}{} ", typ))
            .unwrap_or_default();
        let template_params = fmt_template_params(&self.template_parameters);
        let body = &self.body;
        write!(
            f,
            "{attrs}fn {name}{template_params}({params}) {ret_typ}{body}"
        )
    }
}

impl Display for FormalParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, true);
        let name = &self.name;
        let typ = &self.typ;
        write!(f, "{attrs}{name}: {typ}")
    }
}

impl Display for ConstAssert {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let expr = &self.expression;
        let template_params = fmt_template_params(&self.template_parameters);
        write!(f, "const_assert{template_params} {expr};",)
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let args = self
            .arguments
            .as_ref()
            .map(|args| format!("({})", args.iter().format(", ")))
            .unwrap_or_default();
        write!(f, "@{name}{args}")
    }
}

fn fmt_attrs(attrs: &[S<Attribute>], inline: bool) -> String {
    let print = attrs.iter().format(" ");
    let suffix = if attrs.is_empty() {
        ""
    } else if inline {
        " "
    } else {
        "\n"
    };
    format!("{print}{suffix}")
}

impl<T: Display> Display for S<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(print) => write!(f, "{}", print),
            Expression::Parenthesized(expr) => {
                write!(f, "({expr})")
            }
            Expression::NamedComponent(print) => write!(f, "{}", print),
            Expression::Indexing(print) => write!(f, "{}", print),
            Expression::Unary(print) => write!(f, "{}", print),
            Expression::Binary(print) => write!(f, "{}", print),
            Expression::FunctionCall(print) => write!(f, "{}", print),
            Expression::Identifier(print) => write!(f, "{}", print),
            Expression::Type(print) => write!(f, "{}", print),
        }
    }
}

impl Display for LiteralExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LiteralExpression::True => write!(f, "true"),
            LiteralExpression::False => write!(f, "false"),
            LiteralExpression::AbstractInt(num) => write!(f, "{}", num.parse::<i64>().unwrap()),
            LiteralExpression::AbstractFloat(num) => write!(f, "{:?}", num.parse::<f64>().unwrap()), // using the Debug formatter to print the trailing .0 in floats representing integers. because format!("{}", 3.0f32) == "3"
            LiteralExpression::I32(num) => write!(f, "{num}i"),
            LiteralExpression::U32(num) => write!(f, "{num}u"),
            LiteralExpression::F32(num) => write!(f, "{}f", num.parse::<f32>().unwrap()),
            LiteralExpression::F16(num) => write!(f, "{}h", num.parse::<f32>().unwrap()),
        }
    }
}

// impl Display for ParenthesizedExpression {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         let expr = self);
//         write!(f, "({expr})")
//     }
// }

impl Display for NamedComponentExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let base = &self.base;
        let component = &self.component;
        write!(f, "{base}.{component}")
    }
}

impl Display for IndexingExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let base = &self.base;
        let index = &self.index;
        write!(f, "{base}[{index}]")
    }
}

impl Display for UnaryExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let operator = &self.operator;
        let operand = &self.operand;
        write!(f, "{operator}{operand}")
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::LogicalNegation => write!(f, "!"),
            UnaryOperator::Negation => write!(f, "-"),
            UnaryOperator::BitwiseComplement => write!(f, "~"),
            UnaryOperator::AddressOf => write!(f, "&"),
            UnaryOperator::Indirection => write!(f, "*"),
        }
    }
}

impl Display for BinaryExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let operator = &self.operator;
        let left = &self.left;
        let right = &self.right;
        write!(f, "{left} {operator} {right}")
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::ShortCircuitOr => write!(f, "||"),
            BinaryOperator::ShortCircuitAnd => write!(f, "&&"),
            BinaryOperator::Addition => write!(f, "+"),
            BinaryOperator::Subtraction => write!(f, "-"),
            BinaryOperator::Multiplication => write!(f, "*"),
            BinaryOperator::Division => write!(f, "/"),
            BinaryOperator::Remainder => write!(f, "%"),
            BinaryOperator::Equality => write!(f, "=="),
            BinaryOperator::Inequality => write!(f, "!="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::LessThanEqual => write!(f, "<="),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::GreaterThanEqual => write!(f, ">="),
            BinaryOperator::BitwiseOr => write!(f, "|"),
            BinaryOperator::BitwiseAnd => write!(f, "&"),
            BinaryOperator::BitwiseXor => write!(f, "^"),
            BinaryOperator::ShiftLeft => write!(f, "<<"),
            BinaryOperator::ShiftRight => write!(f, ">>"),
        }
    }
}

impl Display for FunctionCallExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let path = self.path.iter().format("::");
        let args = self.arguments.iter().format(", ");
        write!(f, "{path}({args})")
    }
}

impl Display for PathPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.name, fmt_template_args(&self.template_args))
    }
}

impl Display for TypeExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.path.iter().format("::"))
    }
}

impl Display for IdentifierExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.path.iter().format("::"))
    }
}

fn fmt_template_args(tplt: &Option<Vec<S<TemplateArg>>>) -> String {
    match tplt {
        Some(tplt) if !tplt.is_empty() => {
            let print = tplt.iter().format(", ");
            format!("<{print}>")
        }
        _ => "".to_string(),
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Void => write!(f, ";"),
            Statement::Compound(print) => write!(f, "{}", print),
            Statement::Assignment(print) => write!(f, "{}", print),
            Statement::Increment(expr) => write!(f, "{}++;", expr),
            Statement::Decrement(expr) => write!(f, "{}--;", expr),
            Statement::If(print) => write!(f, "{}", print),
            Statement::Switch(print) => write!(f, "{}", print),
            Statement::Loop(print) => write!(f, "{}", print),
            Statement::For(print) => write!(f, "{}", print),
            Statement::While(print) => write!(f, "{}", print),
            Statement::Break => write!(f, "break;"),
            Statement::Continue => write!(f, "continue;"),
            Statement::Return(expr) => {
                let expr = expr
                    .as_ref()
                    .map(|expr| format!(" {}", expr))
                    .unwrap_or_default();
                write!(f, "return{expr};")
            }
            Statement::Discard => write!(f, "discard;"),
            Statement::FunctionCall(expr) => write!(f, "{};", expr),
            Statement::ConstAssert(print) => write!(f, "{}", print),
            Statement::Declaration(print) => write!(f, "{}", print),
        }
    }
}

impl Display for CompoundDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompoundDirective::Use(usage) if matches!(usage.content.value, UseContent::Item(_)) => {
                writeln!(f, "use {usage};")?;
            }
            CompoundDirective::Use(usage) => {
                writeln!(f, "use {usage}")?;
            }
        }
        Ok(())
    }
}

impl Display for ModuleDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleDirective::Use(usage) if matches!(usage.content.value, UseContent::Item(_)) => {
                writeln!(f, "use {usage};\n")?;
            }
            ModuleDirective::Use(usage) => {
                writeln!(f, "use {usage}\n")?;
            }
            ModuleDirective::Extend(extend) => {
                writeln!(f, "{extend}")?;
            }
        }
        Ok(())
    }
}

impl Display for ExtendDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attrs = fmt_attrs(&self.attributes, true);

        writeln!(f, "{attrs}extend {};", self.path.iter().format("::"))
    }
}

impl Display for CompoundStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, true);
        let mut directives = format!("{}", self.directives.iter().format("\n"));
        if !directives.is_empty() {
            directives = format!("{}", Indent(directives));
            directives.push('\n');
        }
        let stmts = Indent(self.statements.iter().format("\n"));
        write!(f, "{attrs}{{\n{}{stmts}\n}}", directives)
    }
}

impl Display for AssignmentStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let operator = &self.operator;
        let lhs = &self.lhs;
        let rhs = &self.rhs;
        write!(f, "{lhs} {operator} {rhs};")
    }
}

impl Display for AssignmentOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AssignmentOperator::Equal => write!(f, "="),
            AssignmentOperator::PlusEqual => write!(f, "+="),
            AssignmentOperator::MinusEqual => write!(f, "-="),
            AssignmentOperator::TimesEqual => write!(f, "*="),
            AssignmentOperator::DivisionEqual => write!(f, "/="),
            AssignmentOperator::ModuloEqual => write!(f, "%="),
            AssignmentOperator::AndEqual => write!(f, "&="),
            AssignmentOperator::OrEqual => write!(f, "|="),
            AssignmentOperator::XorEqual => write!(f, "^="),
            AssignmentOperator::ShiftRightAssign => write!(f, ">>="),
            AssignmentOperator::ShiftLeftAssign => write!(f, "<<="),
        }
    }
}

impl Display for DeclarationStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.declaration)?;
        if !self.statements.is_empty() {
            f.write_char('\n')?;
            let stmts = self.statements.iter().format("\n");
            write!(f, "{stmts}")?;
        };
        Ok(())
    }
}

impl Display for IfStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let expr = &self.if_clause.0;
        let stmt = &self.if_clause.1;
        write!(f, "{attrs}if {expr} {stmt}")?;
        for else_if_clause in self.else_if_clauses.iter() {
            let expr = &else_if_clause.0;
            let stmt = &else_if_clause.1;
            write!(f, "\nelse if {expr} {stmt}")?;
        }
        if let Some(ref else_stmt) = self.else_clause {
            write!(f, "\nelse {else_stmt}")?;
        }
        Ok(())
    }
}

impl Display for SwitchStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let expr = &self.expression;
        let body_attrs = fmt_attrs(&self.body_attributes, false);
        let clauses = Indent(self.clauses.iter().format("\n"));
        write!(f, "{attrs}switch {expr} {body_attrs}{{\n{clauses}\n}}")
    }
}

impl Display for SwitchClause {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cases = self.case_selectors.iter().format(", ");
        let body = &self.body;
        write!(f, "case {cases} {body}")
    }
}

impl Display for CaseSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CaseSelector::Default => write!(f, "default"),
            CaseSelector::Expression(expr) => {
                write!(f, "{}", expr)
            }
        }
    }
}

impl Display for LoopStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let body_attrs = fmt_attrs(&self.body.attributes, false);
        let stmts = Indent(
            self.body
                .statements
                .iter()
                // .filter(|stmt| !matches!(stmt, Statement::Void))
                .format("\n"),
        );
        let continuing = self
            .continuing
            .as_ref()
            .map(|cont| format!("{}\n", Indent(cont)))
            .unwrap_or_default();
        write!(f, "{attrs}loop {body_attrs}{{\n{stmts}\n{continuing}}}")
    }
}

impl Display for ContinuingStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let body_attrs = fmt_attrs(&self.body.attributes, false);
        let stmts = Indent(
            self.body
                .statements
                .iter()
                // .filter(|stmt| !matches!(stmt, Statement::Void))
                .format("\n"),
        );
        let break_if = self
            .break_if
            .as_ref()
            .map(|cont| format!("{};\n", Indent(cont)))
            .unwrap_or_default();
        write!(f, "continuing {body_attrs}{{\n{stmts}\n{break_if}}}")
    }
}

// impl Display for BreakIfStatement {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         Ok(())
//     }
// }

impl Display for ForStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let mut init = self
            .initializer
            .as_ref()
            .map(|stmt| format!("{}", stmt))
            .unwrap_or_default();
        if init.ends_with(';') {
            init.pop();
        }
        let cond = self
            .condition
            .as_ref()
            .map(|expr| format!("{}", expr))
            .unwrap_or_default();
        let mut updt = self
            .update
            .as_ref()
            .map(|stmt| format!("{}", stmt))
            .unwrap_or_default();
        if updt.ends_with(';') {
            updt.pop();
        }
        let body = &self.body;
        write!(f, "{attrs}for ({init}; {cond}; {updt}) {body}")
    }
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let cond = &self.condition;
        let body = &self.body;
        write!(f, "{attrs}while ({cond}) {body}")
    }
}

// BEGIN WESL ADDITIONS
impl Display for ModuleMemberDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            &ModuleMemberDeclaration::Void => write!(f, ";"),
            ModuleMemberDeclaration::Declaration(print) => write!(f, "{}", print),
            ModuleMemberDeclaration::Alias(print) => write!(f, "{}", print),
            ModuleMemberDeclaration::Struct(print) => write!(f, "{}", print),
            ModuleMemberDeclaration::Function(print) => write!(f, "{}", print),
            ModuleMemberDeclaration::ConstAssert(print) => write!(f, "{}", print),
            ModuleMemberDeclaration::Module(print) => write!(f, "{}", print),
        }
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let members = self.members.iter().format("\n\n");
        let directives = self.directives.iter().format("\n");
        let mut template_params = String::new();
        if !self.template_parameters.is_empty() {
            template_params.push('<');
            template_params.push_str(&self.template_parameters.iter().format(", ").to_string());
            template_params.push('>');
        }
        write!(
            f,
            "{}{}mod {}{} {{\n{}\n}}",
            attrs,
            if attrs.is_empty() { "" } else { " " },
            self.name,
            template_params,
            Indent(format!("{}{}", directives, members))
        )
    }
}

impl Display for UseContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UseContent::Item(UseItem {
                name,
                rename,
                template_args,
                inline_template_args,
            }) => {
                let mut args = String::new();
                if let Some(template_args) = template_args.as_ref() {
                    args.push('<');
                    args.push_str(&template_args.iter().format(", ").to_string());
                    args.push('>');
                };
                if let Some(inlines) = inline_template_args.as_ref() {
                    args.push_str(&format!("{inlines}"));
                }
                if let Some(rename) = rename {
                    write!(f, "{name}{args} as {rename}")
                } else {
                    write!(f, "{name}{args}")
                }
            }
            UseContent::Collection(c) => {
                write!(f, "{{ {} }}", c.iter().map(|x| format!("{x}")).join(", "))
            }
        }
    }
}

impl Display for InlineTemplateArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "with {{\n{}\n}}",
            Indent(format!(
                "{}{}",
                self.directives.iter().map(|x| format!("{x}")).join("\n"),
                self.members.iter().map(|x| format!("{x}")).join("\n"),
            ))
        )
    }
}

impl Display for Use {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attrs = fmt_attrs(&self.attributes, false);
        let path = self.path.iter().format("::").to_string();
        if !path.is_empty() {
            write!(f, "{attrs}{path}::{}", self.content.value)?;
        } else {
            write!(f, "{attrs}{}", self.content.value)?;
        };
        Ok(())
    }
}

impl Display for TemplateArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(arg_name) = self.arg_name.as_ref() {
            write!(f, "{arg_name} = {}", self.expression)
        } else {
            write!(f, "{}", self.expression)
        }
    }
}

impl Display for FormalTemplateParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(default_value) = self.default_value.as_ref() {
            write!(f, "{} = {default_value}", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}
