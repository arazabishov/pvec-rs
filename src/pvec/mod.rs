extern crate serde;
extern crate serde_json;

use pvec::rrbtree::BRANCH_FACTOR;
use pvec::rrbtree::RrbTree;
use self::serde::ser::Serialize;
use std::fmt::Debug;
use std::mem;
use std::ops;

#[macro_use]
mod rrbtree;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct PVec<T> {
    tree: RrbTree<T>,
    tail: [Option<T>; BRANCH_FACTOR],
    tail_len: usize,
}

impl<T: Clone + Debug + Serialize> PVec<T> {
    pub fn new() -> Self {
        PVec {
            tree: RrbTree::new(),
            tail: new_branch!(),
            tail_len: 0,
        }
    }

    #[cold]
    pub fn push(&mut self, item: T) {
        self.tail[self.tail_len] = Some(item);
        self.tail_len += 1;

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
            self.tail_len = BRANCH_FACTOR;

            let new_tail = self.tree.pop();
            mem::replace(&mut self.tail, new_tail);
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

    // ToDo: get rid of unnecessary allocations of RrbTree in corner cases.
    // ToDo: this can be solved by making tree optional (see todoist for the task)

    // ToDo: abstract push_tail function away
    // ToDo: consider switching out tail (array) in PVec with Leaf
    pub fn append(&mut self, that: &mut PVec<T>) {
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

        if self.tail_len == BRANCH_FACTOR {
            let tail = mem::replace(&mut self.tail, new_branch!());

            self.tree.push(tail, self.tail_len);
            self.tail_len = 0;
        }
    }
}

impl<T: Clone + Debug + Serialize> Default for PVec<T> {
    fn default() -> Self {
        PVec::new()
    }
}

impl<T: Clone + Debug + Serialize> ops::Index<usize> for PVec<T> {
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

impl<T: Clone + Debug + Serialize> ops::IndexMut<usize> for PVec<T> {
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

mod json {
    extern crate serde;
    extern crate serde_json;

    use self::serde::ser::{Serialize, Serializer, SerializeStruct};
    use super::PVec;

