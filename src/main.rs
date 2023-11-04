use skiplist::SkipList;

mod skiplist;

pub fn main() {
    let mut skiplist = SkipList::new();

    skiplist.add(1, 1);
    skiplist.add(2, 2);
    skiplist.add(3, 3);
    skiplist.add(4, 4);
}
