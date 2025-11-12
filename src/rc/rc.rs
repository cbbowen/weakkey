use crate::generic;
use alloc::rc::{Rc};

impl<T> generic::Pointer for Rc<T> {
    type Strong = Rc<T>;
    type Key = *const ();

    fn upgrade(&self) -> Option<Self::Strong> {
        Some(self.clone())
    }

    fn key(&self) -> Self::Key {
        Rc::as_ptr(self) as *const ()
    }
}

/// A thin wrapper around [`std::rc::Rc`] suitable for use as a key.
///
/// Equality and comparisons are implemented in terms of the inner value pointer and the hash is
/// consistent with this definition. This is stable in the presence of internal mutability and
/// when the inner value is dropped.
pub struct RcKey<T> {
    inner: generic::ByPointer<Rc<T>>,
}

impl<T> RcKey<T> {
    /// Returns a [`RcKey`] wrapping the provided [`Rc`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use weakkey::rc::RcKey;
    /// let rc = std::rc::Rc::new(());
    /// assert_eq!(RcKey::new(rc.clone()), RcKey::from(&rc));
    /// ```
    pub fn new(inner: Rc<T>) -> Self {
        Self {
            inner: generic::ByPointer::new(inner),
        }
    }

    /// Returns the inner [`Rc`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use weakkey::rc::RcKey;
    /// let rc = std::rc::Rc::new(());
    /// assert!(std::rc::Rc::ptr_eq(&RcKey::new(rc.clone()).into_inner(), &rc));
    /// ```
    pub fn into_inner(self) -> Rc<T> {
        self.inner.into_inner()
    }

    /// Returns a reference to the inner [`Rc`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use weakkey::rc::RcKey;
    /// let rc = std::rc::Rc::new(());
    /// let key = RcKey::new(rc.clone());
    /// assert!(std::rc::Rc::ptr_eq(key.inner(), &rc));
    /// ```
    pub fn inner(&self) -> &Rc<T> {
        self.inner.inner()
    }
}

impl<T> Clone for RcKey<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> PartialEq for RcKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Eq for RcKey<T> {}

impl<T> PartialOrd for RcKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.inner.cmp(&other.inner))
    }
}

impl<T> Ord for RcKey<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> core::hash::Hash for RcKey<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<T> From<Rc<T>> for RcKey<T> {
    fn from(value: Rc<T>) -> Self {
        Self::new(value)
    }
}

impl<T> From<&Rc<T>> for RcKey<T> {
    fn from(value: &Rc<T>) -> Self {
        Self::new(value.clone())
    }
}

impl<T> core::fmt::Debug for RcKey<T> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.inner.fmt(fmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::hash::{Hash, Hasher};
    use proptest::prelude::*;

    #[derive(Debug)]
    struct TestValue;

    fn test_rc() -> impl Strategy<Value = Rc<TestValue>> {
        Just(Rc::new(TestValue))
    }

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct TestHasher {
        writes: alloc::vec::Vec<alloc::vec::Vec<u8>>,
    }

    impl Hasher for TestHasher {
        fn finish(&self) -> u64 {
            0
        }

        fn write(&mut self, bytes: &[u8]) {
            self.writes.push(bytes.iter().cloned().collect())
        }
    }

    proptest! {

        #[test]
        fn into_inner(w in test_rc()) {
            let key: RcKey<_> = w.clone().into();
            assert_eq!(Rc::as_ptr(&key.into_inner()), Rc::as_ptr(&w));
        }

        #[test]
        fn inner(w in test_rc()) {
            let key: RcKey<_> = w.clone().into();
            assert_eq!(Rc::as_ptr(&key.inner()), Rc::as_ptr(&w));
        }

        #[test]
        fn from_weak(w in test_rc()) {
            let key = RcKey::from(w.clone());
            assert_eq!(Rc::as_ptr(&key.into_inner()), Rc::as_ptr(&w));
        }

        #[test]
        fn clone(w in test_rc()) {
            let key: RcKey<_> = w.clone().into();
            assert_eq!(key.clone(), key);
        }

        #[test]
        fn eq(wa in test_rc(), wb in test_rc()) {
            let ka: RcKey<_> = wa.clone().into();
            let kb: RcKey<_> = wb.clone().into();
            assert_eq!(ka, ka);
            assert_eq!(kb, kb);
            assert_eq!(ka == kb, Rc::ptr_eq(&wa, &wb));
            assert_eq!(kb == ka, Rc::ptr_eq(&wb, &wa));
        }

        #[test]
        fn lt(wa in test_rc(), wb in test_rc()) {
            let ka: RcKey<_> = wa.clone().into();
            let kb: RcKey<_> = wb.clone().into();
            assert!(!(ka < ka));
            assert!(!(kb < kb));
            let a_eq_b = if ka == kb { 1 } else { 0 };
            let a_lt_b = if ka < kb { 1 } else { 0 };
            let b_lt_a = if kb < ka { 1 } else { 0 };
            assert_eq!(a_eq_b + a_lt_b + b_lt_a, 1);
        }

        #[test]
        fn hash(wa in test_rc(), wb in test_rc()) {
            let ka: RcKey<_> = wa.clone().into();
            let mut ha = TestHasher::default();
            ka.hash(&mut ha);

            let kb: RcKey<_> = wb.clone().into();
            let mut hb = TestHasher::default();
            kb.hash(&mut hb);

            assert_eq!(ha == hb, Rc::ptr_eq(&wa, &wb));
        }

    }
}
