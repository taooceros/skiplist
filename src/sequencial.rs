use rand::{thread_rng, seq::SliceRandom};

use self::skiplist::SkipList;

mod skiplist;

pub fn test_sequencial_skiplist() {
    let mut skiplist = SkipList::new();

    // write some complicate test for the skiplist

    let mut rng = thread_rng();

    let length = 50000;

    let mut numbers = (1..length).collect::<Vec<_>>();

    numbers.shuffle(&mut rng);

    for i in numbers.iter() {
        skiplist.add(*i, *i);
    }

    for i in 1..length {
        assert_eq!(skiplist.get(i), Some(i).as_ref());
    }

    numbers.shuffle(&mut rng);

    for i in numbers {
        assert_eq!(skiplist.remove(i), Some(i));
    }
}
