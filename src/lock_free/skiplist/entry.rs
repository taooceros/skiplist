use std::cmp::Ordering;
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::Mutex;

use parking_lot::ReentrantMutex;

use crate::lock_free::markable_atomic_ptr::MarkableAtomicPtr;

pub struct Entry<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    pub key: Key<K>,
    pub value: Option<V>,
    pub top_level: usize,
    pub nexts: Vec<MarkableAtomicPtr<Entry<K, V, C>>>,
}

impl<K, V, C> Entry<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    pub fn new(key: Key<K>, value: Option<V>, top_level: usize) -> Self {
        let mut entry = Entry {
            key,
            value,
            top_level,
            nexts: Vec::with_capacity(top_level + 1),
        };

        for _ in 0..=top_level {
            entry.nexts.push(Default::default());
        }

        entry
    }
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
