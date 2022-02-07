use std::marker::PhantomData;

/// Convenience type that can be used to avoid having to write a separate
/// newtype for each different key type you want to have.
///
/// # Example
///
/// Instead of this:
///
/// ```
/// use brine_asset::HashSlab;
///
/// struct Foo;
///
/// struct FooKey(usize);
///
/// /*
/// impl From<usize> for FooKey { ... }
/// impl From<FooKey> for usize { ... }
/// impl Add for FooKey { ... }
/// ...
/// */
///
/// let hash_slab: HashSlab<Foo, FooKey> = HashSlab::default();
/// ```
///
/// Do this:
///
/// ```
/// use brine_asset::{HashSlab, UsizeKey};
///
/// struct Foo;
///
/// type FooKey = UsizeKey<Foo>;
///
/// let hash_slab: HashSlab<Foo, FooKey> = HashSlab::default();
/// ```
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct UsizeKey<T>(usize, PhantomData<fn() -> T>);

impl<T> From<usize> for UsizeKey<T> {
    #[inline]
    fn from(u: usize) -> Self {
        Self(u, PhantomData)
    }
}

impl<T> From<UsizeKey<T>> for usize {
    #[inline]
    fn from(k: UsizeKey<T>) -> Self {
        k.0
    }
}
