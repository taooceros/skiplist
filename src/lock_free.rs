mod markable_atomic_ptr;
mod skiplist;

use std::thread;

use rand::{seq::SliceRandom, thread_rng};

use self::skiplist::SkipList;

pub fn test_lockfree_skiplist() {
    let skiplist = &SkipList::new();
    thread::scope(|s| {
        // write some complicate test for the skiplist

        let handles = (1..32)
            .map(|i| {
                s.spawn(move || {
                    let length = 50000;

                    let mut numbers = (1..length).collect::<Vec<_>>();

                    let rng = &mut thread_rng();

                    numbers.shuffle(rng);

                    for j in numbers {
                        skiplist.add((i * length) + j, i * j);
                    }

                    // for j in 1..1000 {
                    //     let value = skiplist.get(i * j).unwrap();
                    //     assert_eq!(*value, i * j);
                    // }

                    for j in 1..length {
                        let item = skiplist.remove((i * length) + j);

                        assert_eq!(item.unwrap(), i * j);
                    }
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }
    })
}
