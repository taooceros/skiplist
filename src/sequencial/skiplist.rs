use std::{
    cmp::{min, Ordering},
    ptr::null_mut,
};

use self::entry::{Entry, Key};
use rand::random;

mod entry;

pub struct SkipList<K, V, C = fn(&K, &K) -> Ordering>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    head: *mut Entry<K, V, C>,
    key_cmp: C,
}

fn default_cmp<K: Ord>(k1: &K, k2: &K) -> Ordering {
    k1.cmp(k2)
}

impl<K, V> SkipList<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        SkipList::with_cmp(default_cmp)
    }
}

impl<K, V, C> SkipList<K, V, C>
where
    K: Ord,
    C: Fn(&K, &K) -> Ordering,
{
    pub fn with_cmp(cmp: C) -> Self {
        let head = Box::into_raw(Box::new(Entry {
            key: Key::Head,
            value: None,
            nexts: vec![null_mut(); MAX_LEVEL],
        }));

        let tail = Box::into_raw(Box::new(Entry {
            key: Key::Tail,
            value: None,
            nexts: vec![null_mut(); MAX_LEVEL],
        }));

        unsafe {
            for level in 0..MAX_LEVEL {
                (*head).nexts[level] = tail;
            }
        }

        SkipList { head, key_cmp: cmp }
    }

    pub fn add(&mut self, key: K, value: V) -> bool {
        let top_level = random_level();
        let mut preds = vec![null_mut(); MAX_LEVEL + 1];
        let mut succs = vec![null_mut(); MAX_LEVEL + 1];

        let key = Key::Entry(key);

        let level_found = self.find(&key, &mut preds, &mut succs);

        if level_found.is_some() {
            return false;
        }

        let new_entry = Box::into_raw(Box::new(Entry {
            key,
            value: Some(value),
            nexts: vec![null_mut(); top_level + 1],
        }));

        unsafe {
            for level in 0..=top_level {
                (*new_entry).nexts[level] = succs[level];
                (*preds[level]).nexts[level] = new_entry;
            }
        }

        return true;
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let mut preds = vec![null_mut(); MAX_LEVEL + 1];
        let mut succs = vec![null_mut(); MAX_LEVEL + 1];

        let key = Key::Entry(key);

        let level_found = self.find(&key, &mut preds, &mut succs);

        if level_found.is_none() {
            return None;
        }

        let entry_to_remove = succs[level_found.unwrap()];

        unsafe {
            for level in (0..=level_found.unwrap()).rev() {
                (*preds[level]).nexts[level] = (*entry_to_remove).nexts[level];
            }
        }

        // We uses box as a allocator
        let entry_to_remove = unsafe { Box::from_raw(entry_to_remove) };

        return entry_to_remove.value;
    }

    fn find<'a, 'b>(
        &'a self,
        key: &Key<K>,
        preds: &'b mut Vec<*mut Entry<K, V, C>>,
        succs: &'b mut Vec<*mut Entry<K, V, C>>,
    ) -> Option<usize>
    where
        'a: 'b,
    {
        let head = self.head;

        let mut level_found = None;

        let mut pred = unsafe { &mut *head };

        for level in (0..MAX_LEVEL).rev() {
            let mut current = unsafe { pred.nexts[level].as_mut().unwrap() };

            while current.key < *key {
                pred = current;
                current = unsafe { pred.nexts[level].as_mut().unwrap() };
            }

            if level_found.is_none() && current.key == *key {
                level_found = Some(level);
            }

            preds[level] = pred;
            succs[level] = current;
        }

        return level_found;
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let pred = self.head;

        let key = Key::Entry(key);

        for level in (0..MAX_LEVEL).rev() {
            let mut current = unsafe { (*pred).nexts[level].as_mut().unwrap() };

            while current.key < key {
                current = unsafe { (*current).nexts[level].as_mut().unwrap() };
            }

            if current.key == key {
                return current.value.as_ref();
            }
        }

        return None;
    }
}

static P: f32 = 0.5;
static MAX_LEVEL: usize = 32;

fn random_level() -> usize {
    let level = (f32::log2(1. - random::<f32>()) / f32::log2(1. - P)) as usize;
    return min(level, MAX_LEVEL);
}
