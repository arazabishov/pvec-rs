use super::PVec;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct PVecIter<T> {
    // - you should avoid heap allocation in iterators
    pvec: PVec<T>,
    len: usize,
}

impl<T: Clone + Debug> Iterator for PVecIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T: Clone + Debug> IntoIterator for PVec<T> {
    type Item = T;
    type IntoIter = PVecIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        PVecIter {
            len: self.len(),
            pvec: self,
        }
    }
}
