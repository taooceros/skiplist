use std::cmp::Ordering;

pub struct Entry<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    pub key: Key<K>,
    pub value: Option<V>,
    pub nexts: Vec<*mut Entry<K, V, C>>,
}

pub enum Key<K>
where
    K: Ord,
{
    Head,
    Entry(K),
    Tail,
}
