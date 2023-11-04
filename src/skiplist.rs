use std::{
    borrow::BorrowMut,
    cmp::min,
    ptr::{null, null_mut},
};

use self::entry::Entry;
use rand::random;

mod entry;

pub struct SkipList<K, V>
where
    K: Ord,
{
    head: *mut Entry<K, V>,
    max_level: usize,
}

impl<K, V> SkipList<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        SkipList {
            head: Box::into_raw(Box::new(Entry::new(
                0,
                0,
                MAX_LEVEL,
            ))),
            max_level: 0,
        }
    }

    pub fn add(&mut self, key: K, value: V) -> bool {
        let top_level = random_level();
        let mut preds = vec![null_mut(); top_level + 1];
        let mut succs = vec![null_mut(); top_level + 1];

        let level_found = self.find(&key, &mut preds, &mut succs);

        if level_found.is_some() {
            return false;
        }

        let new_entry = Box::into_raw(Box::new(Entry::new(key, value, top_level)));

        unsafe {
            for level in 0..=top_level {
                (*new_entry).nexts[level] = succs[level];
                (*preds[level]).nexts[level] = new_entry;
            }
        }

        return true;
    }

    fn find<'a, 'b>(
        &'a self,
        key: &K,
        preds: &'b mut Vec<*mut Entry<K, V>>,
        succs: &'b mut Vec<*mut Entry<K, V>>,
    ) -> Option<usize>
    where
        'a: 'b,
    {
        let head = self.head;

        let mut level_found = None;

        if head.is_none() {
            return level_found;
        }

        let mut pred = unsafe { &mut *head.unwrap() };

        for level in (self.max_level..0).rev() {
            let mut current = unsafe { pred.nexts[level].as_mut().unwrap() };

            while key > &current.key {
                pred = current;
                current = unsafe { pred.nexts[level].as_mut().unwrap() };
            }

            if level_found.is_none() && key == &current.key {
                level_found = Some(level);
            }

            preds[level] = pred;
            succs[level] = current;
        }

        return level_found;
    }
}

static P: f32 = 0.5;
static MAX_LEVEL: usize = 32;

fn random_level() -> usize {
    let level = (f32::log2(1. - random::<f32>()) / f32::log2(1. - P)) as usize;
    return min(level, MAX_LEVEL);
}
