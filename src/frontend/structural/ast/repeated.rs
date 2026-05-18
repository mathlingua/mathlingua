macro_rules! impl_repeated_items {
    ($name:ident) => {
        impl<T> std::ops::Deref for $name<T> {
            /// Slice view of the wrapped repeated items.
            type Target = [T];

            /// Exposes the repeated wrapper as a slice.
            ///
            /// Most downstream passes only need read-only list behavior, so the
            /// wrapper dereferences to the inner slice while preserving its
            /// cardinality guarantee in the type name.
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> IntoIterator for $name<T> {
            /// Owned item yielded by value iteration.
            type Item = T;
            /// Owned vector iterator used after consuming the wrapper.
            type IntoIter = std::vec::IntoIter<T>;

            /// Consumes the wrapper and iterates over its owned items.
            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a $name<T> {
            /// Shared item yielded by borrowed iteration.
            type Item = &'a T;
            /// Shared slice iterator over the wrapped values.
            type IntoIter = std::slice::Iter<'a, T>;

            /// Iterates immutably over the wrapped items.
            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl<'a, T> IntoIterator for &'a mut $name<T> {
            /// Mutable item yielded by borrowed mutable iteration.
            type Item = &'a mut T;
            /// Mutable slice iterator over the wrapped values.
            type IntoIter = std::slice::IterMut<'a, T>;

            /// Iterates mutably over the wrapped items.
            fn into_iter(self) -> Self::IntoIter {
                self.0.iter_mut()
            }
        }
    };
}

/// A list that may contain any number of items, including none.
///
/// Structural AST nodes use this wrapper instead of bare `Vec<T>` to make the
/// MathLingua section grammar explicit in type signatures.  Optional sections
/// can still distinguish between a missing section and a present empty section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroOrMore<T>(Vec<T>);

impl<T> Default for ZeroOrMore<T> {
    /// Creates an empty repeated-item wrapper.
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> ZeroOrMore<T> {
    /// Consumes the wrapper and returns the underlying vector.
    ///
    /// This is used at API boundaries where callers need ordinary collection
    /// operations that are not worth mirroring on the wrapper type.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for ZeroOrMore<T> {
    /// Wraps a vector without changing its contents or cardinality.
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> From<OneOrMore<T>> for ZeroOrMore<T> {
    /// Forgets the non-empty guarantee while preserving the items.
    fn from(value: OneOrMore<T>) -> Self {
        Self(value.into_vec())
    }
}

impl_repeated_items!(ZeroOrMore);

/// A list that is guaranteed to contain at least one item.
///
/// Required repeated sections use this wrapper so successful structural parsing
/// records the cardinality guarantee in the AST, rather than requiring every
/// backend pass to re-check emptiness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneOrMore<T>(Vec<T>);

impl<T> OneOrMore<T> {
    /// Creates a nonempty list from its first item plus any remaining items.
    pub fn new(first: T, rest: Vec<T>) -> Self {
        let mut items = Vec::with_capacity(rest.len() + 1);
        items.push(first);
        items.extend(rest);
        Self(items)
    }

    /// Consumes the wrapper and returns the underlying vector.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> TryFrom<Vec<T>> for OneOrMore<T> {
    /// Original vector returned when it is empty and cannot satisfy the invariant.
    type Error = Vec<T>;

    /// Converts a vector to a nonempty wrapper when it contains at least one item.
    ///
    /// Empty input is returned unchanged so callers can decide how to report the
    /// missing required content.
    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(value)
        } else {
            Ok(Self(value))
        }
    }
}

impl<T> TryFrom<ZeroOrMore<T>> for OneOrMore<T> {
    /// Original wrapper returned when it is empty and cannot satisfy the invariant.
    type Error = ZeroOrMore<T>;

    /// Converts a zero-or-more wrapper to a one-or-more wrapper when possible.
    ///
    /// On failure the original cardinality wrapper is returned, preserving the
    /// caller's type-level knowledge about the source section.
    fn try_from(value: ZeroOrMore<T>) -> Result<Self, Self::Error> {
        let items = value.into_vec();
        if items.is_empty() {
            Err(ZeroOrMore::default())
        } else {
            Ok(Self(items))
        }
    }
}

impl_repeated_items!(OneOrMore);

