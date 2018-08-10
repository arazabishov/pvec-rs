use pvec::rrbtree::RrbTree;
use pvec::rrbtree::BRANCH_FACTOR;
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

impl<T: Clone + Debug> PVec<T> {
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
                    let mut self_tail_i = self.tail_len;
                    let mut that_tail_i = 0;

                    while self_tail_i < BRANCH_FACTOR && that_tail_i < that_tail_len {
                        self.tail[self_tail_i] = that_tail[that_tail_i].take();

                        self_tail_i += 1;
                        that_tail_i += 1;
                    }

                    let self_tail = mem::replace(&mut self.tail, new_branch!());
                    let self_tail_len = self.tail_len;

                    self.tail_len = 0;

                    for i in 0..that_tail_len {
                        self.tail[i] = that_tail[that_tail_i].take();
                        self.tail_len += 1;

                        that_tail_i += 1;
                    }

                    self.tree.push(self_tail, self_tail_len);
                }
            } else {
                if self.tail_len == 0 {
                    self.tail = that_tail;
                    self.tail_len = that_tail_len;
                } else {
                    let self_tail = mem::replace(&mut self.tail, that_tail);
                    let self_tail_len = self.tail_len;

                    self.tail_len = that_tail_len;

                    // ToDo: when pushing a tail, sizes within the tree have to be re-calculated
                    // ToDo: otherwise all table sizes will be invalid!

                    // ToDo: check how this implementation works when RelaxedBranches are enforced!
                    // ToDo: implement serde serializers for PVec for visualisation
                    self.tree.push(self_tail, self_tail_len);
                }

                self.tree.append(&mut that.tree);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::PVec;
    use super::BRANCH_FACTOR;

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
}
