use crate::generic;
use alloc::sync::Arc;

impl<T> generic::Pointer for Arc<T> {
    type Strong = Arc<T>;
    type Key = *const ();

    fn upgrade(&self) -> Option<Self::Strong> {
        Some(self.clone())
    }

    fn key(&self) -> Self::Key {
        Arc::as_ptr(self) as *const ()
    }
}

/// A thin wrapper around [`std::sync::Arc`] suitable for use as a key.
///
/// Equality and comparisons are implemented in terms of the inner value pointer and the hash is
/// consistent with this definition. This is stable in the presence of internal mutability and
/// when the inner value is dropped.
pub type ArcKey<T> = generic::ByPointer<Arc<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use core::hash::{Hash, Hasher};
    use proptest::prelude::*;

    #[derive(Debug)]
    struct TestValue;

    fn test_arc() -> impl Strategy<Value = Arc<TestValue>> {
        Just(Arc::new(TestValue))
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
        fn into_inner(w in test_arc()) {
            let key: ArcKey<_> = w.clone().into();
            assert_eq!(Arc::as_ptr(&key.into_inner()), Arc::as_ptr(&w));
        }

        #[test]
        fn inner(w in test_arc()) {
            let key: ArcKey<_> = w.clone().into();
            assert_eq!(Arc::as_ptr(&key.inner()), Arc::as_ptr(&w));
        }

        #[test]
        fn from_weak(w in test_arc()) {
            let key = ArcKey::from(w.clone());
            assert_eq!(Arc::as_ptr(&key.into_inner()), Arc::as_ptr(&w));
        }

        #[test]
        fn clone(w in test_arc()) {
            let key: ArcKey<_> = w.clone().into();
            assert_eq!(key.clone(), key);
        }

        #[test]
        fn eq(wa in test_arc(), wb in test_arc()) {
            let ka: ArcKey<_> = wa.clone().into();
            let kb: ArcKey<_> = wb.clone().into();
            assert_eq!(ka, ka);
            assert_eq!(kb, kb);
            assert_eq!(ka == kb, Arc::ptr_eq(&wa, &wb));
            assert_eq!(kb == ka, Arc::ptr_eq(&wb, &wa));
        }

        #[test]
        fn lt(wa in test_arc(), wb in test_arc()) {
            let ka: ArcKey<_> = wa.clone().into();
            let kb: ArcKey<_> = wb.clone().into();
            assert!(!(ka < ka));
            assert!(!(kb < kb));
            let a_eq_b = if ka == kb { 1 } else { 0 };
            let a_lt_b = if ka < kb { 1 } else { 0 };
            let b_lt_a = if kb < ka { 1 } else { 0 };
            assert_eq!(a_eq_b + a_lt_b + b_lt_a, 1);
        }

        #[test]
        fn hash(wa in test_arc(), wb in test_arc()) {
            let ka: ArcKey<_> = wa.clone().into();
            let mut ha = TestHasher::default();
            ka.hash(&mut ha);

            let kb: ArcKey<_> = wb.clone().into();
            let mut hb = TestHasher::default();
            kb.hash(&mut hb);

            assert_eq!(ha == hb, Arc::ptr_eq(&wa, &wb));
        }

    }
}
