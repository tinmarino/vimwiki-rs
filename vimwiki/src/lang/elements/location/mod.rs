use crate::lang::parsers::Span;
use derive_more::{Deref, DerefMut, Display, From};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

mod position;
pub use position::Position;
mod region;
pub use region::Region;

/// Represents an encapsulation of a language element and its location
/// within some string/file
#[derive(Clone, Debug, Display, Deref, DerefMut, Eq, Serialize, Deserialize)]
#[display(fmt = "{}", element)]
pub struct Located<'a, T> {
    #[deref]
    #[deref_mut]
    pub element: T,
    lazy_region: LazyRegion<'a>,
}

impl<'a, T> Located<'a, T> {
    pub fn new(inner: T, lazy_region: impl Into<LazyRegion<'a>>) -> Self {
        Self {
            element: inner,
            lazy_region: lazy_region.into(),
        }
    }

    /// Maps a `Located<T>` to `Located<U>` by applying a
    /// function to the underlying element. Useful when upleveling the
    /// element (such as wrapping a Header1) while the region remains
    /// unchanged.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Located<'a, U> {
        Located::new(f(self.element), self.lazy_region)
    }

    /// Takes a `Located` and replaces its region, producing the
    /// updated region. This takes ownership of the existing element!
    pub fn take_with_region<'b>(
        self,
        lazy_region: impl Into<LazyRegion<'b>>,
    ) -> Located<'b, T> {
        Located::new(self.element, lazy_region.into())
    }

    /// Takes a `Located` and shifts its region such that it starts
    /// at the specified line. This takes ownership of the existing element!
    pub fn take_at_line(mut self, line: usize) -> Self {
        let mut region: Region = self.lazy_region.into();
        let diff = region.end.line - region.start.line;
        region.start.line = line;
        region.end.line = line + diff;
        self.lazy_region = LazyRegion::Owned(region);
        self
    }

    /// Converts from `&Located<T>` to `Located<&T>`
    pub fn as_ref(&self) -> Located<&T> {
        Located {
            element: &self.element,
            lazy_region: self.lazy_region,
        }
    }

    /// Converts from `&mut Located<T>` to `Located<&mut T>`
    pub fn as_mut(&mut self) -> Located<&mut T> {
        Located {
            element: &mut self.element,
            lazy_region: self.lazy_region,
        }
    }

    /// Converts from `&Located<T>` to `&T`
    pub fn as_inner(&self) -> &T {
        &self.element
    }

    /// Converts from `&mut Located<T>` to `&mut T`
    pub fn as_mut_inner(&mut self) -> &mut T {
        &mut self.element
    }

    /// Converts from `Located<T>` to `T`
    pub fn into_inner(self) -> T {
        self.element
    }

    /// Returns the lazy region contained within this located instance
    pub fn lazy_region(&self) -> LazyRegion<'a> {
        self.lazy_region
    }
}

impl<'a, T: PartialEq> PartialEq for Located<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

impl<'a, T: PartialEq> PartialEq<T> for Located<'a, T> {
    fn eq(&self, other: &T) -> bool {
        &self.element == other
    }
}

impl<'a, T: Hash> Hash for Located<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.element.hash(state);
    }
}

impl<'a, T> From<T> for Located<'a, T> {
    /// Creates a new located element around `T`, using a default location
    fn from(t: T) -> Self {
        Self::new(t, LazyRegion::default())
    }
}

/// Represents a region whose line and column are lazily calculated
///
/// - Serializing involves the expensive calcuation of the line/column
/// - Deserializing will always yield a pre-calculated, owned instance
#[derive(Copy, Clone, Debug, From, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Region", into = "Region")]
pub enum LazyRegion<'a> {
    Borrowed(Span<'a>),
    Owned(Region),
}

impl LazyRegion<'_> {
    pub fn into_owned(self) -> LazyRegion<'static> {
        LazyRegion::Owned(self.into())
    }
}

impl<'a> Default for LazyRegion<'a> {
    fn default() -> Self {
        Self::Owned(Default::default())
    }
}

