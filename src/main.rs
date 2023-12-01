#![feature(ptr_mask)]
#![feature(strict_provenance)]
#![feature(core_intrinsics)]

use lock_base::test_lockbase_skiplist;
use lock_free::test_lockfree_skiplist;
use sequencial::test_sequencial_skiplist;

mod lock_base;
mod lock_free;
mod sequencial;

fn measure_time<F: Fn()>(f: F) {
    let start = std::time::Instant::now();

    f();

    let end = std::time::Instant::now();

    println!("time: {:?}", end - start);
}

pub fn main() {
    measure_time(|| {
        test_sequencial_skiplist();
    });

    measure_time(|| {
        test_lockbase_skiplist();
    });

    measure_time(|| {
        test_lockfree_skiplist();
    });
}
