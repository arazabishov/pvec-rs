pub extern crate pvec_core as core;
extern crate pvec_utils as utils;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
extern crate rayon;

use std::fmt::Debug;
use std::ops;

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

#[cfg(test)]
#[macro_use]
mod test {
    use super::{PVec, Representation, RrbVec};

    #[test]
    fn interleaving_append_split_off_operations() {
        let mut vec_one = PVec::new();
        let mut value = 0;

        for size in 1..(32 * 8 + 32) {
            let mut vec_two = PVec::new();
            for _ in 0..size {
                vec_two.push(value);
                value += 1;
            }

            vec_one.append(&mut vec_two);

            let mid = vec_one.len() / 2;
            let mut right = vec_one.split_off(mid);

            vec_one.append(&mut right);
            value = vec_one.len();
        }

        for i in 0..value {
            assert_eq!(vec_one.get(i).cloned(), Some(i));
        }
    }

    #[test]
    fn sizes() {
        let pvec_size = std::mem::size_of::<PVec<u8>>();
        let rrbvec_size = std::mem::size_of::<RrbVec<u8>>();
        let flavor_size = std::mem::size_of::<Representation<u8>>();
        let vec_size = std::mem::size_of::<Vec<u8>>();

        println!("Size of PVec<u8>={}", pvec_size);
        println!("Size of RrbVec<u8>={}", rrbvec_size);
        println!("Size of Flavor<u8>={}", flavor_size);
        println!("Size of Vec<u8>={}", vec_size);
    }

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
    fn internal_state() {
        let mut input = create_input(16000);
        let mut vec = PVec::new();

        for mut input in input.iter_mut() {
            println!(
                "input is flat={} && is len is={}",
                input.0.is_flat(),
                input.len()
            );
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
}
