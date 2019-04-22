#![feature(nll)]

#[macro_use]
extern crate serde_json;
extern crate serde;

use std::fmt::Debug;
use std::ops;

#[macro_use]
mod rrbtree;
mod rrbvec;

mod iter;
mod sharedptr;

use rrbvec::RrbVec;
use sharedptr::SharedPtr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Flavor<T> {
    Standard(SharedPtr<Vec<T>>),
    Persistent(RrbVec<T>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PVec<T>(Flavor<T>);

impl<T: Clone + Debug> PVec<T> {
    pub fn new() -> Self {
        PVec(Flavor::Standard(SharedPtr::new(Vec::new())))
    }

    #[cold]
    pub fn push(&mut self, item: T) {
        if self.is_standard() && self.len() > 2048 {
            self.upgrade();
        }

        match self.0 {
            Flavor::Standard(ref mut vec_arc) => SharedPtr::make_mut(vec_arc).push(item),
            Flavor::Persistent(ref mut pvec) => pvec.push(item),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.0 {
            Flavor::Standard(ref mut vec_arc) => SharedPtr::make_mut(vec_arc).pop(),
            Flavor::Persistent(ref mut pvec) => pvec.pop(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        return match self.0 {
            Flavor::Standard(ref vec_arc) => vec_arc.get(index),
            Flavor::Persistent(ref pvec) => pvec.get(index),
        };
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        return match self.0 {
            Flavor::Standard(ref mut vec_arc) => {
                dbg!(SharedPtr::strong_count(vec_arc));
                SharedPtr::make_mut(vec_arc).get_mut(index)
            }
            Flavor::Persistent(ref mut pvec) => pvec.get_mut(index),
        };
    }

    pub fn len(&self) -> usize {
        return match self.0 {
            Flavor::Standard(ref vec) => vec.len(),
            Flavor::Persistent(ref pvec) => pvec.len(),
        };
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
            Flavor::Persistent(ref mut rrbvec) => rrbvec,
        }
    }

    pub fn append(&mut self, that: &mut PVec<T>) {
        // ToDo: reconsider cases when either self or that are very big

        // a(s), b(s) (upgrade a, push b into a)
        // a(p), b(s) (push b into a)
        // a(s), b(p) (upgrade a, a append b)
        // a(p), b(p) (a append b)

        if self.len() + that.len() > 2048 {
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
                Flavor::Persistent(ref mut rrbvec_that) => rrbvec.append(rrbvec_that),
            }
        } else {
            let self_vec = self.as_mut_standard();
            let that_vec = that.as_mut_standard();

            self_vec.append(that_vec);
        }
    }

    fn upgrade(&mut self) {
        let pvec = match self.0 {
            Flavor::Standard(ref vec) => RrbVec::from(vec),
            Flavor::Persistent(..) => unreachable!(),
        };

        self.0 = Flavor::Persistent(pvec);
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
