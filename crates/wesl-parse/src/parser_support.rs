//! support functions to be injected in the lalrpop parser.

use crate::{span::S, syntax::*};

pub(crate) enum Component {
    Named(S<String>),
    Index(Box<S<Expression>>),
}

pub(crate) fn apply_components(components: Vec<Component>, expr: S<Expression>) -> S<Expression> {
    components.into_iter().fold(expr, |base, comp| match comp {
        Component::Named(component) => {
            let span = base.span().start..component.span().end;
            S::new(
                Expression::NamedComponent(NamedComponentExpression {
                    base: base.into(),
                    component,
                }),
                span,
            )
        }
        Component::Index(index) => {
            let span = base.span().start..index.span().end;
            S::new(
                Expression::Indexing(IndexingExpression {
                    base: base.into(),
                    index,
                }),
                span,
            )
        }
    })
}
