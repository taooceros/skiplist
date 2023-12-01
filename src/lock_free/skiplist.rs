use core::panic;
use std::backtrace::Backtrace;
use std::fmt::Debug;
use std::intrinsics::breakpoint;
use std::ptr::null;
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

const MASK: usize = !0b1;

fn extract_ptr<'a, T>(ptr: *mut T) -> (&'a mut T, bool) {
    if ptr.is_null() {
        panic!("ptr is null {}", Backtrace::force_capture());
    }

    unsafe { (ptr.mask(MASK).as_mut().unwrap(), ptr.addr() & !MASK != 0) }
}

fn mark_ptr<T>(ptr: *mut T) -> *mut T {
    (ptr as *mut T).map_addr(|p| p | !MASK)
}

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
        unsafe {
            let head = &mut *Box::into_raw(Box::new(Entry::new(Key::Head, None, MAX_LEVEL)));
            let tail = &mut *Box::into_raw(Box::new(Entry::new(Key::Tail, None, MAX_LEVEL)));

            for level in 0..=MAX_LEVEL {
                (*head).nexts[level].store(tail, false, Relaxed);
            }

            SkipList { head, key_cmp: cmp }
        }
    }

    pub fn add(&self, key: K, value: V) -> bool {
        let top_level = random_level();
        let bottom_level = 0;
        let mut preds = [null_mut(); MAX_LEVEL + 1];
        let mut succs = [null_mut(); MAX_LEVEL + 1];

        let key = Key::Entry(key);
        let new_entry = Box::into_raw(Box::new(Entry::new(key, Some(value), top_level)));
        let key_ref = unsafe { &(*new_entry).key };

        loop {
            let found = self.find(key_ref, &mut preds, &mut succs);

            if found {
                return false;
            }

            unsafe {
                for level in bottom_level..=top_level {
                    (*new_entry).nexts[level].store(succs[level], false, Relaxed);
                }

                let pred = &mut *preds[bottom_level];
                let succ = &mut *succs[bottom_level];

                if !pred.nexts[bottom_level]
                    .compare_exchange(succ, new_entry, false, false, Release, Relaxed)
                    .is_ok()
                {
                    continue;
                }

                // TODO: why bottom level?
                for level in bottom_level + 1..=top_level {
                    loop {
                        let pred = &mut *preds[level];
                        let succ = &mut *succs[level];

                        if pred.nexts[level]
                            .compare_exchange(succ, new_entry, false, false, Release, Relaxed)
                            .is_ok()
                        {
                            break;
                        }

                        self.find(key_ref, &mut preds, &mut succs);
                    }
                }
            }

            return true;
        }
    }

    pub fn remove(&self, key: K) -> Option<V> {
        let bottom_level = 0;

        let mut preds = [null_mut(); MAX_LEVEL + 1];
        let mut succs = [null_mut(); MAX_LEVEL + 1];

        let key = Key::Entry(key);
        loop {
            let found = self.find(&key, &mut preds, &mut succs);

            if !found {
                return None;
            }

            let node_to_remove = unsafe { &mut *succs[bottom_level] };

            for level in (bottom_level + 1..=node_to_remove.top_level).rev() {
                let (mut ptr, mut marked) = node_to_remove.nexts[level].load(Acquire);

                while !marked {
                    let _ = node_to_remove.nexts[level]
                        .compare_exchange(ptr, ptr, false, true, Release, Relaxed);
                    (ptr, marked) = node_to_remove.nexts[level].load(Acquire);
                }
            }

            let (mut succ, _) = node_to_remove.nexts[bottom_level].load(Acquire);

            loop {
                match node_to_remove.nexts[bottom_level]
                    .compare_exchange(succ, succ, false, true, Release, Acquire)
                {
                    Ok(_) => {
                        self.find(&key, &mut preds, &mut succs);
                        return Some(node_to_remove.value.take().unwrap());
                    }
                    Err((actual_succ, marked)) => {
                        succ = actual_succ;
                        if marked {
                            return None;
                        }
                    }
                };
            }
        }
    }

    fn find<'a, 'b, const N: usize>(
        &'a self,
        key: &Key<K>,
        preds: &'b mut [*mut Entry<K, V, C>; N],
        succs: &'b mut [*mut Entry<K, V, C>; N],
    ) -> bool
    where
        'a: 'b,
    {
        let bottom_level = 0;

        let mut current_ptr = null_mut();
        let mut marked = false;

        let mut tries = 0;

        'retry: loop {
            tries += 1;

            if tries > 100 {
                panic!("try to much");
            }

            let mut pred = self.head;
            for level in (bottom_level..=MAX_LEVEL).rev() {
                current_ptr = unsafe { (*pred).nexts[level].load_ptr(Acquire) };

                loop {
                    if current_ptr.is_null() {
                        break;
                    }

                    let mut current = unsafe { current_ptr.as_mut().unwrap() };
                    let (mut succ_ptr, mut marked) = current.nexts[level].load(Acquire);

                    while marked {
                        unsafe {
                            match (*pred).nexts[level].compare_exchange(
                                current_ptr,
                                succ_ptr,
                                false,
                                false,
                                Release,
                                Acquire,
                            ) {
                                Ok(_) => {
                                    current_ptr = (*pred).nexts[level].load_ptr(Acquire);
                                    if current_ptr.is_null() {
                                        return false;
                                    }
                                    current = current_ptr.as_mut().unwrap();
                                    (succ_ptr, marked) = current.nexts[level].load(Acquire);
                                }
                                Err((actual, _)) => {
                                    // println!(
                                    //     "actual: {:p}, current_ptr: {:p}",
                                    //     actual, current_ptr
                                    // );
                                    continue 'retry;
                                }
                            }
                        }
                    }

                    if current.key < *key && !succ_ptr.is_null() {
                        pred = current;
                        current_ptr = succ_ptr;
                    } else {
                        break;
                    }
                }

                preds[level] = pred;
                succs[level] = current_ptr;
            }

            return unsafe { !current_ptr.is_null() && (*current_ptr).key == *key };
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let mut pred = self.head;

        let key = Key::Entry(key);

        for level in (0..MAX_LEVEL).rev() {
            let (mut current_ptr, mut marked) = unsafe { (*pred).nexts[level].load(Relaxed) };
            let mut current = unsafe { current_ptr.as_mut().unwrap() };

            todo!("");
        }

        return None;
    }
}

static P: f32 = 0.5;
const MAX_LEVEL: usize = 32;

fn random_level() -> usize {
    let level = (f32::log2(1. - random::<f32>()) / f32::log2(1. - P)) as usize;
    return min(level, MAX_LEVEL);
}
