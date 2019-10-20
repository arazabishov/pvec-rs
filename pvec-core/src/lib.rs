#[macro_use]
#[cfg(feature = "serde-serializer")]
extern crate serde_json;

extern crate pvec_utils as utils;
#[cfg(all(feature = "arc", feature = "rayon-iter"))]
extern crate rayon;

#[cfg(feature = "serde-serializer")]
extern crate serde;

use rrbtree::RrbTree;
use rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;
use std::mem;
use std::ops;

pub mod iter;

#[macro_use]
mod rrbtree;

#[cfg(feature = "serde-serializer")]
mod serializer;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RbVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

macro_rules! impl_vec {
    ($vec:ident) => {
        impl<T: Clone + Debug> Default for $vec<T> {
            fn default() -> Self {
                $vec::new()
            }
        }

        impl<T: Clone + Debug> $vec<T> {
            pub fn new() -> Self {
                $vec {
                    tree: RrbTree::new(),
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

                    self.tree.push(tail, self.tail_len);
                    self.tail_len = 0;
                }
            }

            pub fn pop(&mut self) -> Option<T> {
                if self.is_empty() {
                    return None;
                }

                if self.tail_len == 0 {
                    let (new_tail, new_tail_len) = self.tree.pop();
                    mem::replace(&mut self.tail, new_tail);

                    self.tail_len = new_tail_len;
                }

                let item = self.tail[self.tail_len - 1].take();
                self.tail_len -= 1;

                item
            }

            pub fn get(&self, index: usize) -> Option<&T> {
                if self.tree.len() > index {
                    self.tree.get(index)
                } else {
                    self.tail[index - self.tree.len()].as_ref()
                }
            }

            pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
                if self.tree.len() > index {
                    self.tree.get_mut(index)
                } else {
                    self.tail[index - self.tree.len()].as_mut()
                }
            }

            pub fn len(&self) -> usize {
                self.tree.len() + self.tail_len
            }

            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }
        }

        impl<T: Clone + Debug> ops::Index<usize> for $vec<T> {
            type Output = T;

            fn index(&self, index: usize) -> &T {
                self.get(index).unwrap_or_else(|| {
                    panic!(
                        "index `{}` out of bounds in RrbVec of length `{}`",
                        index,
                        self.len()
                    )
                })
            }
        }

        impl<T: Clone + Debug> ops::IndexMut<usize> for $vec<T> {
            fn index_mut(&mut self, index: usize) -> &mut T {
                let len = self.len();
                self.get_mut(index).unwrap_or_else(|| {
                    panic!(
                        "index `{}` out of bounds in RrbVec of length `{}`",
                        index, len
                    )
                })
            }
        }
    };
}

impl_vec!(RbVec);
impl_vec!(RrbVec);

impl<T: Clone + Debug> RbVec<T> {
    pub fn split_off(&mut self, mid: usize) -> Self {
        if mid == 0 {
            mem::replace(self, Self::new())
        } else if mid < self.len() {
            if self.tree.len() > mid {
                let chunks_count = (self.tree.len() - mid) / BRANCH_FACTOR;
                let mut chunks = Vec::with_capacity(chunks_count);

                while self.tree.len() - BRANCH_FACTOR > mid {
                    chunks.push(self.tree.pop());
                }

                let (mut left_tail, mut left_tail_len) = self.tree.pop();

                let from_i = mid - self.tree.len();
                let to_len = left_tail_len;

                let mut right = Self::new();
                for i in from_i..to_len {
                    right.push(left_tail[i].take().unwrap());
                    left_tail_len -= 1;
                }

                let mut right_tail = mem::replace(&mut self.tail, left_tail);
                let right_tail_len = mem::replace(&mut self.tail_len, left_tail_len);

                for (mut chunk, chunk_len) in chunks.into_iter().rev() {
                    for i in 0..chunk_len {
                        right.push(chunk[i].take().unwrap());
                    }
                }

                for i in 0..right_tail_len {
                    right.push(right_tail[i].take().unwrap());
                }

                right
            } else {
                let left_tail_len = mid - self.tree.len();

                let mut right_tail = new_branch!();
                let mut right_tail_len = 0;

                for i in left_tail_len..self.tail_len {
                    right_tail[right_tail_len] = self.tail[i].take();
                    right_tail_len += 1;
                }

                self.tail_len = left_tail_len;

                RbVec {
                    tree: RrbTree::new(),
                    tail: right_tail,
                    tail_len: right_tail_len,
                }
            }
        } else if mid == self.len() {
            Self::new()
        } else {
            panic!()
        }
    }

