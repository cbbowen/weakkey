This crate provides a single type, `WeakKey`, which is a thin wrapper around `std::rc::Weak<T>` with implementations of `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash` that make it usable as a key, for example, in a `std::collections::HashMap` or `std::collections::BTreeMap`.

# Examples

```rust
let mut s = std::collections::HashSet::<weakkey::WeakKey<()>>::new();
let r = std::rc::Rc::new(());
s.insert((&r).into());
```

```rust
let mut s = std::collections::BTreeSet::<weakkey::WeakKey<()>>::new();
let r = std::rc::Rc::new(());
s.insert((&r).into());
```