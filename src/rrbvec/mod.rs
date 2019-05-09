use rrbtree::RrbTree;
use rrbtree::BRANCH_FACTOR;
use std::fmt::Debug;
use std::mem;

pub mod iter;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

impl<T: Clone + Debug> RrbVec<T> {
    pub fn new() -> Self {
        RrbVec {
            tree: RrbTree::new(),
            tail: new_branch!(),
            tail_len: 0,
        }
    }

    pub fn from(vec: &Vec<T>) -> Self {
        // ToDo: do something smart to pre-allocate space for all new items?

        let vec = vec.clone();
        let mut rrbvec = RrbVec::new();

        for item in vec.into_iter() {
            rrbvec.push(item)
        }

        rrbvec
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        if self.tree.len() > index {
            self.tree.get(index)
        } else {
            self.tail[index - self.tree.len()].as_ref()
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.tree.len() > index {
            self.tree.get_mut(index)
        } else {
            self.tail[index - self.tree.len()].as_mut()
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.tree.len() + self.tail_len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
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
                    for i in 0..that_tail_len {
                        self.tail[self.tail_len] = that_tail[i].take();
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

    #[inline(always)]
    pub fn split_off(&mut self, mid: usize) -> Self {
        let cloned = self.clone();
        self.split_right_at(mid);

        cloned.split_left_at(mid)
    }

    // ToDo: reconsider implementation of tree.split_at_* to
    // ToDo: avoid additional cloning, copying, etc
    fn split_right_at(&mut self, mid: usize) {
        // ToDo: see other adjustments which are
        // ToDo: made to tail after tree is split

        if mid == 0 {
            mem::replace(self, RrbVec::new());
        } else if mid < self.len() {
            // only need to cut the tail off
            if self.tree.len() <= mid {
                let new_tail_len = mid - self.tree.len();

                for i in new_tail_len..self.tail_len {
                    self.tail[i] = None;
                }

                self.tail_len = new_tail_len
            } else {
                let mut new_tree = self.tree.split_right_at(mid);
                let (new_tail, new_tail_len) = new_tree.pop();

                self.tree = new_tree;
                self.tail = new_tail;
                self.tail_len = new_tail_len;
            }
        } else if mid > self.len() {
            panic!()
        }
    }

    // ToDo: see other adjustments which are
    // ToDo: made to tail after tree is split
    fn split_left_at(mut self, mid: usize) -> Self {
        if mid == 0 {
            self
        } else if mid < self.len() {
            let remaining = self.len() - mid;

            if remaining <= self.tail_len {
                let mut tail = new_branch!();
                let mut tail_len = 0;

                for i in (self.tail_len - remaining)..self.tail_len {
                    tail[tail_len] = self.tail[i].take();
                    tail_len += 1;
                }

                return RrbVec {
                    tree: self.tree,
                    tail: tail,
                    tail_len: tail_len,
                };
            }

            let mut rrbvec = RrbVec {
                tree: self.tree.split_left_at(mid),
                tail: self.tail,
                tail_len: self.tail_len,
            };

            if rrbvec.tree.is_root_leaf() {
                if rrbvec.len() <= BRANCH_FACTOR {
                    // all values can fit into a single tail
                    let (mut new_tail, mut new_tail_len) = rrbvec.tree.pop();

                    for i in 0..rrbvec.tail_len {
                        new_tail[new_tail_len] = rrbvec.tail[i].take();
                        new_tail_len += 1;
                    }

                    rrbvec.tail = new_tail;
                    rrbvec.tail_len = new_tail_len;
                } else if rrbvec.tree.len() < BRANCH_FACTOR {
                    // root is leaf, but it is not fully dense
                    // hence, some of the values should be redistributed to the actual leaf

                    let (mut root, mut root_len) = rrbvec.tree.pop();
                    let mut index = 0;

                    while root_len < BRANCH_FACTOR && index < rrbvec.tail_len {
                        root[root_len] = rrbvec.tail[index].take();

                        root_len += 1;
                        index += 1;
                    }

                    rrbvec.tree.push(root, root_len);

                    let (mut new_tail, mut new_tail_len) = (new_branch!(), 0);
                    while index < rrbvec.tail_len {
                        new_tail[new_tail_len] = rrbvec.tail[index].take();

                        new_tail_len += 1;
                        index += 1;
                    }

                    rrbvec.tail = new_tail;
                    rrbvec.tail_len = new_tail_len;
                }
            }

            rrbvec
        } else if mid == self.len() {
            RrbVec::new()
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use super::{RrbVec, BRANCH_FACTOR};

    #[test]
    fn split_right_at() {
        let mut rrbvec = RrbVec::new();

        for i in 0..(BRANCH_FACTOR * BRANCH_FACTOR) {
            rrbvec.push(i);
        }

        for i in (0..BRANCH_FACTOR * BRANCH_FACTOR).rev() {
            rrbvec.split_right_at(i);
            assert_eq!(rrbvec.len(), i);

            for j in 0..i {
                assert_eq!(rrbvec.get(j).cloned(), Some(j));
            }
        }

        assert_eq!(rrbvec.tree.len(), 0);
        assert_eq!(rrbvec.tail_len, 0);
    }

    #[test]
    fn split_left_at() {
        let mut rrbvec = RrbVec::new();

        for i in 0..(BRANCH_FACTOR * BRANCH_FACTOR) {
            rrbvec.push(i);
        }

        let rrbvec_len = rrbvec.len();
        for i in 1..rrbvec_len + 1 {
            rrbvec = rrbvec.split_left_at(1);
            assert_eq!(rrbvec.len(), rrbvec_len - i);

            for j in 0..(rrbvec_len - i) {
                assert_eq!(rrbvec.get(j).cloned(), Some(i + j));
            }
        }

        assert_eq!(rrbvec.tree.len(), 0);
        assert_eq!(rrbvec.tail_len, 0);
    }

    #[test]
    fn interleaving_append_split_at_operations() {
        let mut rrbvec = RrbVec::new();
        let mut value = 0;

        for size in 1..(BRANCH_FACTOR * 8 + BRANCH_FACTOR) {
            let mut another_rrbvec = RrbVec::new();
            for _ in 0..size {
                another_rrbvec.push(value);
                value += 1;
            }

            rrbvec.append(&mut another_rrbvec);

            let mid = rrbvec.len() / 2;
            let mut right = rrbvec.split_off(mid);

            rrbvec.append(&mut right);
            value = rrbvec.len();
        }

        for i in 0..value {
            assert_eq!(rrbvec.get(i).cloned(), Some(i));
        }
    }
}
