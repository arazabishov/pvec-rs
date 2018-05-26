use pvec::rrbtree::BRANCH_FACTOR;
use pvec::rrbtree::RrbTree;
use std::fmt::Debug;
use std::mem;
use std::ops;

#[macro_use]
mod rrbtree;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct PVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

impl<T: Clone + Debug> PVec<T> {
    pub fn new() -> Self {
        PVec {
            tree: RrbTree::new(),
            tail: new_branch!(),
            tail_len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.tail[self.tail_len] = Some(item);
        self.tail_len += 1;

        if self.tail_len == self.tail.len() {
            let tail = mem::replace(&mut self.tail, new_branch!());

            self.tree.push(tail);
            self.tail_len = 0;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        if self.tail_len == 0 {
            self.tail_len = BRANCH_FACTOR;

            let new_tail = self.tree.pop();
            mem::replace(&mut self.tail, new_tail);
        }

        let item = self.tail[self.tail_len - 1].take();
        self.tail_len -= 1;

        return item;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if self.tree.len() > index {
            self.tree.get(index)
        } else {
            self.tail[index - self.tree.len()].as_ref()
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.tree.len() > index {
            self.tree.get_mut(index)
        } else {
            self.tail[index - self.tree.len()].as_mut()
        }
    }

    pub fn len(&self) -> usize {
        self.tree.len() + self.tail_len
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