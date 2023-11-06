use std::cmp::Ordering;
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::Mutex;

use parking_lot::ReentrantMutex;

pub struct Entry<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    pub key: Key<K>,
    pub value: Option<V>,
    pub lock: ReentrantMutex<()>,
    pub marked: AtomicBool,
    pub fully_linked: AtomicBool,
    pub top_level: usize,
    pub nexts: Vec<AtomicPtr<Entry<K, V, C>>>,
}

#[derive(PartialEq, Debug)]
pub enum Key<K>
where
    K: Ord,
{
    Head,
    Entry(K),
    Tail,
}

impl<K> PartialOrd for Key<K>
where
    K: Ord,
{
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
