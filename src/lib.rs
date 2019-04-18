#![feature(nll)]

#[macro_use]
extern crate serde_json;
extern crate serde;

use rrbtree::RrbTree;
use rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;
use std::mem;
use std::ops;

#[macro_use]
mod rrbtree;
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

#[derive(Debug, Clone)]
pub struct PVecIter<T> {
    // - you should avoid heap allocation in iterators
    pvec: PVec<T>,
    len: usize,
}

impl<T: Clone + Debug> Iterator for PVecIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T: Clone + Debug> IntoIterator for PVec<T> {
    type Item = T;
    type IntoIter = PVecIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        PVecIter {
            len: self.len(),
            pvec: self,
        }
    }
}

mod serializer {
    extern crate serde;
    extern crate serde_json;

    use self::serde::ser::{Serialize, SerializeStruct, Serializer};
    use super::PVec;

    impl<T> Serialize for PVec<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
        where
            S: Serializer,
        {
            let mut serde_state = serializer.serialize_struct("PVec", 1)?;
            serde_state.serialize_field("tree", &self.tree)?;
            serde_state.serialize_field("tail", &self.tail)?;
            serde_state.serialize_field("tail_len", &self.tail_len)?;
            serde_state.end()
        }
    }
}
