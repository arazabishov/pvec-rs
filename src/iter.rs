//! A module providing implementation of the standard
//! [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html),
//! as well as [Rayon's ParallelIterator](https://docs.rs/rayon/1.3.0/rayon/iter/trait.ParallelIterator.html)
//! if the `rayon_iter` feature flag is specified.

use super::PVec;
use super::Representation;

use std::fmt::Debug;

#[cfg(all(test, not(feature = "small_branch")))]
pub const BRANCH_FACTOR: usize = 32;

#[cfg(all(test, feature = "small_branch"))]
pub const BRANCH_FACTOR: usize = 4;

use crate::core::iter::RrbVecIter;
use crate::core::RrbVec;
use std::iter::FromIterator;
use std::vec::IntoIter as VecIter;

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
use rayon::prelude::{
    FromParallelIterator, IndexedParallelIterator, IntoParallelIterator, ParallelIterator,
};

/// This struct owns another, actual iterator
/// either of the standard vector or RrbVec and is
/// used to implement [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
/// trait.
#[derive(Debug, Clone)]
pub struct PVecIter<T> {
    iter_vec: Option<VecIter<T>>,
    iter_rrbvec: Option<RrbVecIter<T>>,
}

impl<T: Clone + Debug> PVecIter<T> {
    fn from_vec(vec: Vec<T>) -> Self {
        PVecIter {
            iter_vec: Some(vec.into_iter()),
            iter_rrbvec: None,
        }
    }

    fn from_rrbvec(rrbvec: RrbVec<T>) -> Self {
        PVecIter {
            iter_vec: None,
            iter_rrbvec: Some(rrbvec.into_iter()),
        }
    }
}

impl<T: Clone + Debug> Iterator for PVecIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter_vec) = self.iter_vec.as_mut() {
            iter_vec.next()
        } else if let Some(iter_rrbvec) = self.iter_rrbvec.as_mut() {
            iter_rrbvec.next()
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(iter_vec) = self.iter_vec.as_ref() {
            iter_vec.size_hint()
        } else if let Some(iter_rrbvec) = self.iter_rrbvec.as_ref() {
            iter_rrbvec.size_hint()
        } else {
            (0, None)
        }
    }
}

impl<T: Clone + Debug> ExactSizeIterator for PVecIter<T> {
    fn len(&self) -> usize {
        if let Some(iter_vec) = self.iter_vec.as_ref() {
            iter_vec.len()
        } else if let Some(iter_rrbvec) = self.iter_rrbvec.as_ref() {
            iter_rrbvec.len()
        } else {
            0
        }
    }
}

impl<T: Clone + Debug> DoubleEndedIterator for PVecIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(iter_vec) = self.iter_vec.as_mut() {
            iter_vec.next_back()
        } else if let Some(iter_rrbvec) = self.iter_rrbvec.as_mut() {
            iter_rrbvec.next_back()
        } else {
            None
        }
    }
}

impl<T: Clone + Debug> IntoIterator for PVec<T> {
    type Item = T;
    type IntoIter = PVecIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Representation::Flat(vec) => PVecIter::from_vec(vec),
            Representation::Tree(vec) => PVecIter::from_rrbvec(vec),
        }
    }
}

/// This struct is used to implement the
/// [parallel iterator](https://docs.rs/rayon/1.3.0/rayon/iter/trait.ParallelIterator.html)
#[derive(Debug, Clone)]
#[cfg(all(feature = "arc", feature = "rayon_iter"))]
pub struct PVecParIter<T: Send + Sync + Debug + Clone> {
    vec: PVec<T>,
}

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
impl<T: Send + Sync + Debug + Clone> IntoParallelIterator for PVec<T> {
    type Item = T;
    type Iter = PVecParIter<T>;

    fn into_par_iter(self) -> Self::Iter {
        PVecParIter { vec: self }
    }
}

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
impl<T: Send + Sync + Debug + Clone> ParallelIterator for PVecParIter<T> {
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
impl<T: Send + Sync + Debug + Clone> IndexedParallelIterator for PVecParIter<T> {
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
        callback.callback(VecProducer { vec: self.vec })
    }
}

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
struct VecProducer<T: Send + Sync + Debug + Clone> {
    vec: PVec<T>,
}

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
impl<T: Send + Sync + Debug + Clone> Producer for VecProducer<T> {
    type Item = T;
    type IntoIter = PVecIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        std::mem::replace(&mut self.vec, PVec::new()).into_iter()
    }

    fn split_at(mut self, index: usize) -> (Self, Self) {
        let mut vec = std::mem::replace(&mut self.vec, PVec::new());

        let right = vec.split_off(index);
        let left = vec;

        (VecProducer { vec: left }, VecProducer { vec: right })
    }
}

#[cfg(all(feature = "arc", feature = "rayon_iter"))]
impl<T: Clone + Debug + Send + Sync> FromParallelIterator<T> for PVec<T>
where
    T: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        par_iter
            .into_par_iter()
            .fold(PVec::new, |mut vec, elem| {
                vec.push(elem);
                vec
            })
            .reduce(PVec::new, |mut list1, mut list2| {
                list1.append(&mut list2);
                list1
            })
    }
}

impl<T: Clone + Debug> FromIterator<T> for PVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = PVec::new();
        for i in iter {
            vec.push(i);
        }
        vec
    }
}
