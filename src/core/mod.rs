//! A module providing persistent vector types based on RrbTree.

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
extern crate rayon;

#[cfg(feature = "serde_serializer")]
extern crate serde;

use rrbtree::RrbTree;
use rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;
use std::mem;
use std::ops;

pub mod iter;
mod sharedptr;

#[macro_use]
mod rrbtree;

#[cfg(feature = "serde_serializer")]
pub mod serializer;

#[cfg(not(feature = "small_branch"))]
macro_rules! clone_arr {
    ($source:expr) => {{
        let s = $source;
        [
            Some(s[0x00].clone()),
            Some(s[0x01].clone()),
            Some(s[0x02].clone()),
            Some(s[0x03].clone()),
            Some(s[0x04].clone()),
            Some(s[0x05].clone()),
            Some(s[0x06].clone()),
            Some(s[0x07].clone()),
            Some(s[0x08].clone()),
            Some(s[0x09].clone()),
            Some(s[0x0A].clone()),
            Some(s[0x0B].clone()),
            Some(s[0x0C].clone()),
            Some(s[0x0D].clone()),
            Some(s[0x0E].clone()),
            Some(s[0x0F].clone()),
            Some(s[0x10].clone()),
            Some(s[0x11].clone()),
            Some(s[0x12].clone()),
            Some(s[0x13].clone()),
            Some(s[0x14].clone()),
            Some(s[0x15].clone()),
            Some(s[0x16].clone()),
            Some(s[0x17].clone()),
            Some(s[0x18].clone()),
            Some(s[0x19].clone()),
            Some(s[0x1A].clone()),
            Some(s[0x1B].clone()),
            Some(s[0x1C].clone()),
            Some(s[0x1D].clone()),
            Some(s[0x1E].clone()),
            Some(s[0x1F].clone()),
        ]
    }};
}

#[cfg(feature = "small_branch")]
macro_rules! clone_arr {
    ($source:expr) => {{
        let s = $source;
        [
            Some(s[0x00].clone()),
            Some(s[0x01].clone()),
            Some(s[0x02].clone()),
            Some(s[0x03].clone()),
        ]
    }};
}

/// A persistent vector based on the balanced RbTree.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RbVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

/// A persistent vector based on the relaxed RrbTree.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

macro_rules! impl_vec {
    ($vec:ident) => {
        impl<T: Clone + Debug> Default for $vec<T> {
            fn default() -> Self {
                $vec::new()
            }
        }

        impl<T: Clone + Debug> $vec<T> {
            /// Constructs a new, empty vector.
            /// The vector allocates a buffer equal to
            /// the selected branching factor size.
            pub fn new() -> Self {
                $vec {
                    tree: RrbTree::new(),
                    tail: new_branch!(),
                    tail_len: 0,
                }
            }

            /// Adds an element to the back of a collection.
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

            /// Removes the last element from a vector and
            /// returns it, or None if it is empty.
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

            /// Returns a reference to an element at the given
            /// position or None if out of bounds.
            pub fn get(&self, index: usize) -> Option<&T> {
                if self.tree.len() > index {
                    self.tree.get(index)
                } else {
                    self.tail[index - self.tree.len()].as_ref()
                }
            }

            /// Returns a mutable reference to an element at the given
            /// position or None if out of bounds.
            pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
                if self.tree.len() > index {
                    self.tree.get_mut(index)
                } else {
                    self.tail[index - self.tree.len()].as_mut()
                }
            }

            /// Returns the number of elements in the vector.
            pub fn len(&self) -> usize {
                self.tree.len() + self.tail_len
            }

            /// Returns true if the vector has a length of 0.
            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }
        }

        impl<T: Clone + Debug> ops::Index<usize> for $vec<T> {
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

        impl<T: Clone + Debug> ops::IndexMut<usize> for $vec<T> {
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
    };
}

impl_vec!(RbVec);
impl_vec!(RrbVec);

impl<T: Clone + Debug> RbVec<T> {
    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated vector containing the elements
    /// in the range [at, len). After the call, the original vector
    /// will be left containing the elements [0, at).
    pub fn split_off(&mut self, mid: usize) -> Self {
        if mid == 0 {
            mem::replace(self, Self::new())
        } else if mid < self.len() {
            if self.tree.len() > mid {
                let chunks_count = (self.tree.len() - mid) / BRANCH_FACTOR;
                let mut chunks = Vec::with_capacity(chunks_count);

                while self.tree.len() - BRANCH_FACTOR > mid {
                    chunks.push(self.tree.pop());
                }

                let (mut left_tail, mut left_tail_len) = self.tree.pop();

                let from_i = mid - self.tree.len();
                let to_len = left_tail_len;

                let mut right = Self::new();
                for i in from_i..to_len {
                    right.push(left_tail[i].take().unwrap());
                    left_tail_len -= 1;
                }

                let mut right_tail = mem::replace(&mut self.tail, left_tail);
                let right_tail_len = mem::replace(&mut self.tail_len, left_tail_len);

                for (mut chunk, chunk_len) in chunks.into_iter().rev() {
                    for i in 0..chunk_len {
                        right.push(chunk[i].take().unwrap());
                    }
                }

                for i in 0..right_tail_len {
                    right.push(right_tail[i].take().unwrap());
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

                RbVec {
                    tree: RrbTree::new(),
                    tail: right_tail,
                    tail_len: right_tail_len,
                }
            }
        } else if mid == self.len() {
            Self::new()
        } else {
            panic!()
        }
    }

    /// Moves all the elements of `that` into
    /// `Self`, leaving `other` empty.
    pub fn append(&mut self, that: &mut RbVec<T>) {
        let that_is_empty = that.is_empty();

        if self.is_empty() {
            mem::swap(&mut self.tree, &mut that.tree);
            mem::swap(&mut self.tail, &mut that.tail);
            mem::swap(&mut self.tail_len, &mut that.tail_len);
        } else if !that_is_empty {
            let that_tree = mem::replace(&mut that.tree, RrbTree::new());
            let that_tail = mem::replace(&mut that.tail, new_branch!());

            let that_tail_len = that.tail_len;
            that.tail_len = 0;

            let that_vec = RbVec {
                tree: that_tree,
                tail: that_tail,
                tail_len: that_tail_len,
            };

            for value in that_vec.into_iter() {
                self.push(value);
            }
        }
    }
}

impl<T: Clone + Debug> RrbVec<T> {
    /// Splits the collection into two at the given index.
    ///
    /// Returns a vector containing the elements in the range [at, len).
    /// After the call, the original vector will be left
    /// containing the elements [0, at).
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

    /// Moves all the elements of `that` into `Self` by concatenating
    /// the underlying tree structures, leaving `other` empty.
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

impl<T: Clone + Debug> From<&Vec<T>> for RrbVec<T> {
    #[inline(always)]
    fn from(vec: &Vec<T>) -> RrbVec<T> {
        let mut tree = RrbTree::new();

        let mut chunks = vec.chunks_exact(BRANCH_FACTOR);
        for chunk in chunks.by_ref() {
            tree.push(clone_arr!(chunk), chunk.len());
        }

        let mut tail = new_branch!();
        let mut tail_len = 0;

        for item in chunks.remainder() {
            tail[tail_len] = Some(item.clone());
            tail_len += 1;
        }

        RrbVec {
            tree,
            tail,
            tail_len,
        }
    }
}
