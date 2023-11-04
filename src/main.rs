use skiplist::SkipList;

mod skiplist;

pub fn main() {
    let mut skiplist = SkipList::new();

    for i in 1..10 {
        skiplist.add(i, i);
    }

    for i in 1..11 {
        println!("{:?}", skiplist.remove(i));
    }
}
