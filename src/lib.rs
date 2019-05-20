#![feature(nll)]

#[macro_use]
extern crate serde_json;
extern crate serde;

use rrbtree::RrbTree;
use rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;
use std::mem;
use std::ops;

pub mod iter;

#[macro_use]
mod rrbtree;
mod serializer;
mod sharedptr;

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

    #[cold]
    pub fn push(&mut self, item: T) {
        self.tail[self.tail_len] = Some(item);
        self.tail_len += 1;

        self.push_tail();
    }

    #[inline(always)]
    fn push_tail(&mut self) {
        if self.tail_len == BRANCH_FACTOR {
            let tail = mem::replace(&mut self.tail, new_branch!());

            self.tree.push(tail, self.tail_len);
            self.tail_len = 0;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        if self.tail_len == 0 {
            let (new_tail, new_tail_len) = self.tree.pop();
            mem::replace(&mut self.tail, new_tail);

            self.tail_len = new_tail_len;
        }

        let item = self.tail[self.tail_len - 1].take();
        self.tail_len -= 1;

        item
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn split_off(&mut self, mid: usize) -> Self {
        let cloned = self.clone();
        self.split_right_at(mid);

        cloned.split_left_at(mid)
    }

    // ToDo: reconsider implementation of tree.split_at_* to
    // ToDo: avoid additional cloning, copying, etc
    fn split_right_at(&mut self, mid: usize) {
        // ToDo: see other adjustments which are
        // ToDo: made to tail after tree is split

        if mid == 0 {
            mem::replace(self, PVec::new());
        } else if mid < self.len() {
            // only need to cut the tail off
            if self.tree.len() <= mid {
                let new_tail_len = mid - self.tree.len();

                for i in new_tail_len..self.tail_len {
                    self.tail[i] = None;
                }

                self.tail_len = new_tail_len
            } else {
                let mut new_tree = self.tree.split_right_at(mid);
                let (new_tail, new_tail_len) = new_tree.pop();

                self.tree = new_tree;
                self.tail = new_tail;
                self.tail_len = new_tail_len;
            }
        } else if mid > self.len() {
            panic!()
        }
    }

    // ToDo: see other adjustments which are
    // ToDo: made to tail after tree is split
    fn split_left_at(mut self, mid: usize) -> Self {
        if mid == 0 {
            self
        } else if mid < self.len() {
            let remaining = self.len() - mid;

            if remaining <= self.tail_len {
                let mut tail = new_branch!();
                let mut tail_len = 0;

                for i in (self.tail_len - remaining)..self.tail_len {
                    tail[tail_len] = self.tail[i].take();
                    tail_len += 1;
                }

                return PVec {
                    tree: self.tree,
                    tail: tail,
                    tail_len: tail_len,
                };
            }

            let mut pvec = PVec {
                tree: self.tree.split_left_at(mid),
                tail: self.tail,
                tail_len: self.tail_len,
            };

            if pvec.tree.is_root_leaf() {
                if pvec.len() <= BRANCH_FACTOR {
                    // all values can fit into a single tail
                    let (mut new_tail, mut new_tail_len) = pvec.tree.pop();

                    for i in 0..pvec.tail_len {
                        new_tail[new_tail_len] = pvec.tail[i].take();
                        new_tail_len += 1;
                    }

                    pvec.tail = new_tail;
                    pvec.tail_len = new_tail_len;
                } else if pvec.tree.len() < BRANCH_FACTOR {
                    // root is leaf, but it is not fully dense
                    // hence, some of the values should be redistributed to the actual leaf

                    let (mut root, mut root_len) = pvec.tree.pop();
                    let mut index = 0;

                    while root_len < BRANCH_FACTOR && index < pvec.tail_len {
                        root[root_len] = pvec.tail[index].take();

                        root_len += 1;
                        index += 1;
                    }

                    pvec.tree.push(root, root_len);

                    let (mut new_tail, mut new_tail_len) = (new_branch!(), 0);
                    while index < pvec.tail_len {
                        new_tail[new_tail_len] = pvec.tail[index].take();

                        new_tail_len += 1;
                        index += 1;
                    }

                    pvec.tail = new_tail;
                    pvec.tail_len = new_tail_len;
                }
            }

            pvec
        } else if mid == self.len() {
            PVec::new()
        } else {
            panic!()
        }
    }

    pub fn append(&mut self, that: &mut PVec<T>) {
        if self.is_empty() {
            self.tail = mem::replace(&mut that.tail, new_branch!());
            self.tree = mem::replace(&mut that.tree, RrbTree::new());

            self.tail_len = that.tail_len;
            that.tail_len = 0;
        } else if !that.is_empty() {
            let mut that_tail = mem::replace(&mut that.tail, new_branch!());
            let that_tail_len = that.tail_len;

            that.tail_len = 0;

            if that.tree.is_empty() {
                if self.tail_len == BRANCH_FACTOR {
                    let self_tail = mem::replace(&mut self.tail, that_tail);
                    let self_tail_len = self.tail_len;

                    self.tail_len = that_tail_len;
                    self.tree.push(self_tail, self_tail_len);
                } else if self.tail_len + that_tail_len <= BRANCH_FACTOR {
                    for i in 0..that_tail_len {
                        self.tail[self.tail_len] = that_tail[i].take();
                        self.tail_len += 1;
                    }
                } else {
                    let mut self_tail = mem::replace(&mut self.tail, new_branch!());
                    let mut self_tail_i = mem::replace(&mut self.tail_len, 0);
                    let mut that_tail_i = 0;

                    while self_tail_i < BRANCH_FACTOR && that_tail_i < that_tail_len {
                        self_tail[self_tail_i] = that_tail[that_tail_i].take();

                        self_tail_i += 1;
                        that_tail_i += 1;
                    }

                    self.tree.push(self_tail, self_tail_i);

                    let that_tail_elements_left = that_tail_len - that_tail_i;
                    for i in 0..that_tail_elements_left {
                        self.tail[i] = that_tail[that_tail_i].take();
                        that_tail_i += 1;
                    }

                    self.tail_len = that_tail_elements_left;
                }
            } else {
                if self.tail_len == 0 {
                    self.tail = that_tail;
                    self.tail_len = that_tail_len;
                } else {
                    let self_tail = mem::replace(&mut self.tail, that_tail);
                    let self_tail_len = self.tail_len;

                    self.tail_len = that_tail_len;
                    self.tree.push(self_tail, self_tail_len);
                }

                self.tree.append(&mut that.tree);
            }
        }

        self.push_tail();
    }
}

impl<T: Clone + Debug> Default for PVec<T> {
    fn default() -> Self {
        PVec::new()
    }
}

impl<T: Clone + Debug> ops::Index<usize> for PVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "index `{}` out of bounds in PVec of length `{}`",
                index,
                self.len()
            )
        })
    }
}