    impl<T> Serialize for PVec<T>
        where
            T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
            where
                S: Serializer,
        {
            let mut serde_state = serializer.serialize_struct("PVec", 1)?;
            serde_state.serialize_field("tree", &self.tree)?;
            serde_state.serialize_field("tail", &self.tail)?;
            serde_state.serialize_field("tail_len", &self.tail_len)?;
            serde_state.end()
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate serde;
    extern crate serde_json;

    use super::BRANCH_FACTOR;
    use super::PVec;

    #[test]
    fn concat_must_return_expected_result() {
        let mut pvec_l = PVec::new();
        let mut pvec_c = PVec::new();
        let mut pvec_r = PVec::new();

        let mut branch_i = 0;

        for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR * BRANCH_FACTOR {
            pvec_l.push(branch_i);
            branch_i += 1;
        }

        for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR {
            pvec_c.push(branch_i);
            branch_i += 1;
        }

        for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR {
            pvec_r.push(branch_i);
            branch_i += 1;
        }

        let pvec_l_clone = pvec_l.clone();
        let pvec_c_clone = pvec_c.clone();
        let pvec_r_clone = pvec_r.clone();

        pvec_l.append(&mut pvec_c);
        pvec_l.append(&mut pvec_r);

        for i in 0..pvec_l_clone.len() {
            println!("pvec_l_clone: item={:?}", pvec_l_clone.get(i))
        }

        println!("=====");

        for i in 0..pvec_c_clone.len() {
            println!("pvec_c_clone: item={:?}", pvec_c_clone.get(i));
        }

        println!("=====");

        for i in 0..pvec_r_clone.len() {
            println!("pvec_r_clone: item={:?}", pvec_c_clone.get(i));
        }

        println!("#####");

        // println!("tree_l={}", serde_json::to_string(&tree_l).unwrap());
        // println!("tree_c={}", serde_json::to_string(&tree_c).unwrap());
        // println!("tree_r={}", serde_json::to_string(&tree_r).unwrap());

        // println!(
        //     "tree_l_clone={}",
        //     serde_json::to_string(&tree_l_clone).unwrap()
        // );
        // println!(
        //     "tree_c_clone={}",
        //     serde_json::to_string(&tree_c_clone).unwrap()
        // );
        // println!(
        //     "tree_r_clone={}",
        //     serde_json::to_string(&tree_r_clone).unwrap()
        // );

        for i in 0..pvec_l.len() {
            println!("pvec_l: item={:?}", pvec_l.get(i))
        }

        for i in 0..pvec_c.len() {
            println!("pvec_c: item={:?}", pvec_c.get(i))
        }

        for i in 0..pvec_r.len() {
            println!("pvec_r: item={:?}", pvec_r.get(i))
        }
    }

    fn append_pvec_must_maintain_correct_internal_structure(vec_size: usize) {
        println!("vec_size={}", vec_size);

        let mut vec_one = PVec::new();
        let mut vec_two_item = 0;

        for i in 0..1024 {
            if i % 2 == 0 {
                let mut vec_two = PVec::new();

                for _ in 0..vec_size {
                    vec_two.push(vec_two_item);
                    vec_two_item += 1;
                }

                assert_eq!(vec_two.len(), vec_size);

                vec_one.append(&mut vec_two);

                assert_eq!(vec_two.len(), 0);
            } else {
                for _ in 0..(vec_size + vec_size) {
                    vec_one.push(vec_two_item);
                    vec_two_item += 1;
                }
            }

            assert_eq!(vec_one.len(), vec_two_item);

            for i in 0..vec_one.len() {
                assert_eq!(*vec_one.get(i).unwrap(), i);
            }
        }

        assert_eq!(vec_one.len(), vec_two_item);

        // todo: make assertions regarding the state of both vectors, not just one
        // todo: add a test case where both vectors (or either of them) are empty and appended one to another
        // todo: add a test case with cloning both vectors in process of modification
    }

    #[test]
    fn append_pvec_must_maintain_correct_internal_structure_for_different_sizes() {
        for vec_size in 0..40 {
            append_pvec_must_maintain_correct_internal_structure(vec_size);
        }
    }

    #[test]
    fn append_pvec() {
        let mut vec_one = PVec::new();
        let mut vec_two = PVec::new();

        // ToDo: BRANCH_FACTOR == 32: crashes at 50 with called `Option::unwrap()` on a `None` value
        // ToDo: BRANCH_FACTOR == 32: crashes at 1000 with unreachable code
        // ToDo: BRANCH_FACTOR == 32: crashes at 5000 with `assertion failed: shift.0 >= BITS_PER_LEVEL`

        // ToDo: BRANCH_FACTOR == 4: crashes at 9 with unreachable code (sort of fixed)
        // ToDo: BRANCH_FACTOR == 4: crashes at 33 with `assertion failed: shift.0 >= BITS_PER_LEVEL` (sort of fixed)

        // ToDo: BRANCH_FACTOR == 4: crashes at 70 with 'called `Option::unwrap()` on a `None` value
        // ToDo: (pay attention to vec_two_before_append, it has invalid leaves after rebalancing)

        // Weird observation when running test on size 21 is that there is a RelaxedBranch created
        // for completely balanced node.. This might happen when tree starts from RelaxedBranch, and
        // then more leaves are pushed down into it and it becomes full. It is still strange though...

        // 3, 21
        for i in 0..70 {
            vec_one.push(i);
            vec_two.push(i);
        }

        // ToDo: patch get_mut as well as you did the get()

        // println!("vec_one={}", serde_json::to_string(&vec_one).unwrap());
        // println!("vec_two={}", serde_json::to_string(&vec_two).unwrap());

        for j in 0..256 {
            // println!("******** j={}", j);

            for i in 0..16 {
                // println!("######## append-i={}", i);

                vec_two.append(&mut vec_one.clone());

                // println!("vec_two={}", serde_json::to_string(&vec_two).unwrap());
                //
                //                if i == 1 {
                //                    break;
                //                }

                // println!("vec_two={}", serde_json::to_string(&vec_two).unwrap());
            }
            //
            //            if j == 0 {
            //                break;
            //            }
        }

        // println!("vec_two_len={}", vec_two.len());
        // println!("vec_two={}", serde_json::to_string(&vec_two).unwrap());

        for i in 0..vec_two.len() {
            // vec_two.get(i).unwrap();
            // println!("vec_two.get({}) -> {:?}", i, vec_two.get(i));
        }
    }
}
