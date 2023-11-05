use std::{cmp::Ordering};

pub(crate) struct Entry<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    pub key: Key<K>,
    pub value: Option<V>,
    pub nexts: Vec<*mut Entry<K, V, C>>,
}

#[derive(PartialEq)]
pub(crate) enum Key<K>
where
    K: Ord,
{
    Head,
    Entry(K),
    Tail,
}


impl<K> PartialOrd for Key<K> where K: Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Key::Head, Key::Head) => Some(Ordering::Equal),
            (Key::Head, _) => Some(Ordering::Less),
            (_, Key::Head) => Some(Ordering::Greater),
            (Key::Tail, Key::Tail) => Some(Ordering::Equal),
            (Key::Tail, _) => Some(Ordering::Greater),
            (_, Key::Tail) => Some(Ordering::Less),
            (Key::Entry(k1), Key::Entry(k2)) => k1.partial_cmp(k2),
        }
    }
}
