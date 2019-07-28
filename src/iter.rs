use super::PVec;
use rrbtree::iter::RrbTreeIter;
use rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct PVecIter<T> {
    // - you should avoid heap allocation in iterators
    tree_iter: RrbTreeIter<T>,
    tree_len: usize,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
    index: usize,
    chunk: Option<([Option<T>; BRANCH_FACTOR], usize)>,
    chunk_index: usize,
}

impl<T: Clone + Debug> Iterator for PVecIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tree_len {
            if self.chunk.is_none() {
                self.chunk = self.tree_iter.next();
            }

            let chunk = self.chunk.as_mut().unwrap();
            if self.chunk_index >= chunk.1 {
                self.chunk_index = 0;
                self.chunk = self.tree_iter.next();
            }

            let chunk = self.chunk.as_mut().unwrap();
            let value = chunk.0[self.chunk_index].take();

            self.chunk_index += 1;
            self.index += 1;

            value
        } else if self.index < self.tree_len + self.tail_len {
            let index = self.index - self.tree_len;

            self.index += 1;
            self.tail[index].take()
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.tree_len + self.tail_len;
        (len, Some(len))
    }
}

impl<T: Clone + Debug> IntoIterator for PVec<T> {
    type Item = T;
    type IntoIter = PVecIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        PVecIter {
            tree_len: self.tree.len(),
            tree_iter: self.tree.into_iter(),
            tail_len: self.tail_len,
            tail: self.tail,
            index: 0,
            chunk: None,
            chunk_index: 0,
        }
    }
}

impl<T: Clone + Debug> ExactSizeIterator for PVecIter<T> {
    fn len(&self) -> usize {
        self.tree_len + self.tail_len
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
