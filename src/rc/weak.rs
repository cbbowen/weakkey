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
pub type WeakKey<T> = generic::ByPointer<Weak<T>>;

impl<T> From<&Rc<T>> for WeakKey<T> {
    fn from(value: &Rc<T>) -> Self {
        Rc::downgrade(value).into()
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
