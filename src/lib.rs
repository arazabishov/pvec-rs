// TODO: abstract shift behind a struct
// TODO: re-consider usage of usize as a number type
// TODO: consider making NODE_CHILDREN_SIZE constant to be architecture aware (32, 64, etc)use std::sync::Arc;

use std::sync::Arc;

const MASK: usize = 0b11111;
const NODE_CHILDREN_SIZE: usize = 32;
const BITS_PER_LEVEL: usize = 5;

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

fn shift(key: usize, shift: usize) -> usize {
    return (key >> shift) & MASK;
}

enum Node<T> {
    Branch {
        children: [Option<Arc<Node<T>>>; NODE_CHILDREN_SIZE]
    },
    Leaf {
        elements: [Option<T>; NODE_CHILDREN_SIZE]
    },
}

struct PVec<T> {
    root: Node<T>,
    size: usize,
    tail: [Option<T>; NODE_CHILDREN_SIZE],
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
        assert_eq!(shift(141, 0), 0b01101);
        assert_eq!(shift(141, 5), 0b00100);
        assert_eq!(shift(141, 10), 0b00000);
        assert_eq!(shift(141, 15), 0b00000);
        assert_eq!(shift(141, 20), 0b00000);
        assert_eq!(shift(141, 25), 0b00000);
        assert_eq!(shift(141, 30), 0b00000);
        assert_eq!(shift(141, 25), 0b00000);
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