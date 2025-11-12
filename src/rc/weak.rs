use crate::generic;
use alloc::rc::{Rc, Weak};

impl<T> generic::Pointer for Weak<T> {
    type Strong = Rc<T>;
    type Key = *const ();

    fn upgrade(&self) -> Option<Self::Strong> {
        self.upgrade()
    }

    fn key(&self) -> Self::Key {
        self.as_ptr() as *const ()
    }
}

/// A thin wrapper around [`std::rc::Weak`] suitable for use as a key.
///
/// Equality and comparisons are implemented in terms of the inner value pointer and the hash is
/// consistent with this definition. This is stable in the presence of internal mutability and
/// when the inner value is dropped.
pub struct WeakKey<T> {
    inner: generic::ByPointer<Weak<T>>,
}

impl<T> WeakKey<T> {
    /// Returns a [`WeakKey`] with the inner [`Weak`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use weakkey::rc::WeakKey;
    /// let weak = std::rc::Weak::<()>::new();
    /// assert_eq!(WeakKey::new(weak.clone()), WeakKey::new(weak));
    /// ```
    pub fn new(inner: Weak<T>) -> Self {
        Self {
            inner: generic::ByPointer::new(inner),
        }
    }

    /// Returns the inner [`Weak`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use weakkey::rc::WeakKey;
    /// let weak = std::rc::Weak::<()>::new();
    /// assert!(WeakKey::new(weak.clone()).into_inner().ptr_eq(&weak));
    /// ```
    pub fn into_inner(self) -> Weak<T> {
        self.inner.into_inner()
    }

    /// Returns a reference to the inner [`Weak`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use weakkey::rc::WeakKey;
    /// let weak = std::rc::Weak::<()>::new();
    /// assert!(WeakKey::new(weak.clone()).inner().ptr_eq(&weak));
    /// ```
    pub fn inner(&self) -> &Weak<T> {
        self.inner.inner()
    }

    /// Attempts to upgrade the [`Weak`] pointer to an [`Rc`], delaying dropping of the inner value
    /// if successful.
    ///
    /// Returns [`None`] if the inner value has since been dropped.
    ///
    /// This is equivalent to `self.inner().upgrade()` but is provided for convenience.
    ///
    /// # Examples
    ///
    /// ```
    /// # use weakkey::rc::WeakKey;
    /// let weak = std::rc::Weak::<()>::new();
    /// assert!(WeakKey::new(weak).upgrade().is_none());
    /// ```
    ///
    /// ```
    /// # use weakkey::rc::WeakKey;
    /// let rc = std::rc::Rc::new(());
    /// assert!(WeakKey::new(std::rc::Rc::downgrade(&rc)).upgrade().is_some());
    /// ```
    pub fn upgrade(&self) -> Option<Rc<T>> {
        self.inner.upgrade()
    }
}

impl<T> Clone for WeakKey<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> PartialEq for WeakKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Eq for WeakKey<T> {}

impl<T> PartialOrd for WeakKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.inner.cmp(&other.inner))
    }
}

impl<T> Ord for WeakKey<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> core::hash::Hash for WeakKey<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<T> From<Weak<T>> for WeakKey<T> {
    fn from(value: Weak<T>) -> Self {
        Self::new(value)
    }
}

impl<T> From<&Rc<T>> for WeakKey<T> {
    fn from(value: &Rc<T>) -> Self {
        Rc::downgrade(value).into()
    }
}

impl<T> core::fmt::Debug for WeakKey<T> {
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

    fn test_rc() -> impl Strategy<Value = (Weak<TestValue>, Option<Rc<TestValue>>)> {
        prop_oneof![
            // Empty case.
            Just((Weak::new(), None)),
            // Dangling case.
            {
                let rc = Rc::new(TestValue);
                Just((Rc::downgrade(&rc), None))
            },
            // Valid case.
            {
                let rc = Rc::new(TestValue);
                Just((Rc::downgrade(&rc), Some(rc)))
            },
        ]
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
        fn into_inner((weak, _) in test_rc()) {
            let key: WeakKey<_> = weak.clone().into();
            assert_eq!(key.into_inner().as_ptr(), weak.as_ptr());
        }

        #[test]
        fn inner((weak, _) in test_rc()) {
            let key: WeakKey<_> = weak.clone().into();
            assert_eq!(key.inner().as_ptr(), weak.as_ptr());
        }

        #[test]
        fn upgrade((weak, strong) in test_rc()) {
            let key: WeakKey<_> = weak.clone().into();
            assert_eq!(key.upgrade().as_ref().map(Rc::as_ptr), strong.as_ref().map(Rc::as_ptr));
        }

        #[test]
        fn from_weak((weak, _) in test_rc()) {
            let key = WeakKey::from(weak.clone());
            assert_eq!(key.into_inner().as_ptr(), weak.as_ptr());
        }

        #[test]
        fn from_strong((_, strong) in test_rc()) {
            if let Some(strong) = strong {
                let key = WeakKey::from(&strong);
                assert_eq!(key.into_inner().as_ptr(), Rc::as_ptr(&strong));
            }
        }

        #[test]
        fn clone((weak, _) in test_rc()) {
            let key: WeakKey<_> = weak.clone().into();
            assert_eq!(key.clone(), key);
        }

        #[test]
        fn eq((wa, _) in test_rc(), (wb, _) in test_rc()) {
            let ka: WeakKey<_> = wa.clone().into();
            let kb: WeakKey<_> = wb.clone().into();
            assert_eq!(ka, ka);
            assert_eq!(kb, kb);
            assert_eq!(ka == kb, wa.ptr_eq(&wb));
            assert_eq!(kb == ka, wb.ptr_eq(&wa));
        }

        #[test]
        fn lt((wa, _) in test_rc(), (wb, _) in test_rc()) {
            let ka: WeakKey<_> = wa.clone().into();
            let kb: WeakKey<_> = wb.clone().into();
            assert!(!(ka < ka));
            assert!(!(kb < kb));
            let a_eq_b = if ka == kb { 1 } else { 0 };
            let a_lt_b = if ka < kb { 1 } else { 0 };
            let b_lt_a = if kb < ka { 1 } else { 0 };
            assert_eq!(a_eq_b + a_lt_b + b_lt_a, 1);
        }

        #[test]
        fn hash((wa, _) in test_rc(), (wb, _) in test_rc()) {
            let ka: WeakKey<_> = wa.clone().into();
            let mut ha = TestHasher::default();
            ka.hash(&mut ha);

            let kb: WeakKey<_> = wb.clone().into();
            let mut hb = TestHasher::default();
            kb.hash(&mut hb);

            assert_eq!(ha == hb, wa.ptr_eq(&wb));
        }

    }
}
