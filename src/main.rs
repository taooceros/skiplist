use core::num;

use rand::{rngs::mock::StepRng, seq::SliceRandom, thread_rng};
use skiplist::SkipList;

mod skiplist;

pub fn main() {
    let mut skiplist = SkipList::new();

    // write some complicate test for the skiplist

    let mut rng = thread_rng();

    let mut numbers = (1..1000).collect::<Vec<_>>();

    numbers.shuffle(&mut rng);

    for i in numbers.iter() {
        skiplist.add(*i, *i);
    }

    for i in 1..1000 {
        assert_eq!(skiplist.get(i), Some(i).as_ref());
    }

    numbers.shuffle(&mut rng);

    for i in numbers {
        assert_eq!(skiplist.remove(i), Some(i));
    }

}
