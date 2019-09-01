use super::Flavor;
use super::PVec;

use std::fmt::Debug;

#[cfg(all(test, not(feature = "small_branch")))]
pub const BRANCH_FACTOR: usize = 32;

#[cfg(all(test, feature = "small_branch"))]
pub const BRANCH_FACTOR: usize = 4;

use core::iter::RrbVecIter;
use core::RrbVec;
use std::vec::IntoIter as VecIter;
use utils::sharedptr::Take;

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
            Flavor::Standard(vec_arc) => PVecIter::from_vec(vec_arc.take()),
            Flavor::Persistent(pvec) => PVecIter::from_rrbvec(pvec),
        }
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use super::PVec;
    use super::BRANCH_FACTOR;

    #[test]
    fn empty_pvec() {
        let pvec: PVec<usize> = PVec::new();
        let mut iter = pvec.into_iter();

        let size = iter.size_hint();
        let next = iter.next();

        assert_eq!(next, None);
        assert_eq!(size, (0, Some(0)));
    }

    #[test]
    fn pvec_has_tail_only() {
        let mut pvec = PVec::new();

        for i in 0..BRANCH_FACTOR {
            pvec.push(i);
        }

        for (i, val) in pvec.into_iter().enumerate() {
            assert_eq!(i, val);
        }
    }

    #[test]
    fn underlying_tree_has_multiple_levels() {
        let mut pvec = PVec::new();

        let mut val = 0;
        for _ in 0..(BRANCH_FACTOR * BRANCH_FACTOR * BRANCH_FACTOR) {
            pvec.push(val);
            val += 1;
        }

        for _ in 0..(BRANCH_FACTOR / 2) {
            pvec.push(val);
            val += 1;
        }

        for (i, val) in pvec.into_iter().enumerate() {
            assert_eq!(i, val);
        }
    }

    #[test]
    fn underlying_tree_is_relaxed() {
        let vec_size = 33;

        let mut vec = PVec::new();
        let mut vec_item = 0;

        for i in 0..128 {
            if i % 2 == 0 {
                let mut vec_temp = PVec::new();

                for _ in 0..vec_size {
                    vec_temp.push(vec_item);
                    vec_item += 1;
                }

                assert_eq!(vec_temp.len(), vec_size);

                vec.append(&mut vec_temp);

                assert_eq!(vec_temp.len(), 0);
            } else {
                for _ in 0..(vec_size + vec_size) {
                    vec.push(vec_item);
                    vec_item += 1;
                }
            }

            assert_eq!(vec.len(), vec_item);

            for i in 0..vec.len() {
                assert_eq!(*vec.get(i).unwrap(), i);
                assert_eq!(*vec.get_mut(i).unwrap(), i);
            }

            let mut vec_one_clone = vec.clone();
            for i in (0..vec_item).rev() {
                assert_eq!(vec_one_clone.pop().unwrap(), i);
            }

            assert_eq!(vec_one_clone.len(), 0);
            assert_eq!(vec.len(), vec_item);

            let vec_clone = vec.clone();
            for (i, val) in vec_clone.into_iter().enumerate() {
                assert_eq!(i, val);
            }
        }
    }
}
