use std::sync::atomic::{AtomicPtr, Ordering};

#[derive(Debug)]
pub struct MarkableAtomicPtr<T> {
    ptr: AtomicPtr<T>,
}

const MASK: usize = !0b1;

impl<T> MarkableAtomicPtr<T> {
    pub fn new(ptr: *mut T, mark: bool) -> Self {
        Self {
            ptr: AtomicPtr::new(ptr.map_addr(|p| p | mark as usize)),
        }
    }

    pub fn load_ptr(&self, order: Ordering) -> *mut T {
        self.ptr.load(order).mask(MASK)
    }

    pub fn load(&self, order: Ordering) -> (*mut T, bool) {
        let ptr = self.ptr.load(order);
        (ptr.mask(MASK), ptr.addr() & !MASK != 0)
    }

    pub fn store(&self, ptr: *mut T, mark: bool, order: Ordering) {
        self.ptr.store(ptr.map_addr(|p| p | mark as usize), order);
    }

    pub fn compare_exchange(
        &self,
        expect: *mut T,
        new: *mut T,
        expect_mark: bool,
        new_mark: bool,
        succ_order: Ordering,
        fail_order: Ordering,
    ) -> Result<*mut T, (*mut T, bool)> {
        self.ptr
            .compare_exchange(
                expect.map_addr(|p| p | expect_mark as usize),
                new.map_addr(|p| p | new_mark as usize),
                succ_order,
                fail_order,
            )
            .map(|p| p.mask(MASK))
            .map_err(|p| (p.mask(MASK), p.addr() & !MASK != 0))
    }
}

impl<T> Default for MarkableAtomicPtr<T> {
    fn default() -> Self {
        Self {
            ptr: AtomicPtr::default(),
        }
    }
}