    pub fn append(&mut self, that: &mut RbVec<T>) {
        let that_is_empty = that.is_empty();

        let that_tree = mem::replace(&mut that.tree, RrbTree::new());
        let that_tail = mem::replace(&mut that.tail, new_branch!());

        let that_tail_len = that.tail_len;
        that.tail_len = 0;

        if self.is_empty() {
            self.tree = that_tree;
            self.tail = that_tail;
            self.tail_len = that_tail_len;
        } else if !that_is_empty {
            let that_vec = RbVec {
                tree: that_tree,
                tail: that_tail,
                tail_len: that_tail_len,
            };

            for value in that_vec.into_iter() {
                self.push(value);
            }
        }
    }
}

impl<T: Clone + Debug> RrbVec<T> {
    pub fn split_off(&mut self, mid: usize) -> Self {
        if mid == 0 {
            mem::replace(self, RrbVec::new())
        } else if mid < self.len() {
            if self.tree.len() > mid {
                let right_tree = self.tree.split_off(mid);

                let (left_tail, left_tail_len) = self.tree.pop();

                let right_tail = mem::replace(&mut self.tail, left_tail);
                let right_tail_len = mem::replace(&mut self.tail_len, left_tail_len);

                let mut right = RrbVec {
                    tree: right_tree,
                    tail: right_tail,
                    tail_len: right_tail_len,
                };

                if right.tree.is_root_leaf() {
                    if right.len() <= BRANCH_FACTOR {
                        // all values can fit into a single tail
                        let (mut new_tail, mut new_tail_len) = right.tree.pop();

                        for i in 0..right.tail_len {
                            new_tail[new_tail_len] = right.tail[i].take();
                            new_tail_len += 1;
                        }

                        right.tail = new_tail;
                        right.tail_len = new_tail_len;
                    } else if right.tree.len() < BRANCH_FACTOR {
                        // root is leaf, but it is not fully dense
                        // hence, some of the values should be redistributed to the actual leaf

                        let (mut root, mut root_len) = right.tree.pop();
                        let mut index = 0;

                        while root_len < BRANCH_FACTOR && index < right.tail_len {
                            root[root_len] = right.tail[index].take();

                            root_len += 1;
                            index += 1;
                        }

                        right.tree.push(root, root_len);

                        let (mut new_tail, mut new_tail_len) = (new_branch!(), 0);
                        while index < right.tail_len {
                            new_tail[new_tail_len] = right.tail[index].take();

                            new_tail_len += 1;
                            index += 1;
                        }

                        right.tail = new_tail;
                        right.tail_len = new_tail_len;
                    }
                }

                right
            } else {
                let left_tail_len = mid - self.tree.len();

                let mut right_tail = new_branch!();
                let mut right_tail_len = 0;

                for i in left_tail_len..self.tail_len {
                    right_tail[right_tail_len] = self.tail[i].take();
                    right_tail_len += 1;
                }

                self.tail_len = left_tail_len;

                RrbVec {
                    tree: RrbTree::new(),
                    tail: right_tail,
                    tail_len: right_tail_len,
                }
            }
        } else if mid == self.len() {
            RrbVec::new()
        } else {
            panic!()
        }
    }

