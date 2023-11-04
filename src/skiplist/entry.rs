use std::{cmp::Ordering, ptr::null_mut};

pub(crate) struct Entry<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    pub key: Key<K>,
    pub value: Option<V>,
    pub nexts: Vec<*mut Entry<K, V, C>>,
}

pub(crate) enum Key<K>
where
    K: Ord,
{
    Head,
    Entry(K),
    Tail,
}