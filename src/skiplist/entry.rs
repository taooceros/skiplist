use std::ptr::null_mut;

pub(crate) struct Entry<K, V>
where
    K: Ord,
{
    pub key: K,
    pub value: V,
    pub nexts: Vec<*mut Entry<K, V>>,
}
impl<K, V> Entry<K, V> where K: Ord {
    pub fn new(key: K, value: V, top_level: usize) -> Self {
        Entry {
            key,
            value,
            nexts: vec![null_mut(); top_level + 1],
        }
    }
}