impl<T: Clone + Debug> ops::IndexMut<usize> for PVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        let len = self.len();
        self.get_mut(index).unwrap_or_else(|| {
            panic!(
                "index `{}` out of bounds in PVec of length `{}`",
                index, len
            )
        })
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use super::{PVec, BRANCH_FACTOR};

    #[test]
    fn split_right_at() {
        let mut pvec = PVec::new();

        for i in 0..(BRANCH_FACTOR * BRANCH_FACTOR) {
            pvec.push(i);
        }

        for i in (0..BRANCH_FACTOR * BRANCH_FACTOR).rev() {
            pvec.split_right_at(i);
            assert_eq!(pvec.len(), i);

            for j in 0..i {
                assert_eq!(pvec.get(j).cloned(), Some(j));
            }
        }

        assert_eq!(pvec.tree.len(), 0);
        assert_eq!(pvec.tail_len, 0);
    }

    #[test]
    fn split_left_at() {
        let mut pvec = PVec::new();

        for i in 0..(BRANCH_FACTOR * BRANCH_FACTOR) {
            pvec.push(i);
        }

        let pvec_len = pvec.len();
        for i in 1..pvec_len + 1 {
            pvec = pvec.split_left_at(1);
            assert_eq!(pvec.len(), pvec_len - i);

            for j in 0..(pvec_len - i) {
                assert_eq!(pvec.get(j).cloned(), Some(i + j));
            }
        }

        assert_eq!(pvec.tree.len(), 0);
        assert_eq!(pvec.tail_len, 0);
    }

    #[test]
    fn interleaving_append_split_at_operations() {
        let mut pvec = PVec::new();
        let mut value = 0;

        for size in 1..(BRANCH_FACTOR * 8 + BRANCH_FACTOR) {
            let mut another_pvec = PVec::new();
            for _ in 0..size {
                another_pvec.push(value);
                value += 1;
            }

            pvec.append(&mut another_pvec);

            let mid = pvec.len() / 2;
            let mut right = pvec.split_off(mid);

            pvec.append(&mut right);
            value = pvec.len();
        }

        for i in 0..value {
            assert_eq!(pvec.get(i).cloned(), Some(i));
        }
    }
}
