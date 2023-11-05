use lock_base::test_lockbase_skiplist;
use sequencial::test_sequencial_skiplist;

mod lock_base;
mod sequencial;

pub fn main() {
    test_sequencial_skiplist();

    test_lockbase_skiplist();
}