    pub fn append(&mut self, that: &mut RrbVec<T>) {
        if self.is_empty() {
            self.tail = mem::replace(&mut that.tail, new_branch!());
            self.tree = mem::replace(&mut that.tree, RrbTree::new());

            self.tail_len = that.tail_len;
            that.tail_len = 0;
        } else if !that.is_empty() {
            let mut that_tail = mem::replace(&mut that.tail, new_branch!());
            let that_tail_len = that.tail_len;

            that.tail_len = 0;

            if that.tree.is_empty() {
                if self.tail_len == BRANCH_FACTOR {
                    let self_tail = mem::replace(&mut self.tail, that_tail);
                    let self_tail_len = self.tail_len;

                    self.tail_len = that_tail_len;
                    self.tree.push(self_tail, self_tail_len);
                } else if self.tail_len + that_tail_len <= BRANCH_FACTOR {
                    for item in that_tail.iter_mut().take(that_tail_len) {
                        self.tail[self.tail_len] = item.take();
                        self.tail_len += 1;
                    }
                } else {
                    let mut self_tail = mem::replace(&mut self.tail, new_branch!());
                    let mut self_tail_i = mem::replace(&mut self.tail_len, 0);
                    let mut that_tail_i = 0;

                    while self_tail_i < BRANCH_FACTOR && that_tail_i < that_tail_len {
                        self_tail[self_tail_i] = that_tail[that_tail_i].take();

                        self_tail_i += 1;
                        that_tail_i += 1;
                    }

                    self.tree.push(self_tail, self_tail_i);

                    let that_tail_elements_left = that_tail_len - that_tail_i;
                    for i in 0..that_tail_elements_left {
                        self.tail[i] = that_tail[that_tail_i].take();
                        that_tail_i += 1;
                    }

                    self.tail_len = that_tail_elements_left;
                }
            } else {
                if self.tail_len == 0 {
                    self.tail = that_tail;
                    self.tail_len = that_tail_len;
                } else {
                    let self_tail = mem::replace(&mut self.tail, that_tail);
                    let self_tail_len = self.tail_len;

                    self.tail_len = that_tail_len;
                    self.tree.push(self_tail, self_tail_len);
                }

                self.tree.append(&mut that.tree);
            }
        }

        self.push_tail();
    }
}

