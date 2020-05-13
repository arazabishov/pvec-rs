//! Persistent vectors with efficient clone, append and split
//! operations with idiomatic Rust interface.
//!
//! # Provided vector types
//! There are three public vector types available at pvec-rs:
//! * [RbVec](crate::core::RbVec): based on RbTree with naive
//! append and split operations. This type is not recommended
//! for use, as it is here solely for comparison in benchmarks.
//! * [RrbVec](crate::core::RrbVec): based on RrbTree that
//! enables efficient append and split operations.
//! * [PVec](crate::PVec): a persistent vector that starts out
//! as the standard [vec](std::vec::Vec), and transitions to
//! [RrbVec](crate::core::RrbVec) on the first clone. The cost
//! of its operations is identical to the representation
//! it is backed by.
//!
//! All vector types in the list expose exactly the same set of
//! operations with identical API. The difference is only in the
//! cost of operations.
//!
//! # Features
//! [RbVec](crate::core::RbVec) and [RrbVec](crate::core::RrbVec)
//! both use [Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html)
//! for garbage collection. The library provides an option to
//! compile all vectors using [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)
//! if needed, especially when passing instances between threads. To compile the
//! library with [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html), use
//! the `arc` feature flag.
//!
//! All types implement [Rayon's IntoParallelIterator trait](https://docs.rs/rayon/1.3.0/rayon/iter/trait.IntoParallelIterator.html),
//! that enables the conversion into a parallel iterator. As dependency on
//! Rayon is optional, you will need to explicitly request the parallel
//! iterator implementation by passing both the `arc` and `rayon_iter`
//! feature flags.
//!
//! By default, the tree-based vectors have nodes that are 32 elements wide. The
//! maximum number of child nodes is also referred to as the branching factor.
//! This value can be changed to 4 if necessary, by specifying the `small_branch`
//! feature flag. Though, the default value of 32 is recommended
//! for optimal performance.

#![warn(missing_docs)]

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
extern crate rayon;

#[macro_use]
#[cfg(feature = "serde_serializer")]
extern crate serde_json;

use std::fmt::Debug;
use std::ops;

pub mod core;
pub mod iter;

use crate::core::RrbVec;

#[cfg(not(feature = "small_branch"))]
const BRANCH_FACTOR: usize = 32;

#[cfg(feature = "small_branch")]
const BRANCH_FACTOR: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Representation<T> {
    Flat(Vec<T>),
    Tree(RrbVec<T>),
}

impl<T: Clone + Debug> Representation<T> {
    #[inline(always)]
    fn as_flat(&self) -> &Vec<T> {
        match self {
            Representation::Flat(ref vec) => vec,
            Representation::Tree(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn as_mut_flat(&mut self) -> &mut Vec<T> {
        match self {
            Representation::Flat(ref mut vec) => vec,
            Representation::Tree(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn as_mut_tree(&mut self) -> &mut RrbVec<T> {
        match self {
            Representation::Flat(..) => unreachable!(),
            Representation::Tree(ref mut tree) => tree,
        }
    }

    #[inline(always)]
    fn is_flat(&self) -> bool {
        match self {
            Representation::Flat(..) => true,
            Representation::Tree(..) => false,
        }
    }

    #[inline(always)]
    fn spill(vec: &Vec<T>) -> Representation<T> {
        Representation::Tree(RrbVec::from(vec))
    }
}

/// A persistent vector that is backed by the flat
/// representation - the standard vector, or the
/// tree-based vector when cloned.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PVec<T>(Representation<T>);

impl<T: Clone + Debug> PVec<T> {
    /// Constructs a new, empty vector backed by the
    /// standard vector internally.
    pub fn new() -> Self {
        PVec(Representation::Flat(Vec::with_capacity(BRANCH_FACTOR)))
    }

    /// Constructs a new, empty vector backed by the
    /// [RrbVec](crate::core::RrbVec).
    pub fn new_with_tree() -> Self {
        PVec(Representation::Tree(RrbVec::new()))
    }

    /// Adds an element to the back of a collection.
    pub fn push(&mut self, item: T) {
        match self.0 {
            Representation::Flat(ref mut vec) => vec.push(item),
            Representation::Tree(ref mut vec) => vec.push(item),
        }
    }

    /// Removes the last element from a vector and
    /// returns it, or None if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        match self.0 {
            Representation::Flat(ref mut vec) => vec.pop(),
            Representation::Tree(ref mut vec) => vec.pop(),
        }
    }

    /// Returns a reference to an element at the given
    /// position or None if out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        match self.0 {
            Representation::Flat(ref vec) => vec.get(index),
            Representation::Tree(ref vec) => vec.get(index),
        }
    }

    /// Returns a mutable reference to an element at the given
    /// position or None if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.0 {
            Representation::Flat(ref mut vec) => vec.get_mut(index),
            Representation::Tree(ref mut vec) => vec.get_mut(index),
        }
    }

    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        match self.0 {
            Representation::Flat(ref vec) => vec.len(),
            Representation::Tree(ref vec) => vec.len(),
        }
    }

    /// Returns true if the vector has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Moves all the elements of `that` into `Self` by concatenating
    /// the underlying tree structures, leaving `other` empty.
    /// Note, if either of vectors is tree-based, the resulting
    /// vector will end-up being tree-based as well.
    pub fn append(&mut self, that: &mut PVec<T>) {
        let this = &mut self.0;
        let that = &mut that.0;

        let this_is_flat = this.is_flat();
        let that_is_flat = that.is_flat();

        if this_is_flat && that_is_flat {
            this.as_mut_flat().append(that.as_mut_flat());
        } else if this_is_flat {
            let mut vec = RrbVec::from(this.as_flat());
            vec.append(that.as_mut_tree());

            *this = Representation::Tree(vec);
        } else if that_is_flat {
            let mut vec = RrbVec::from(that.as_flat());
            this.as_mut_tree().append(&mut vec);
        } else {
            this.as_mut_tree().append(that.as_mut_tree());
        }
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a vector containing the elements in the range [at, len).
    /// After the call, the original vector will be left
    /// containing the elements [0, at).
    pub fn split_off(&mut self, mid: usize) -> Self {
        let representation = match self.0 {
            Representation::Flat(ref mut vec) => Representation::Flat(vec.split_off(mid)),
            Representation::Tree(ref mut vec) => Representation::Tree(vec.split_off(mid)),
        };

        PVec(representation)
    }
}

impl<T: Clone + Debug> Default for PVec<T> {
    fn default() -> Self {
        PVec::new()
    }
}

impl<T: Clone + Debug> Clone for PVec<T> {
    fn clone(&self) -> Self {
        let representation = match self.0 {
            Representation::Flat(ref vec) => Representation::spill(vec),
            Representation::Tree(ref vec) => Representation::Tree(vec.clone()),
        };

        PVec(representation)
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