impl<'a> From<LazyRegion<'a>> for Region {
    fn from(lazy_region: LazyRegion<'a>) -> Self {
        match lazy_region {
            LazyRegion::Borrowed(span) => span.into(),
            LazyRegion::Owned(region) => region,
        }
    }
}

impl<'a> From<Span<'a>> for Region {
    fn from(span: Span<'a>) -> Self {
        let start_pos = Position::new(span.line(), span.column());

        let span = if span.remaining_len() > 0 {
            span.starting_at(span.remaining_len() - 1)
        } else {
            span
        };
        let end_pos = Position::new(span.line(), span.column());

        Region::new(start_pos, end_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn map_should_transform_inner_element_and_keep_region() {
        let le = Located::new(3, Region::from(((1, 2), (3, 4))));
        let mapped_le = le.map(|c| c + 1);
        assert_eq!(mapped_le.element, 4);
        assert_eq!(
            Region::from(mapped_le.lazy_region()),
            Region::from(((1, 2), (3, 4)))
        );
    }

    #[test]
    fn located_element_equality_with_other_located_element_should_only_use_inner_element(
    ) {
        let le1 = Located::new(3, Region::from(((1, 2), (3, 4))));
        let le2 = Located::new(3, Region::default());
        assert_eq!(le1, le2);
    }

    #[test]
    fn located_element_equality_with_inner_type_should_only_use_inner_element()
    {
        let le = Located::new(3, Region::from(((1, 2), (3, 4))));
        let inner = 3;
        assert_eq!(le, inner);
        assert!(le != inner + 1);
    }

    #[test]
    fn located_element_hashing_should_only_use_inner_element() {
        let le1 = Located::new(3, Region::from(((1, 2), (3, 4))));
        let le2 = Located::new(3, Region::default());
        let le3 = Located::new(4, Region::from(((1, 2), (3, 4))));
        let le4 = Located::new(3, Region::from(((1, 2), (3, 4))));

        let mut m = HashSet::new();
        m.insert(le1);

        let le = m
            .get(&le2)
            .expect("Failed to retrieve Located with another Located");
        assert_eq!(le.element, 3);
        assert_eq!(
            Region::from(le.lazy_region()),
            Region::from(((1, 2), (3, 4)))
        );

        assert_eq!(m.get(&le3), None);

        let le = m
            .get(&le4)
            .expect("Failed to retrieve Located with another Located");
        assert_eq!(le.element, 3);
        assert_eq!(
            Region::from(le.lazy_region()),
            Region::from(((1, 2), (3, 4)))
        );
    }

    #[test]
    fn located_element_as_ref_should_return_new_element_with_ref_and_same_region(
    ) {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let le = Located::new(Test(5), Region::from(((1, 2), (3, 4))));
        let le_ref = le.as_ref();

        assert_eq!(le_ref.element, &Test(5));
        assert_eq!(
            Region::from(le_ref.lazy_region()),
            Region::from(((1, 2), (3, 4)))
        );
    }

    #[test]
    fn located_element_as_mut_should_return_new_element_with_mut_and_same_region(
    ) {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let mut le = Located::new(Test(5), Region::from(((1, 2), (3, 4))));
        let le_mut = le.as_mut();

        assert_eq!(le_mut.element, &mut Test(5));
        assert_eq!(
            Region::from(le_mut.lazy_region()),
            Region::from(((1, 2), (3, 4)))
        );
    }

    #[test]
    fn located_element_as_inner_should_return_new_element_with_ref_to_inner_and_same_region(
    ) {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let le = Located::new(Test(5), Region::from(((1, 2), (3, 4))));
        let inner = le.as_inner();

        assert_eq!(inner, &Test(5));
    }

    #[test]
    fn located_element_as_mut_inner_should_return_new_element_with_mut_ref_to_inner_and_same_region(
    ) {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let mut le = Located::new(Test(5), Region::from(((1, 2), (3, 4))));
        let inner = le.as_mut_inner();

        assert_eq!(inner, &mut Test(5));
    }

    #[test]
    fn located_element_into_inner_should_return_inner_element_as_owned() {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let le = Located::new(Test(5), Region::from(((1, 2), (3, 4))));
        let inner = le.into_inner();

        assert_eq!(inner, Test(5));
    }
}
