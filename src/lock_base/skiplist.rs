use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering::*};
use std::sync::{Mutex, MutexGuard};
use std::{
    cmp::{min, Ordering},
    ptr::null_mut,
    sync::atomic::AtomicPtr,
};

use self::entry::{Entry, Key};
use parking_lot::ReentrantMutex;
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
    K: Ord + Debug,
{
    pub fn new() -> Self {
        SkipList::with_cmp(default_cmp)
    }
}

unsafe impl<K, V, C> Send for SkipList<K, V, C>
where
    K: Ord + Send,
    V: Send,
    C: Fn(&K, &K) -> Ordering + Send,
{
}

unsafe impl<K, V, C> Sync for SkipList<K, V, C>
where
    K: Ord + Sync,
    V: Sync,
    C: Fn(&K, &K) -> Ordering + Sync,
{
}

impl<K, V, C> SkipList<K, V, C>
where
    K: Ord + Debug,
    C: Fn(&K, &K) -> Ordering,
{
    pub fn with_cmp(cmp: C) -> Self {
        let head = Box::into_raw(Box::new(Entry {
            key: Key::Head,
            value: None,
            lock: ReentrantMutex::new(()),
            marked: AtomicBool::new(false),
            fully_linked: AtomicBool::new(false),
            top_level: MAX_LEVEL,
            nexts: (1..=MAX_LEVEL).map(|_| AtomicPtr::default()).collect(),
        }));

        let tail = Box::into_raw(Box::new(Entry {
            key: Key::Tail,
            value: None,
            lock: ReentrantMutex::new(()),
            marked: AtomicBool::new(false),
            fully_linked: AtomicBool::new(false),
            top_level: MAX_LEVEL,
            nexts: (1..=MAX_LEVEL).map(|_| AtomicPtr::default()).collect(),
        }));

        unsafe {
            for level in 0..MAX_LEVEL {
                (*head).nexts[level].store(tail, Relaxed);
            }
        }

        SkipList { head, key_cmp: cmp }
    }

    pub fn add(&self, key: K, value: V) -> bool {
        let top_level = random_level();
        let mut preds = vec![null_mut(); MAX_LEVEL + 1];
        let mut succs = vec![null_mut(); MAX_LEVEL + 1];

        let key = Key::Entry(key);

        loop {
            let level_found = self.find(&key, &mut preds, &mut succs);

            if level_found.is_some() {
                unsafe {
                    let entry_found = &*succs[level_found.unwrap()];
                    if !entry_found.marked.load(Relaxed) {
                        while !entry_found.fully_linked.load(Relaxed) {}
                        return false;
                    }
                }

                return false;
            }

            let mut valid = true;
            let mut guards = Vec::with_capacity(top_level + 1);

            for level in 0..=top_level {
                unsafe {
                    let pred = &mut *preds[level];
                    let succ = &mut *succs[level];

                    guards.push(pred.lock.lock());

                    valid = !pred.marked.load(Relaxed)
                        && !(*succ).marked.load(Relaxed)
                        && pred.nexts[level].load(Relaxed) == succ;

                    if !valid {
                        break;
                    }
                }
            }

            if !valid {
                // I suppose guards will be dropped here
                continue;
            }

            let new_entry = Box::into_raw(Box::new(Entry {
                key,
                value: Some(value),
                lock: ReentrantMutex::new(()),
                marked: AtomicBool::new(false),
                fully_linked: AtomicBool::new(false),
                top_level,
                nexts: (0..=top_level).map(|_| AtomicPtr::default()).collect(),
            }));

            unsafe {
                for level in 0..=top_level {
                    (*new_entry).nexts[level].store(succs[level], Relaxed);
                    (*preds[level]).nexts[level].store(new_entry, Relaxed);
                }

                (*new_entry).fully_linked.store(true, Relaxed);
            }

            return true;
        }
    }

    pub fn remove(&self, key: K) -> Option<V> {
        // let mut victim = None;
        let mut is_marked = false;
        let mut top_level = usize::MAX;

        let mut preds = vec![null_mut(); MAX_LEVEL + 1];
        let mut succs = vec![null_mut(); MAX_LEVEL + 1];

        let key = Key::Entry(key);
        loop {
            let level_found = self.find(&key, &mut preds, &mut succs);

            let mut _victim_handle = None;

            if let Some(level_found) = level_found {
                let victim = unsafe { &mut *succs[level_found] };

                if is_marked
                    || (victim.fully_linked.load(Relaxed)
                        && victim.top_level == level_found
                        && !victim.marked.load(Relaxed))
                {
                    if !is_marked {
                        top_level = victim.top_level;
                        _victim_handle = Some(&victim.lock.lock());
                        if victim.marked.load(Relaxed) {
                            return None;
                        }

                        victim.marked.store(true, Relaxed);
                        is_marked = true;
                    }
                }

                let mut highest_locked = usize::MAX;

                let mut valid = true;

                let mut guards = Vec::with_capacity(top_level + 1);

                for level in 0..=top_level {
                    unsafe {
                        let pred = &mut *preds[level];

                        guards.push(pred.lock.lock());
                        if highest_locked < level {
                            highest_locked = level;
                        }

                        valid =
                            !pred.marked.load(Relaxed) && pred.nexts[level].load(Relaxed) == (victim);

                        if !valid {
                            break;
                        }
                    }
                }

                if !valid {
                    continue;
                }

                unsafe {
                    for level in 0..=top_level {
                        (*preds[level]).nexts[level]
                            .store((*succs[level]).nexts[level].load(Relaxed), Relaxed);
                    }
                }

                return victim.value.take();
            } else {
                return None;
            }
        }
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
            let mut current = unsafe { pred.nexts[level].load(Relaxed).as_mut().unwrap() };

            while current.key < *key {
                pred = current;
                current = unsafe { pred.nexts[level].load(Relaxed).as_mut().unwrap() };
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
        let mut pred = self.head;

        let key = Key::Entry(key);

        for level in (0..MAX_LEVEL).rev() {
            let mut current = unsafe { (*pred).nexts[level].load(Relaxed).as_mut().unwrap() };

            while current.key < key {
                pred = current;
                current = unsafe { (*current).nexts[level].load(Relaxed).as_mut().unwrap() };
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
