use sequencial::test_sequencial_skiplist;

mod sequencial;
mod lock_base;

pub fn main() {
    test_sequencial_skiplist();
}
