use crate::lock_base::skiplist::entry::Entry;
use std::cmp::Ordering;

mod entry;

pub struct SkipList<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    head: *mut Entry<K, V, C>,
    tail: *mut Entry<K, V, C>,
    compare: C,
}
