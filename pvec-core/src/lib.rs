#[macro_use]
#[cfg(feature = "serde-serializer")]
extern crate serde_json;

extern crate pvec_utils as utils;
#[cfg(all(feature = "arc", feature = "rayon-iter"))]
extern crate rayon;

#[cfg(feature = "serde-serializer")]
extern crate serde;

use rrbtree::RrbTree;
use rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;
use std::mem;
use std::ops;

pub mod iter;

#[macro_use]
mod rrbtree;

#[cfg(feature = "serde-serializer")]
mod serializer;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

impl<T: Clone + Debug> RrbVec<T> {
    pub fn new() -> Self {
        RrbVec {
            tree: RrbTree::new(),
            tail: new_branch!(),
            tail_len: 0,
        }
    }

    pub fn from(vec: &[T]) -> Self {
        // ToDo: do something smart to pre-allocate space for all new items?

        let vec = vec.to_owned();
        let mut rrbvec = RrbVec::new();

        for item in vec.into_iter() {
            rrbvec.push(item)
        }

        rrbvec
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
        if mid == 0 {
            mem::replace(self, RrbVec::new())
        } else if mid < self.len() {
            if self.tree.len() > mid {
                let right_tree = self.tree.split_off(mid);

                let (left_tail, left_tail_len) = self.tree.pop();

                let right_tail = mem::replace(&mut self.tail, left_tail);
                let right_tail_len = mem::replace(&mut self.tail_len, left_tail_len);

                let mut right = RrbVec {
                    tree: right_tree,
                    tail: right_tail,
                    tail_len: right_tail_len,
                };

                if right.tree.is_root_leaf() {
                    if right.len() <= BRANCH_FACTOR {
                        // all values can fit into a single tail
                        let (mut new_tail, mut new_tail_len) = right.tree.pop();

                        for i in 0..right.tail_len {
                            new_tail[new_tail_len] = right.tail[i].take();
                            new_tail_len += 1;
                        }

                        right.tail = new_tail;
                        right.tail_len = new_tail_len;
                    } else if right.tree.len() < BRANCH_FACTOR {
                        // root is leaf, but it is not fully dense
                        // hence, some of the values should be redistributed to the actual leaf

                        let (mut root, mut root_len) = right.tree.pop();
                        let mut index = 0;

                        while root_len < BRANCH_FACTOR && index < right.tail_len {
                            root[root_len] = right.tail[index].take();

                            root_len += 1;
                            index += 1;
                        }

                        right.tree.push(root, root_len);

                        let (mut new_tail, mut new_tail_len) = (new_branch!(), 0);
                        while index < right.tail_len {
                            new_tail[new_tail_len] = right.tail[index].take();

                            new_tail_len += 1;
                            index += 1;
                        }

                        right.tail = new_tail;
                        right.tail_len = new_tail_len;
                    }
                }

                right
            } else {
                let left_tail_len = mid - self.tree.len();

                let mut right_tail = new_branch!();
                let mut right_tail_len = 0;

                for i in left_tail_len..self.tail_len {
                    right_tail[right_tail_len] = self.tail[i].take();
                    right_tail_len += 1;
                }

                self.tail_len = left_tail_len;

                RrbVec {
                    tree: RrbTree::new(),
                    tail: right_tail,
                    tail_len: right_tail_len,
                }
            }
        } else if mid == self.len() {
            RrbVec::new()
        } else {
            panic!()
        }
    }

    pub fn append(&mut self, that: &mut RrbVec<T>) {
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
                    for item in that_tail.iter_mut().take(that_tail_len) {
                        self.tail[self.tail_len] = item.take();
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

impl<T: Clone + Debug> Default for RrbVec<T> {
    fn default() -> Self {
        RrbVec::new()
    }
}

impl<T: Clone + Debug> ops::Index<usize> for RrbVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "index `{}` out of bounds in RrbVec of length `{}`",
                index,
                self.len()
            )
        })
    }
}

impl<T: Clone + Debug> ops::IndexMut<usize> for RrbVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        let len = self.len();
        self.get_mut(index).unwrap_or_else(|| {
            panic!(
                "index `{}` out of bounds in RrbVec of length `{}`",
                index, len
            )
        })
    }
}
