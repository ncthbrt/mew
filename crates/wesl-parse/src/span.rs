use std::ops::{Deref, DerefMut};
pub type Span = std::ops::Range<usize>;

pub(crate) type S<T> = Spanned<T>;

#[derive(Clone, Debug, Default)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Spanned<T> {
    pub fn new(t: T, span: Span) -> Self {
        Self { value: t, span }
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> AsMut<T> for Spanned<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> From<Spanned<T>> for Spanned<Box<T>> {
    fn from(value: Spanned<T>) -> Self {
        Spanned::new(value.value.into(), value.span)
    }
}

impl<T: IntoIterator> IntoIterator for Spanned<T> {
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}
