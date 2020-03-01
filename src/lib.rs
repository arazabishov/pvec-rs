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

use crate::core::{RrbVec, BRANCH_FACTOR};

use std::mem;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Representation<T> {
    Flat(Vec<T>),
    Tree(RrbVec<T>),
}

impl<T: Clone + Debug> Representation<T> {
    #[inline(always)]
    fn into_flat(self) -> Vec<T> {
        match self {
            Representation::Flat(vec) => vec,
            Representation::Tree(..) => unreachable!(),
        }
    }

    #[inline(always)]
    fn into_tree(self) -> RrbVec<T> {
        match self {
            Representation::Flat(..) => unreachable!(),
            Representation::Tree(tree) => tree,
        }
    }

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PVec<T>(Representation<T>);

impl<T: Clone + Debug> PVec<T> {
    pub fn new() -> Self {
        PVec(Representation::Flat(Vec::with_capacity(BRANCH_FACTOR)))
    }

    pub fn push(&mut self, item: T) {
        match self.0 {
            Representation::Flat(ref mut vec) => vec.push(item),
            Representation::Tree(ref mut vec) => vec.push(item),
        }
    }

    // TODO: consider downgrades
    // TODO: consider using macro to cut down boilerplate
    pub fn pop(&mut self) -> Option<T> {
        match self.0 {
            Representation::Flat(ref mut vec) => vec.pop(),
            Representation::Tree(ref mut vec) => vec.pop(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match self.0 {
            Representation::Flat(ref vec) => vec.get(index),
            Representation::Tree(ref vec) => vec.get(index),
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self.0 {
            Representation::Flat(ref mut vec) => vec.get_mut(index),
            Representation::Tree(ref mut vec) => vec.get_mut(index),
        }
    }

    pub fn len(&self) -> usize {
        match self.0 {
            Representation::Flat(ref vec) => vec.len(),
            Representation::Tree(ref vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_spilled(&self) -> bool {
        self.0.is_flat()
    }

    pub fn spill(&mut self) {
        self.0 = match self.0 {
            Representation::Flat(ref vec) => Representation::spill(vec),
            Representation::Tree(..) => unreachable!(),
        }
    }

    // what about the following idea:
    // - if both vectors that are going to be merged are
    //      standard vectors, they won't be upgraded
    // - if one of the vectors is not standard, then you upgrade

    // What are the benefits? If you're working with a standard vector, you won't get 
    // upgraded until you explicitly clone. Essentially, you're in control when and 
    // why you would want to do this. 

    // What about benchmarks? 
    //  - pvec (std)
    //  - pvec (rrbvec)
    // Does it mean that you will force the transition in some cases

    // Options:
    //  1) Spill both vectors when appending
    //  2) Spill either of vectors if non flat

    // The selling point is that you don't have to upgrade
    // if you don't need to, or basically the idea of zero cost 
    // abstraction. See what Niko Mat wrote in his blog post. 

    // Benefits:
    // 1) you get fast push, pop, get, get_mut() operations when used without clones
    // 2) you get very cheap clones(), appends(), split_offs() if needed
    
    // That's how you can up-sell the idea of zero cost abstractions
    // Essentially, clone is the only place where you can upgrade
    
    pub fn append(&mut self, that: &mut PVec<T>) {
        if that.len() == 0 {
            return;
        }

        let (self_is_flat, self_len) = match self.0 {
            Representation::Flat(ref vec) => (true, vec.len()),
            Representation::Tree(ref vec) => (false, vec.len()),
        };

        if self_len == 0 {
            mem::swap(&mut self.0, &mut that.0);
            return;
        }

        if self_is_flat {            
            self.0 = Representation::spill(self.0.as_flat());
        }

        let self_vec = self.0.as_mut_tree();
        match that.0 {
            Representation::Flat(ref mut that_vec) => {
                for i in that_vec.drain(..) {
                    self_vec.push(i);
                }
            }
            Representation::Tree(ref mut that_vec) => self_vec.append(that_vec),
        }
    }

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

#[cfg(test)]
#[macro_use]
mod test {
    use super::{PVec, RrbVec, THRESHOLD};

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

        for i in 0..132 {
            vec_3 = vec_1; // <-- vec_1 reference lives here
            vec_1 = vec_2; // <-- vec_2 reference lives here

            vec_1.push(i);
            vec_2 = vec_1.clone();
        }

        println!("Size of vec_1={}", vec_1.len());
        println!("Size of vec_2={}", vec_2.len());
        println!("Size of vec_3={}", vec_3.len());
    }

    #[test]
    fn spill() {
        let mut vec = Vec::new();

        for i in 0..132 {
            vec.push(Some(i));
        }

        let rrbvec = RrbVec::from(&vec);

        println!("Size of vec={}", vec.len());
        println!("Size of rrbvec={}", rrbvec.len());
    }

    #[test]
    fn internal_state() {
        let mut input = create_input(16000);
        let mut vec = PVec::new();

        for mut input in input.iter_mut() {
            println!("input is flat={}", input.0.is_flat());
            vec.append(&mut input);
            println!("vec is flat={}", vec.0.is_flat());
        }
    }

    fn create_input(n: usize) -> Vec<PVec<usize>> {
        let mut input = Vec::new();
        let mut input_len = 0;
        let mut i = 1;

        while i < n && (input_len + i) <= n {
            let mut vec = PVec::new();

            for j in 0..i {
                vec.push(j);
            }

            input_len += vec.len();
            input.push(vec.clone());

            i *= 2;
        }

        let mut vec = PVec::new();
        let mut j = 0;

        while input_len < n {
            vec.push(j);

            input_len += 1;
            j += 1;
        }

        input.push(vec.clone());
        input
    }

    // fn create_input_2(n: usize) -> PVec<usize> {
    //     let mut i = 1;
    //     let mut vec = PVec::new();

    //     while i < n && (vec.len() + i) <= n {
    //         let mut another_vec = PVec::new();

    //         for j in 0..i {
    //             another_vec.push(j);
    //         }

    //         vec.append(another_vec);
    //         i *= 2;
    //     }

    //     while vec.len() < n {
    //         vec.push(i);
    //     }

    //     vec
    // }
}
