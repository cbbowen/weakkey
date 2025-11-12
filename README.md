This crate provides four thin wrappers—[`rc::RcKey`], [`rc::WeakKey`], [`sync::ArcKey`], and [`sync::WeakKey`]—around the corresponding [`std::rc`] and [`std::sync`] pointer types, with implementations of [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`], and [`Hash`] that make them usable as keys, for example, in a [`HashMap`] or [`BTreeMap`].

[`Hash`]: std::hash::Hash
[`HashMap`]: std::collections::HashMap
[`BTreeMap`]: std::collections::BTreeMap

# Examples

```rust
let mut s = std::collections::HashSet::<weakkey::rc::WeakKey<()>>::new();
let r = std::rc::Rc::new(());
s.insert((&r).into());
```

```rust
let mut s = std::collections::BTreeSet::<weakkey::rc::WeakKey<()>>::new();
let r = std::rc::Rc::new(());
s.insert((&r).into());
```

```rust
let mut s = std::collections::HashSet::<weakkey::sync::WeakKey<()>>::new();
let r = std::sync::Arc::new(());
s.insert((&r).into());
```

```rust
let mut s = std::collections::BTreeSet::<weakkey::sync::WeakKey<()>>::new();
let r = std::sync::Arc::new(());
s.insert((&r).into());
```
