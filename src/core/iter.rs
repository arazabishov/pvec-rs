//! A module providing implementation of the standard
//! [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html),
//! as well as [Rayon's ParallelIterator](https://docs.rs/rayon/1.3.0/rayon/iter/trait.ParallelIterator.html)
//! if the `rayon_iter` feature flag is specified.

use super::RbVec;
use super::RrbVec;
use crate::core::rrbtree::iter::RrbTreeIter;
use crate::core::rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;
use std::iter::FromIterator;

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
use rayon::prelude::{
    FromParallelIterator, IndexedParallelIterator, IntoParallelIterator, ParallelIterator,
};

macro_rules! impl_iter {
    ($vec:ident, $iter:ident) => {
        /// This struct keeps state necessary to implement Iterator
        /// for the tree-based vector. It takes the ownership of the vector
        /// contents. The iterator implementation consumes the contents of
        /// the RrbTree by chunks, this way reducing the number of the
        /// tree traversals.
        #[derive(Debug, Clone)]
        pub struct $iter<T> {
            tree_iter: RrbTreeIter<T>,
            head_chunk: Option<([Option<T>; BRANCH_FACTOR], usize)>,
            head_chunk_idx: usize,
            head_idx: usize,
            tail_chunk: Option<([Option<T>; BRANCH_FACTOR], usize)>,
            tail_chunk_idx: usize,
            tail_idx: usize,
            len: usize,
        }

        impl<T: Clone + Debug> Iterator for $iter<T> {
            type Item = T;

            fn next(&mut self) -> Option<Self::Item> {
                if self.head_idx <= self.tail_idx {
                    if self.head_chunk.is_none() {
                        self.head_chunk = self.tree_iter.next();
                    }

                    let mut chunk = if self.head_chunk.is_some() {
                        self.head_chunk.as_mut()
                    } else {
                        self.tail_chunk.as_mut()
                    };

                    let head_chunk_idx = self.head_chunk_idx;
                    let item = chunk.as_mut().and_then(|it| it.0[head_chunk_idx].take());

                    self.head_idx += 1;
                    self.head_chunk_idx += 1;

                    if let Some(it) = chunk.as_ref() {
                        if self.head_chunk_idx == it.1 {
                            self.head_chunk = self.tree_iter.next();
                            self.head_chunk_idx = 0;
                        }
                    }

                    item
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.len, Some(self.len))
            }
        }

        impl<T: Clone + Debug> DoubleEndedIterator for $iter<T> {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.head_idx <= self.tail_idx {
                    if self.tail_chunk.is_none() {
                        self.tail_chunk = self.tree_iter.next_back();
                        self.tail_chunk_idx = self
                            .tail_chunk
                            .as_ref()
                            .map(|chunk| chunk.1 - 1)
                            .unwrap_or(0);
                    }

                    let chunk = if self.tail_chunk.is_some() {
                        self.tail_chunk.as_mut()
                    } else {
                        self.head_chunk.as_mut()
                    };

                    let tail_chunk_idx = self.tail_chunk_idx;
                    let item = chunk.and_then(|it| it.0[tail_chunk_idx].take());

                    if self.tail_chunk_idx == 0 {
                        self.tail_chunk = self.tree_iter.next_back();
                        self.tail_chunk_idx = self
                            .tail_chunk
                            .as_ref()
                            .map(|chunk| chunk.1 - 1)
                            .unwrap_or(0);
                    } else {
                        self.tail_idx -= 1;
                        self.tail_chunk_idx -= 1;
                    }

                    item
                } else {
                    None
                }
            }
        }

        impl<T: Clone + Debug> ExactSizeIterator for $iter<T> {
            fn len(&self) -> usize {
                self.len
            }
        }

        impl<T: Clone + Debug> IntoIterator for $vec<T> {
            type Item = T;
            type IntoIter = $iter<T>;

            fn into_iter(self) -> Self::IntoIter {
                let len = self.len();

                let mut tail_chunk_idx = self.tail_len;
                let mut tail_idx = len;

                let tail_chunk = if self.tail_len > 0 {
                    Some((self.tail, self.tail_len))
                } else {
                    None
                };

                if tail_chunk_idx > 0 {
                    tail_chunk_idx -= 1;
                }

                if tail_idx > 0 {
                    tail_idx -= 1;
                }

                $iter {
                    tree_iter: self.tree.into_iter(),
                    head_chunk: None,
                    head_chunk_idx: 0,
                    head_idx: 0,
                    tail_chunk,
                    tail_chunk_idx,
                    tail_idx,
                    len,
                }
            }
        }

        impl<T: Clone + Debug> FromIterator<T> for $vec<T> {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
                let mut vec = $vec::new();
                for i in iter {
                    vec.push(i);
                }
                vec
            }
        }
    };
}

