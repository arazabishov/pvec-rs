use std::sync::Arc;

#[cfg(not(small_branch))]
const BRANCH_FACTOR: usize = 32;

#[cfg(small_branch)]
const BRANCH_FACTOR: usize = 4;

#[cfg(not(small_branch))]
const BITS_PER_LEVEL: usize = 5;

#[cfg(small_branch)]
const BITS_PER_LEVEL: usize = 2;

#[cfg(not(small_branch))]
macro_rules! no_children {
    () => {
        [None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,]
    }
}

#[cfg(small_branch)]
macro_rules! no_children {
    () => {
        [None, None, None, None]
    }
}

#[derive(Copy, Clone)]
struct Shift(usize);

#[derive(Copy, Clone)]
struct Index(usize);

impl Shift {
    fn inc(self) -> Shift {
        Shift(self.0 + BITS_PER_LEVEL)
    }
}

impl Index {
    fn child(self, shift: Shift) -> usize {
        (self.0 >> shift.0) & BRANCH_FACTOR - 1
    }
}

enum Node<T> {
    Branch {
        children: [Option<Arc<Node<T>>>; BRANCH_FACTOR]
    },
    Leaf {
        elements: [Option<T>; BRANCH_FACTOR]
    },
}

struct PVec<T> {
    root: Node<T>,
    size: usize,
    tail: [Option<T>; BRANCH_FACTOR],
}

impl<T> PVec<T> {
    pub fn new() -> Self {
        PVec {
            root: Node::Branch { children: no_children!() },
            size: 0,
            tail: no_children!(),
        }
    }

    pub fn push(&mut self, item: T) {
        let index = self.size.clone();
        let mut tail = &mut self.tail;
        let mut size = &mut self.size;

        *size = *size + 1;
        tail[index] = Some(item);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        return self.tail[index].as_ref();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_must_return_correct_index() {
        let index = Index(141);

        let shift_0 = Shift(0);
        let shift_1 = shift_0.inc();
        let shift_2 = shift_1.inc();
        let shift_3 = shift_2.inc();
        let shift_4 = shift_3.inc();
        let shift_5 = shift_4.inc();
        let shift_6 = shift_5.inc();
        let shift_7 = shift_6.inc();

        assert_eq!(index.child(shift_0), 0b01101);
        assert_eq!(index.child(shift_1), 0b00100);
        assert_eq!(index.child(shift_2), 0b00000);
        assert_eq!(index.child(shift_3), 0b00000);
        assert_eq!(index.child(shift_4), 0b00000);
        assert_eq!(index.child(shift_5), 0b00000);
        assert_eq!(index.child(shift_6), 0b00000);
        assert_eq!(index.child(shift_7), 0b00000);
    }

    #[test]
    fn new_must_return_correctly_initialized_pvec_instance() {
        let mut vec = PVec::new();
        vec.push("zero");
        vec.push("one");
        vec.push("two");

        assert_eq!(*vec.get(0).unwrap(), "zero");
        assert_eq!(*vec.get(1).unwrap(), "one");
        assert_eq!(*vec.get(2).unwrap(), "two");
    }
}