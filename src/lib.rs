pub extern crate pvec_core as core;
extern crate pvec_utils as utils;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
extern crate rayon;

#[cfg(not(feature = "small_branch"))]
const THRESHOLD: usize = 1024;

#[cfg(feature = "small_branch")]
const THRESHOLD: usize = 256;

use std::fmt::Debug;
use std::ops;

pub mod iter;

use crate::core::{new_branch, RrbTree, BRANCH_FACTOR};
use crate::utils::sharedptr::{SharedPtr, Take};

use std::mem;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Representation<T> {
    Flat(SharedPtr<Vec<Option<T>>>),
    Tree(RrbTree<T>),
}

impl<T: Clone + Debug> Representation<T> {
    #[inline(always)]
    fn into_flat(self) -> Vec<Option<T>> {
        match self {
            Representation::Flat(ptr) => ptr.take(),
            Representation::Tree(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn into_tree(self) -> RrbTree<T> {
        match self {
            Representation::Flat(..) => unreachable!(),
            Representation::Tree(tree) => tree,
        }
    }

    #[inline(always)]
    fn as_flat(&self) -> &Vec<Option<T>> {
        match self {
            Representation::Flat(ref ptr) => ptr,
            Representation::Tree(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn as_mut_flat(&mut self) -> &mut Vec<Option<T>> {
        match self {
            Representation::Flat(ref mut ptr) => SharedPtr::make_mut(ptr),
            Representation::Tree(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn as_mut_tree(&mut self) -> &mut RrbTree<T> {
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
    fn spill(vec: &Vec<Option<T>>) -> Representation<T> {
        let mut tree = RrbTree::new();

        for chunk in vec.chunks(BRANCH_FACTOR) {
            let mut tail = new_branch!();

            tail.clone_from_slice(chunk);
            tree.push(tail, chunk.len());
        }

        Representation::Tree(tree)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PVec<T> {
    representation: Representation<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

impl<T: Clone + Debug> PVec<T> {
    pub fn new() -> Self {
        PVec {
            representation: Representation::Flat(SharedPtr::new(Vec::new())),
            tail: new_branch!(),
            tail_len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.tail[self.tail_len] = Some(item);
        self.tail_len += 1;

        self.push_tail();
    }

    #[inline(always)]
    fn push_tail(&mut self) {
        if self.tail_len == BRANCH_FACTOR {
            let tail = mem::replace(&mut self.tail, new_branch!());
            let tail_len = self.tail_len;

            match self.representation {
                Representation::Flat(ref mut ptr) => {                    
                    SharedPtr::make_mut(ptr).extend_from_slice(&tail);
                }
                Representation::Tree(ref mut tree) => tree.push(tail, tail_len),
            }

            self.tail_len = 0;
        }
    }

    // pub fn pop(&mut self) -> Option<T> {
    //     let (item, is_standard, len) = match self.0 {
    //         Flavor::Standard(ref mut vec) => {
    //             // let vec = SharedPtr::make_mut(ptr);
    //             let vec_len = vec.len() - 1;

    //             (vec.pop(), true, vec_len)
    //         }
    //         Flavor::Persistent(ref mut pvec) => {
    //             // let pvec = SharedPtr::make_mut(ptr);
    //             let pvec_len = pvec.len() - 1;

    //             (pvec.pop(), false, pvec_len)
    //         }
    //     };

    //     if !is_standard && len <= THRESHOLD {
    //         self.downgrade();
    //     }

    //     item
    // }

    // pub fn get(&self, index: usize) -> Option<&T> {
    //     match self.0 {
    //         Flavor::Standard(ref vec) => vec.get(index),
    //         Flavor::Persistent(ref vec) => vec.get(index),
    //     }
    // }

    // pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
    //     let (is_standard, vec_len, ref_count) = match self.0 {
    //         Flavor::Standard(ref vec) => (true, vec.len(), 0),
    //         Flavor::Persistent(ref vec) => (false, vec.len(), 0),
    //     };

    //     if is_standard && vec_len > THRESHOLD && ref_count > 1 {
    //         self.upgrade();
    //     }

    //     match self.0 {
    //         Flavor::Standard(ref mut vec) => vec.get_mut(index),
    //         Flavor::Persistent(ref mut vec) => vec.get_mut(index),
    //     }
    // }

    pub fn len(&self) -> usize {
        let representation_len = match self.representation {
            Representation::Flat(ref ptr) => ptr.len(),
            Representation::Tree(ref tree) => tree.len(),
        };

        representation_len + self.tail_len
    }

    // pub fn is_empty(&self) -> bool {
    //     self.len() == 0
    // }

    // pub fn append(&mut self, that: &mut PVec<T>) {
    //     // a(s), b(s) (upgrade a, push b into a)
    //     // a(p), b(s) (push b into a)
    //     // a(s), b(p) (upgrade a, a append b)
    //     // a(p), b(p) (a append b)

    //     if that.len() == 0 {
    //         return;
    //     }

    //     let (self_is_standard, self_len) = match self.0 {
    //         Flavor::Standard(ref vec) => (true, vec.len()),
    //         Flavor::Persistent(ref vec) => (false, vec.len()),
    //     };

    //     if self_len == 0 {
    //         mem::swap(&mut self.0, &mut that.0);
    //     } else if self_len + that.len() > THRESHOLD {
    //         if self_is_standard {
    //             self.upgrade();
    //         }

    //         let rrbvec = self.0.as_mut_persistent();
    //         match that.0 {
    //             Flavor::Standard(ref mut vec) => {
    //                 for i in vec.drain(..) {
    //                     rrbvec.push(i);
    //                 }
    //             }
    //             Flavor::Persistent(ref mut vec) => rrbvec.append(vec),
    //         }
    //     } else {
    //         let self_vec = self.0.as_mut_standard();
    //         let that_vec = that.0.as_mut_standard();

    //         self_vec.append(that_vec);
    //     }
    // }

    // // TODO: consider chunking Vec before pushing it onto RrbVec
    // pub fn split_off(&mut self, mid: usize) -> Self {
    //     let self_should_be_standard = mid <= THRESHOLD;
    //     let that_should_be_standard = (self.len() - mid) <= THRESHOLD;

    //     let flavor = if self.0.is_standard() {
    //         let old_that_vec = if self_should_be_standard {
    //             let self_vec = self.0.as_mut_standard();
    //             let that_vec = self_vec.split_off(mid);

    //             that_vec
    //         } else {
    //             let new_flavor = Flavor::Persistent(RrbVec::new());
    //             let old_flavor = mem::replace(&mut self.0, new_flavor);

    //             let mut old_vec = old_flavor.into_standard();
    //             let old_that_vec = old_vec.split_off(mid);

    //             let new_vec = self.0.as_mut_persistent();
    //             for item in old_vec.into_iter() {
    //                 new_vec.push(item);
    //             }

    //             old_that_vec
    //         };

    //         if that_should_be_standard {
    //             Flavor::Standard(old_that_vec)
    //         } else {
    //             let mut new_that_vec = RrbVec::new();
    //             for item in old_that_vec.into_iter() {
    //                 new_that_vec.push(item);
    //             }

    //             Flavor::Persistent(new_that_vec)
    //         }
    //     } else {
    //         let old_that_vec = if self_should_be_standard {
    //             let new_flavor = Flavor::Standard(Vec::with_capacity(mid));
    //             let old_flavor = mem::replace(&mut self.0, new_flavor);

    //             let mut old_vec = old_flavor.into_persistent();
    //             let old_that_vec = old_vec.split_off(mid);

    //             let new_vec = self.0.as_mut_standard();
    //             for item in old_vec.into_iter() {
    //                 new_vec.push(item);
    //             }

    //             old_that_vec
    //         } else {
    //             let self_vec = self.0.as_mut_persistent();
    //             let that_vec = self_vec.split_off(mid);

    //             that_vec
    //         };

    //         if that_should_be_standard {
    //             let mut new_that_vec = Vec::with_capacity(old_that_vec.len());
    //             for item in old_that_vec.into_iter() {
    //                 new_that_vec.push(item);
    //             }

    //             Flavor::Standard(new_that_vec)
    //         } else {
    //             Flavor::Persistent(old_that_vec)
    //         }
    //     };

    //     PVec(flavor)
    // }

    // #[inline(always)]
    // fn downgrade(&mut self) {
    //     let len = self.len();

    //     let new_flavor = Flavor::Standard(Vec::with_capacity(len));
    //     let old_flavor = mem::replace(&mut self.0, new_flavor);

    //     let old_vec = old_flavor.into_persistent();
    //     let new_vec = self.0.as_mut_standard();

    //     for item in old_vec.into_iter() {
    //         new_vec.push(item);
    //     }
    // }
}

impl<T: Clone + Debug> Default for PVec<T> {
    fn default() -> Self {
        PVec::new()
    }
}

impl<T: Clone + Debug> Clone for PVec<T> {
    fn clone(&self) -> Self {
        let representation = match self.representation {
            Representation::Flat(ref vec) => {
                if vec.len() < THRESHOLD {
                    Representation::Flat(vec.clone())                 
                } else {                    
                    Representation::spill(vec)
                }
            }
            Representation::Tree(ref tree) => {                
                Representation::Tree(tree.clone())
            },
        };        

        PVec {
            representation: representation,
            tail: self.tail.clone(),
            tail_len: self.tail_len,
        }
    }
}

// impl<T: Clone + Debug> ops::Index<usize> for PVec<T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &T {
//         self.get(index).unwrap_or_else(|| {
//             panic!(
//                 "index `{}` out of bounds in PVec of length `{}`",
//                 index,
//                 self.len()
//             )
//         })
//     }
// }

// impl<T: Clone + Debug> ops::IndexMut<usize> for PVec<T> {
//     fn index_mut(&mut self, index: usize) -> &mut T {
//         let len = self.len();
//         self.get_mut(index).unwrap_or_else(|| {
//             panic!(
//                 "index `{}` out of bounds in PVec of length `{}`",
//                 index, len
//             )
//         })
//     }
// }

#[cfg(test)]
#[macro_use]
mod test {
    // use super::Flavor;
    use super::PVec;
    // use super::RrbVec;
    use super::THRESHOLD;

    // #[test]
    // fn interleaving_append_split_off_operations() {
    //     let mut pvec = PVec::new();
    //     let mut value = 0;

    //     for size in 1..(32 * 8 + 32) {
    //         let mut another_pvec = PVec::new();
    //         for _ in 0..size {
    //             another_pvec.push(value);
    //             value += 1;
    //         }

    //         pvec.append(&mut another_pvec);

    //         let mid = pvec.len() / 2;
    //         let mut right = pvec.split_off(mid);

    //         pvec.append(&mut right);
    //         value = pvec.len();
    //     }

    //     for i in 0..value {
    //         assert_eq!(pvec.get(i).cloned(), Some(i));
    //     }
    // }

    // #[test]
    // fn pop_must_downgrade_to_standard_vec() {
    //     let mut pvec_one = PVec::new();
    //     for item in 0..(THRESHOLD * 2) - 1 {
    //         pvec_one.push(item);
    //     }

    //     // calling push after cloning a vector
    //     // over a threshold size must upgrade it
    //     let mut pvec_two = pvec_one.clone();
    //     pvec_two.push(0xdeadbeef);

    //     for _ in (THRESHOLD + 1..pvec_two.len()).rev() {
    //         pvec_two.pop();
    //         assert!(!pvec_two.0.is_standard());
    //     }

    //     for _ in (0..THRESHOLD + 1).rev() {
    //         pvec_two.pop();
    //         assert!(pvec_two.0.is_standard());
    //     }

    //     assert!(pvec_two.is_empty());
    // }

    // #[test]
    // fn split_off_must_downgrade_to_standard() {
    //     let mut vec_one = PVec::new();
    //     for item in 0..(THRESHOLD * 2) - 1 {
    //         vec_one.push(item);
    //     }

    //     // calling push after cloning a vector
    //     // over a threshold size must upgrade it
    //     let mut vec_two = vec_one.clone();
    //     vec_two.push(0xdeadbeef);

    //     assert!(!vec_two.0.is_standard());
    //     let vec_two_that = vec_two.split_off(THRESHOLD);

    //     assert!(vec_two_that.0.is_standard());
    //     assert!(vec_two.0.is_standard());
    // }

    // #[test]
    // fn split_off_must_downgrade_to_standard_only_one_half() {
    //     let mut self_vec = PVec::new();

    //     for item in 0..(THRESHOLD * 4) - 1 {
    //         self_vec.push(item);
    //     }

    //     // calling push after cloning a vector
    //     // over a threshold size must upgrade it
    //     let mut self_vec_one = self_vec.clone();
    //     self_vec_one.push(0xdeadbeef);

    //     let mut self_vec_two = self_vec_one.clone();

    //     assert!(!self_vec_one.0.is_standard());
    //     assert!(!self_vec_two.0.is_standard());

    //     let that_vec_one = self_vec_one.split_off(THRESHOLD);
    //     let that_vec_two = self_vec_two.split_off(THRESHOLD * 3);

    //     assert!(self_vec_one.0.is_standard());
    //     assert!(that_vec_two.0.is_standard());

    //     assert!(!self_vec_two.0.is_standard());
    //     assert!(!that_vec_one.0.is_standard());
    // }

    // #[test]
    // fn sizes() {
    //     let pvec_size = std::mem::size_of::<PVec<u8>>();
    //     let rrbvec_size = std::mem::size_of::<RrbVec<u8>>();
    //     let flavor_size = std::mem::size_of::<Flavor<u8>>();
    //     let vec_size = std::mem::size_of::<Vec<u8>>();

    //     println!("Size of PVec<u8>={}", pvec_size);
    //     println!("Size of RrbVec<u8>={}", rrbvec_size);
    //     println!("Size of Flavor<u8>={}", flavor_size);
    //     println!("Size of Vec<u8>={}", vec_size);
    // }

    #[test]
    fn push_clone() {
        let mut vec_1 = PVec::new();
        let mut vec_2 = vec_1.clone();
        let mut vec_3 = vec_2.clone();

        // if you have replaced contents of the original vector 
        // by doing some unsafe manipulations, you wouldn't 
        // have had this problem

        for i in 0..2048 {
            vec_3 = vec_1; // <-- vec_1 reference lives here
            vec_1 = vec_2; // <-- vec_2 reference lives here

            vec_1.push(i);
            vec_2 = vec_1.clone();
        }

        println!("Size of vec_1={}", vec_1.len());
        println!("Size of vec_2={}", vec_2.len());
        println!("Size of vec_3={}", vec_3.len());        
    }
}