impl_iter!(RbVec, RbVecIter);
impl_iter!(RrbVec, RrbVecIter);

macro_rules! impl_into_par_iter {
    ($vec:ident, $iter:ident, $pariter:ident, $producer:ident) => {
        /// This struct is used to implement the
        /// [parallel iterator](https://docs.rs/rayon/1.3.0/rayon/iter/trait.ParallelIterator.html)
        #[derive(Debug, Clone)]
        #[cfg(all(feature = "arc", feature = "rayon_iter"))]
        pub struct $pariter<T: Send + Sync + Debug + Clone> {
            vec: $vec<T>,
        }

        #[cfg(all(feature = "arc", feature = "rayon_iter"))]
        impl<T: Send + Sync + Debug + Clone> ParallelIterator for $pariter<T> {
            type Item = T;

            fn drive_unindexed<C>(self, consumer: C) -> C::Result
            where
                C: UnindexedConsumer<Self::Item>,
            {
                bridge(self, consumer)
            }

            fn opt_len(&self) -> Option<usize> {
                Some(self.vec.len())
            }
        }

        #[cfg(all(feature = "arc", feature = "rayon_iter"))]
        impl<T: Send + Sync + Debug + Clone> IndexedParallelIterator for $pariter<T> {
            fn drive<C>(self, consumer: C) -> C::Result
            where
                C: Consumer<Self::Item>,
            {
                bridge(self, consumer)
            }

            fn len(&self) -> usize {
                self.vec.len()
            }

            fn with_producer<CB>(self, callback: CB) -> CB::Output
            where
                CB: ProducerCallback<Self::Item>,
            {
                callback.callback($producer { vec: self.vec })
            }
        }

        #[cfg(all(feature = "arc", feature = "rayon_iter"))]
        struct $producer<T: Send + Sync + Debug + Clone> {
            vec: $vec<T>,
        }

        #[cfg(all(feature = "arc", feature = "rayon_iter"))]
        impl<T: Send + Sync + Debug + Clone> Producer for $producer<T> {
            type Item = T;
            type IntoIter = $iter<T>;

            fn into_iter(mut self) -> Self::IntoIter {
                std::mem::replace(&mut self.vec, $vec::new()).into_iter()
            }

            fn split_at(mut self, index: usize) -> (Self, Self) {
                let mut vec = std::mem::replace(&mut self.vec, $vec::new());

                let right = vec.split_off(index);
                let left = vec;

                ($producer { vec: left }, $producer { vec: right })
            }
        }

        #[cfg(all(feature = "arc", feature = "rayon_iter"))]
        impl<T: Send + Sync + Debug + Clone> IntoParallelIterator for $vec<T> {
            type Item = T;
            type Iter = $pariter<T>;

            fn into_par_iter(self) -> Self::Iter {
                $pariter { vec: self }
            }
        }

        #[cfg(all(feature = "arc", feature = "rayon_iter"))]
        impl<T: Clone + Debug + Send + Sync> FromParallelIterator<T> for $vec<T>
        where
            T: Send,
        {
            fn from_par_iter<I>(par_iter: I) -> Self
            where
                I: IntoParallelIterator<Item = T>,
            {
                par_iter
                    .into_par_iter()
                    .fold($vec::new, |mut vec, elem| {
                        vec.push(elem);
                        vec
                    })
                    .reduce($vec::new, |mut list1, mut list2| {
                        list1.append(&mut list2);
                        list1
                    })
            }
        }
    };
}

impl_into_par_iter!(RbVec, RbVecIter, RbVecParIter, RbVecProducer);
impl_into_par_iter!(RrbVec, RrbVecIter, RrbVecParIter, RrbVecProducer);
