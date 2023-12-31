use std::thread;

use rand::{seq::SliceRandom, thread_rng};

use self::skiplist::SkipList;

mod skiplist;

pub fn test_lockbase_skiplist() {
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

                    for j in 1..1000 {
                        assert_eq!(skiplist.remove((i * length) + j).unwrap(), i * j);
                    }
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }
    })
}
