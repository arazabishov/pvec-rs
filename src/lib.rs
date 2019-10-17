pub extern crate pvec_core as core;
extern crate pvec_utils as utils;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
extern crate rayon;

#[cfg(not(feature = "small_branch"))]
const BRANCH_FACTOR: usize = 32;

#[cfg(not(feature = "small_branch"))]
const THRESHOLD: usize = 4096;

#[cfg(feature = "small_branch")]
const BRANCH_FACTOR: usize = 4;

#[cfg(feature = "small_branch")]
const THRESHOLD: usize = 1024;

use std::fmt::Debug;
use std::ops;

pub mod iter;

use crate::core::RrbVec;
use crate::utils::sharedptr::SharedPtr;
use crate::utils::sharedptr::Take;

use std::mem;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Flavor<T> {
    Standard(SharedPtr<Vec<T>>),
    Persistent(SharedPtr<RrbVec<T>>),
}

impl<T: Clone + Debug> Flavor<T> {
    #[inline(always)]
    fn into_standard(self) -> Vec<T> {
        match self {
            Flavor::Standard(ptr) => ptr.take(),
            Flavor::Persistent(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn into_persistent(self) -> RrbVec<T> {
        match self {
            Flavor::Standard(..) => unreachable!(),
            Flavor::Persistent(ptr) => ptr.take(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PVec<T>(Flavor<T>);

impl<T: Clone + Debug> PVec<T> {
    pub fn new() -> Self {
        let vec = Vec::with_capacity(BRANCH_FACTOR);
        PVec(Flavor::Standard(SharedPtr::new(vec)))
    }

    #[cold]
    pub fn push(&mut self, item: T) {
        let (is_standard, len) = match self.0 {
            Flavor::Standard(ref mut ptr) => {
                let vec = SharedPtr::make_mut(ptr);
                let vec_len = vec.len() + 1;

                vec.push(item);
                (true, vec_len)
            }
            Flavor::Persistent(ref mut ptr) => {
                let pvec = SharedPtr::make_mut(ptr);
                let pvec_len = pvec.len() + 1;

                pvec.push(item);
                (false, pvec_len)
            }
        };

        if is_standard && len > THRESHOLD {
            self.upgrade();
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let (item, is_standard, len) = match self.0 {
            Flavor::Standard(ref mut ptr) => {
                let vec = SharedPtr::make_mut(ptr);
                let vec_len = vec.len() - 1;

                (vec.pop(), true, vec_len)
            }
            Flavor::Persistent(ref mut ptr) => {
                let pvec = SharedPtr::make_mut(ptr);
                let pvec_len = pvec.len() - 1;

                (pvec.pop(), false, pvec_len)
            }
        };

        if !is_standard && len <= THRESHOLD {
            self.downgrade();
        }

        item
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match self.0 {
            Flavor::Standard(ref ptr) => ptr.get(index),
            Flavor::Persistent(ref ptr) => ptr.get(index),
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.0 {
            Flavor::Standard(ref mut ptr) => SharedPtr::make_mut(ptr).get_mut(index),
            Flavor::Persistent(ref mut ptr) => SharedPtr::make_mut(ptr).get_mut(index),
        }
    }

    pub fn len(&self) -> usize {
        match self.0 {
            Flavor::Standard(ref vec) => vec.len(),
            Flavor::Persistent(ref pvec) => pvec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    fn is_standard(&self) -> bool {
        match self.0 {
            Flavor::Standard(..) => true,
            Flavor::Persistent(..) => false,
        }
    }

    #[inline(always)]
    fn as_mut_standard(&mut self) -> &mut Vec<T> {
        match self.0 {
            Flavor::Standard(ref mut vec_arc) => SharedPtr::make_mut(vec_arc),
            Flavor::Persistent(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn as_mut_persistent(&mut self) -> &mut RrbVec<T> {
        match self.0 {
            Flavor::Standard(..) => unreachable!(),
            Flavor::Persistent(ref mut ptr) => SharedPtr::make_mut(ptr),
        }
    }

    pub fn append(&mut self, that: &mut PVec<T>) {
        // a(s), b(s) (upgrade a, push b into a)
        // a(p), b(s) (push b into a)
        // a(s), b(p) (upgrade a, a append b)
        // a(p), b(p) (a append b)

        if self.len() + that.len() > THRESHOLD {
            if self.is_standard() {
                self.upgrade();
            }

            let rrbvec = self.as_mut_persistent();

            match that.0 {
                Flavor::Standard(ref mut vec_arc) => {
                    // ToDo: drain might be causing performance issues
                    for i in SharedPtr::make_mut(vec_arc).drain(..) {
                        rrbvec.push(i);
                    }
                }
                Flavor::Persistent(ref mut ptr) => rrbvec.append(SharedPtr::make_mut(ptr)),
            }
        } else {
            let self_vec = self.as_mut_standard();
            let that_vec = that.as_mut_standard();

            self_vec.append(that_vec);
        }
    }

    pub fn split_off(&mut self, mid: usize) -> Self {
        let flavor = if self.is_standard() {
            let vec = self.as_mut_standard();
            let right = vec.split_off(mid);

            Flavor::Standard(SharedPtr::new(right))
        } else {
            let self_should_be_standard = mid <= THRESHOLD;
            let that_should_be_standard = (self.len() - mid) <= THRESHOLD;

            let old_that_vec = if self_should_be_standard {
                let new_flavor = Flavor::Standard(SharedPtr::new(Vec::with_capacity(mid)));
                let old_flavor = mem::replace(&mut self.0, new_flavor);

                let mut old_vec = old_flavor.into_persistent();
                let old_that_vec = old_vec.split_off(mid);

                let new_vec = self.as_mut_standard();
                for item in old_vec.into_iter() {
                    new_vec.push(item);
                }

                old_that_vec
            } else {
                let self_vec = self.as_mut_persistent();
                let that_vec = self_vec.split_off(mid);

                that_vec
            };

            if that_should_be_standard {
                let mut new_that_vec = Vec::with_capacity(old_that_vec.len());
                for item in old_that_vec.into_iter() {
                    new_that_vec.push(item);
                }

                Flavor::Standard(SharedPtr::new(new_that_vec))
            } else {
                Flavor::Persistent(SharedPtr::new(old_that_vec))
            }
        };

        PVec(flavor)
    }

    #[inline(always)]
    fn upgrade(&mut self) {
        let new_flavor = Flavor::Persistent(SharedPtr::new(RrbVec::new()));
        let old_flavor = mem::replace(&mut self.0, new_flavor);

        let old_vec = old_flavor.into_standard();
        let new_vec = self.as_mut_persistent();

        for item in old_vec.into_iter() {
            new_vec.push(item);
        }
    }

    #[inline(always)]
    fn downgrade(&mut self) {
        let len = self.len();

        let new_flavor = Flavor::Standard(SharedPtr::new(Vec::with_capacity(len)));
        let old_flavor = mem::replace(&mut self.0, new_flavor);

        let old_vec = old_flavor.into_persistent();
        let new_vec = self.as_mut_standard();

        for item in old_vec.into_iter() {
            new_vec.push(item);
        }
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
    use super::core::RrbVec;
    use super::PVec;
    use super::SharedPtr;
    use super::THRESHOLD;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct PVecTwo<T> {
        inline: Option<SharedPtr<Vec<T>>>,
        spilled: Option<RrbVec<T>>,
    }

    #[test]
    fn interleaving_append_split_off_operations() {
        let mut pvec = PVec::new();
        let mut value = 0;

        for size in 1..(32 * 8 + 32) {
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

    #[test]
    fn pop_must_downgrade_to_standard_vec() {
        let mut pvec = PVec::new();

        for item in 0..THRESHOLD * 2 {
            pvec.push(item);
        }

        for _ in (THRESHOLD + 1..pvec.len()).rev() {
            pvec.pop();
            assert!(!pvec.is_standard());
        }

        for _ in (0..THRESHOLD + 1).rev() {
            pvec.pop();
            assert!(pvec.is_standard());
        }

        assert!(pvec.is_empty());
    }

    #[test]
    fn split_off_must_downgrade_to_standard() {
        let mut self_vec = PVec::new();

        for item in 0..THRESHOLD * 2 {
            self_vec.push(item);
        }

        assert!(!self_vec.is_standard());
        let that_vec = self_vec.split_off(THRESHOLD);

        assert!(that_vec.is_standard());
        assert!(self_vec.is_standard());
    }

    #[test]
    fn split_off_must_downgrade_to_standard_only_one_half() {
        let mut self_vec_one = PVec::new();

        for item in 0..THRESHOLD * 4 {
            self_vec_one.push(item);
        }

        let mut self_vec_two = self_vec_one.clone();

        assert!(!self_vec_one.is_standard());
        assert!(!self_vec_two.is_standard());

        let that_vec_one = self_vec_one.split_off(THRESHOLD);
        let that_vec_two = self_vec_two.split_off(THRESHOLD * 3);

        assert!(self_vec_one.is_standard());
        assert!(that_vec_two.is_standard());

        assert!(!self_vec_two.is_standard());
        assert!(!that_vec_one.is_standard());
    }
}
