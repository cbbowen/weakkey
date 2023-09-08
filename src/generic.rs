pub trait Weak: Clone {
    type Strong;
    type Key: PartialEq + Eq + PartialOrd + Ord + core::hash::Hash + core::fmt::Debug;

    fn upgrade(&self) -> Option<Self::Strong>;
    fn key(&self) -> Self::Key;
}

// A thin wrapper around `W` suitable for use as a key.
//
// Equality and comparisons are implemented in terms of the inner value pointer and the hash is
// consistent with this definition. This is stable in the presence of internal mutability and
// when the inner value is dropped.
pub struct WeakKey<W> {
    inner: W,
}

impl<W: Weak> WeakKey<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn inner(&self) -> &W {
        &self.inner
    }

    pub fn upgrade(&self) -> Option<W::Strong> {
        self.inner.upgrade()
    }
}

// Note that `WeakKey` must not implement `std::borrow::Borrow` because that requires equality and
// comparison to agree with those of the borrowed type.

impl<W: Weak> Clone for WeakKey<W> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<W: Weak> core::fmt::Debug for WeakKey<W> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_tuple("WeakKey").field(&self.inner.key()).finish()
    }
}

impl<W: Weak> PartialEq for WeakKey<W> {
    fn eq(&self, other: &Self) -> bool {
        // This is identical to `Weak::ptr_eq` for both implementations but clarifies that it
        // agrees with the implementations of `Hash` and `Ord`.
        self.inner.key() == other.inner.key()
    }
}

impl<W: Weak> Eq for WeakKey<W> {}

impl<W: Weak> core::hash::Hash for WeakKey<W> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.key().hash(state)
    }
}

impl<W: Weak> PartialOrd for WeakKey<W> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<W: Weak> Ord for WeakKey<W> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.key().cmp(&other.inner.key())
    }
}

impl<W: Weak> From<W> for WeakKey<W> {
    fn from(value: W) -> Self {
        Self::new(value)
    }
}