macro_rules! make_tests {
    ($vec:ident, $module:ident) => {

        #[cfg(test)]
        mod $module {
            use super::$vec;
            use super::BRANCH_FACTOR;

            #[test]
            fn len_matches_actual_size() {
                const N: usize = 5000;

                let mut vec = $vec::new();

                for i in 0..N {
                    vec.push(i);
                }

                assert_eq!(vec.len(), N);

                for i in 0..N {
                    assert_eq!(*vec.get(i).unwrap(), i);
                }
            }

            #[test]
            fn len_matches_len_cloned() {
                const N: usize = 5000;

                let mut vec = $vec::new();

                for i in 0..N {
                    vec.push(i);
                }

                let vec_0 = vec.clone();
                assert_eq!(vec.len(), N);
                assert_eq!(vec_0.len(), N);

                for i in 0..N {
                    vec.push(i);
                }

                assert_eq!(vec.len(), 2 * N);
                assert_eq!(vec_0.len(), N);

                for i in 0..N {
                    assert_eq!(*vec.get(i).unwrap(), i);
                    assert_eq!(*vec_0.get(i).unwrap(), i);
                }

                for i in 0..N {
                    assert_eq!(*vec.get(i + N).unwrap(), i);
                }
            }

            #[test]
            fn mutate_in_place_must_not_mutate_cloned_vec() {
                const N: usize = 32 * 4;

                let mut vec = $vec::new();

                for i in 0..N {
                    vec.push(i);
                }

                let vec_0 = vec.clone();
                assert_eq!(vec.len(), N);
                assert_eq!(vec_0.len(), N);

                for i in 0..(N / 2) {
                    *vec.get_mut(i).unwrap() += 1;
                }

                assert_eq!(vec.len(), N);
                assert_eq!(vec_0.len(), N);

                for i in 0..(N / 2) {
                    assert_eq!(*vec.get(i).unwrap(), i + 1);
                    assert_eq!(*vec_0.get(i).unwrap(), i);
                }

                // the second half ought to be untouched
                for i in N / 2..N {
                    assert_eq!(*vec.get(i).unwrap(), i);
                    assert_eq!(*vec_0.get(i).unwrap(), i);
                    assert_eq!(
                        vec.get(i).unwrap() as *const usize,
                        vec_0.get(i).unwrap() as *const usize
                    );
                }
            }

            #[test]
            fn pop_must_not_mutate_cloned_vec() {
                const N: usize = 32 * 4;

                let mut vec = $vec::new();

                for i in 0..N {
                    vec.push(i);
                }

                let vec_0 = vec.clone();
                assert_eq!(vec.len(), N);
                assert_eq!(vec_0.len(), N);

                for _ in 0..(N / 2) {
                    vec.pop();
                }

                assert_eq!(vec.len(), N / 2);
                assert_eq!(vec_0.len(), N);

                for i in 0..(N / 2) {
                    assert_eq!(*vec.get(i).unwrap(), i);
                    assert_eq!(*vec_0.get(i).unwrap(), i);
                }

                for i in N / 2..N {
                    assert_eq!(*vec_0.get(i).unwrap(), i);
                }
            }

            #[test]
            fn push_pop_must_return_expected_values() {
                const N: usize = 32 * 4;

                let mut vec = $vec::new();

                for i in 0..N {
                    vec.push(i)
                }

                assert_eq!(vec.len(), N);

                for i in (0..N).rev() {
                    assert_eq!(vec.pop().unwrap(), i);
                }

                for i in 0..N {
                    vec.push(i)
                }

                assert_eq!(vec.len(), N);

                for i in (0..N).rev() {
                    assert_eq!(vec.pop().unwrap(), i);
                }

                assert_eq!(vec.len(), 0);
            }

            #[test]
            fn append_must_maintain_vectors_in_correct_state_after_clone() {
                let mut vec_l = $vec::new();
                let mut vec_c = $vec::new();
                let mut vec_r = $vec::new();

                let mut branch_value = 0;

                for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR * BRANCH_FACTOR {
                    vec_l.push(branch_value);
                    branch_value += 1;
                }

                for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR {
                    vec_c.push(branch_value);
                    branch_value += 1;
                }

                for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR {
                    vec_r.push(branch_value);
                    branch_value += 1;
                }

                let vec_l_clone = vec_l.clone();
                let vec_c_clone = vec_c.clone();
                let vec_r_clone = vec_r.clone();

                vec_l.append(&mut vec_c);
                vec_l.append(&mut vec_r);

                assert_eq!(
                    vec_l.len(),
                    vec_l_clone.len() + vec_c_clone.len() + vec_r_clone.len()
                );

                let mut branch_test_value = 0;

                for i in 0..vec_l_clone.len() {
                    assert_eq!(*vec_l_clone.get(i).unwrap(), branch_test_value);
                    branch_test_value += 1;
                }

                for i in 0..vec_c_clone.len() {
                    assert_eq!(*vec_c_clone.get(i).unwrap(), branch_test_value);
                    branch_test_value += 1;
                }

                for i in 0..vec_r_clone.len() {
                    assert_eq!(*vec_r_clone.get(i).unwrap(), branch_test_value);
                    branch_test_value += 1;
                }
            }

            fn interleaving_different_operations_must_maintain_correct_internal_state(vec_size: usize) {
                let mut vec = $vec::new();
                let mut vec_item = 0;

                for i in 0..128 {
                    if i % 2 == 0 {
                        let mut vec_temp = $vec::new();

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
                }

                assert_eq!(vec.len(), vec_item);

                let mut vec_clone = vec.clone();
                for i in (0..vec_item).rev() {
                    assert_eq!(vec_clone.pop().unwrap(), i);

                    for j in 0..vec_clone.len() {
                        assert_eq!(*vec_clone.get(j).unwrap(), j);
                        assert_eq!(*vec_clone.get_mut(j).unwrap(), j);
                    }
                }
            }

            #[test]
            fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_4() {
                interleaving_different_operations_must_maintain_correct_internal_state(4);
            }

            #[test]
            fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_5() {
                interleaving_different_operations_must_maintain_correct_internal_state(5);
            }

            #[test]
            fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_16() {
                interleaving_different_operations_must_maintain_correct_internal_state(16);
            }

            #[test]
            fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_17() {
                interleaving_different_operations_must_maintain_correct_internal_state(17);
            }

            #[test]
            fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_32() {
                interleaving_different_operations_must_maintain_correct_internal_state(32);
            }

            #[test]
            fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_33() {
                interleaving_different_operations_must_maintain_correct_internal_state(33);
            }

            #[test]
            fn interleaving_push_and_append_operations_must_maintain_correct_internal_state_for_var_sizes_32() {
                let mut vec_one = $vec::new();

                for i in 0..32 {
                    vec_one.push(i);
                }

                let mut vec_two = $vec::new();

                for i in 0..1024 {
                    if i % 2 == 0 {
                        vec_two.push(i);
                    } else {
                        vec_two.append(&mut vec_one.clone());
                    }

                    for k in 0..vec_two.len() {
                        vec_two.get(k).unwrap();
                    }
                }
            }

            #[test]
            fn zero_sized_values() {
                let mut v = $vec::new();
                assert_eq!(v.len(), 0);

                v.push(());
                assert_eq!(v.len(), 1);

                v.push(());
                assert_eq!(v.len(), 2);
                assert_eq!(v.pop(), Some(()));
                assert_eq!(v.pop(), Some(()));
                assert_eq!(v.pop(), None);

                assert_eq!(v.len(), 0);

                v.push(());
                assert_eq!(v.len(), 1);

                v.push(());
                assert_eq!(v.len(), 2);

                for i in 0..v.len() {
                    v.get(i);
                }
                assert_eq!(v.len(), 2);

                v.push(());
                assert_eq!(v.len(), 3);

                v.push(());
                assert_eq!(v.len(), 4);

                for i in 0..v.len() {
                    v.get_mut(i);
                }
                assert_eq!(v.len(), 4);
            }

            #[test]
            fn interleaving_append_split_off_operations() {
                let mut vec = $vec::new();
                let mut value = 0;

                for size in 1..(BRANCH_FACTOR * 8 + BRANCH_FACTOR) {
                    let mut another_vec = $vec::new();
                    for _ in 0..size {
                        another_vec.push(value);
                        value += 1;
                    }

                    vec.append(&mut another_vec);

                    let mid = vec.len() / 2;
                    let mut right = vec.split_off(mid);

                    vec.append(&mut right);
                    value = vec.len();
                }

                for i in 0..value {
                    assert_eq!(vec.get(i).cloned(), Some(i));
                }
            }

            #[test]
            fn split_off_by_one() {
                let mut vec = $vec::new();

                for i in 0..(BRANCH_FACTOR * BRANCH_FACTOR * BRANCH_FACTOR + (BRANCH_FACTOR / 2)) {
                    vec.push(i);
                }

                for i in (0..BRANCH_FACTOR * BRANCH_FACTOR * BRANCH_FACTOR + (BRANCH_FACTOR / 2)).rev() {
                    let mut other = vec.split_off(i);
                    assert_eq!(other.pop(), Some(i));
                }

                assert!(vec.is_empty());
            }
        }
    };
}

make_tests!(RbVec, rbvec);
make_tests!(RrbVec, rrbvec);
