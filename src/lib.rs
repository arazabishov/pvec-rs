use std::ops;
use std::mem;
use std::sync::Arc;
use std::cmp::Ordering;
use std::fmt::Debug;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Shift(usize);

impl Shift {
    fn inc(self) -> Shift {
        Shift(self.0 + BITS_PER_LEVEL)
    }

    fn dec(self) -> Shift {
        Shift(self.0 - BITS_PER_LEVEL)
    }
}

impl PartialEq<usize> for Shift {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<usize> for Shift {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

impl PartialEq<usize> for Index {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<usize> for Index {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Index {
    fn child(self, shift: Shift) -> usize {
        (self.0 >> shift.0) & BRANCH_FACTOR - 1
    }

    fn element(self) -> usize {
        self.0 & BRANCH_FACTOR - 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Node<T> {
    Branch {
        children: [Option<Arc<Node<T>>>; BRANCH_FACTOR]
    },
    Leaf {
        elements: [Option<T>; BRANCH_FACTOR]
    },
}

impl<T: Clone> Node<T> {
    fn push_tail(&mut self, index: Index, shift: Shift, tail: [Option<T>; BRANCH_FACTOR]) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let mut node = self;
        let mut shift = shift;

        while shift.0 != BITS_PER_LEVEL {
            let cnode = node; // FIXME: NLL

            let child = match *cnode {
                Node::Leaf { .. } => unreachable!(),
                Node::Branch { ref mut children } => {
                    let i = index.child(shift);

                    if children[i].is_none() {
                        children[i] = Some(Arc::new(Node::Branch {
                            children: no_children!()
                        }));
                    }

                    children[i].as_mut().unwrap()
                }
            };

            node = Arc::make_mut(child);
            shift = shift.dec();
        }

        debug_assert_eq!(shift.0, BITS_PER_LEVEL);

        if let Node::Branch { ref mut children } = *node {
            children[index.child(shift)] = Some(Arc::new(Node::Leaf { elements: tail }));
        }
    }

    pub fn get(&self, index: Index, shift: Shift) -> Option<&T> {
        let mut node = self;
        let mut shift = shift;

        loop {
            match *node {
                Node::Branch { ref children } => {
                    debug_assert!(shift.0 > 0);
                    node = match children[index.child(shift)] {
                        Some(ref child) => &*child,
                        None => unreachable!()
                    };

                    shift = shift.dec();
                }
                Node::Leaf { ref elements } => {
                    debug_assert_eq!(shift.0, 0);
                    return elements[index.element()].as_ref();
                }
            }
        }
    }

    pub fn get_mut(&mut self, index: Index, shift: Shift) -> Option<&mut T> {
        let mut node = self;
        let mut shift = shift;

        loop {
            let cnode = node; // FIXME: NLL
            match *cnode {
                Node::Branch { ref mut children } => {
                    debug_assert!(shift.0 > 0);
                    node = match children[index.child(shift)] {
                        Some(ref mut child) => Arc::make_mut(child),
                        None => unreachable!()
                    };

                    shift = shift.dec();
                }
                Node::Leaf { ref mut elements } => {
                    debug_assert_eq!(shift.0, 0);
                    return elements[index.element()].as_mut();
                }
            }
        }
    }
}

// TODO: consider comparing performance of PVec where tail is backed by the Vec or plain array
struct PVec<T> {
    root: Option<Arc<Node<T>>>,
    root_size: Index,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_size: Index,
    shift: Shift,
}

impl<T: Clone + Debug> PVec<T> {
    pub fn new() -> Self {
        PVec {
            root: None,
            root_size: Index(0),
            tail: no_children!(),
            tail_size: Index(0),
            shift: Shift(0),
        }
    }

    pub fn push(&mut self, item: T) {
        self.tail[self.tail_size.0] = Some(item);
        self.tail_size.0 += 1;

        if self.tail_size.0 == BRANCH_FACTOR {
            let tail = mem::replace(&mut self.tail, no_children!());

            self.push_tail(tail);

            self.root_size.0 += BRANCH_FACTOR;
            self.tail_size.0 = 0;
            self.tail = no_children!();
        }
    }

    pub fn len(&self) -> usize {
        self.root_size.0 + self.tail_size.0
    }

    fn push_tail(&mut self, tail: [Option<T>; BRANCH_FACTOR]) {
        if self.root.is_none() {
            self.root = Some(Arc::new(Node::Branch { children: no_children!() }));
        }

        if let Some(root) = self.root.as_mut() {
            let capacity = BRANCH_FACTOR << self.shift.0;

            if capacity == self.root_size.0 + BRANCH_FACTOR {
                println!("Need to grow this thingy.");

                let mut nodes = no_children!();
                nodes[0] = Some(root.clone());

                self.shift = self.shift.inc();

                *root = Arc::new(Node::Branch { children: nodes });
            }

            Arc::make_mut(root).push_tail(self.root_size, self.shift, tail);
        } else {
            unreachable!()
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if self.root_size.0 > index {
            self.root.as_ref().unwrap().get(Index(index), self.shift)
        } else {
            self.tail[index - self.root_size.0].as_ref()
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.root_size.0 > index {
            Arc::make_mut(self.root.as_mut().unwrap()).get_mut(Index(index), self.shift)
        } else {
            self.tail[index - self.root_size.0].as_mut()
        }
    }
}

impl<T: Clone + Debug> ops::Index<usize> for PVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).unwrap_or_else(||
            panic!("index `{}` out of bounds in PVec of length `{}`", index, self.len())
        )
    }
}

impl<T: Clone + Debug> ops::IndexMut<usize> for PVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        let len = self.len();
        self.get_mut(index).unwrap_or_else(||
            panic!("index `{}` out of bounds in PVec of length `{}`", index, len)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(small_branch))]
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

        assert_eq!(index.element(), 0b01101);
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

        for i in 0..64 {
            vec.push(i);
        }

        for i in 0..64 {
            assert_eq!(vec[i], i);
        }

        assert_eq!(vec.len(), 64);
    }
}