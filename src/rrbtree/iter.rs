use super::{Node, RrbTree, BRANCH_FACTOR};
use sharedptr::SharedPtr;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RrbTreeIter<T> {
    // - you should avoid heap allocation in iterators
    // - you can also preallocate space for focus, to avoid
    // additional allocations

    // keep references to pointers, nodes, or branches directly?
    focus: Vec<Node<T>>,
    indices: Vec<usize>,
    len: usize,
}

// impl<T: Clone + Debug> RrbTreeIter<T> {
// #[inline(always)]
// fn has_next_at_level(&self, level: usize) -> bool {
//     if let (Some(node), Some(index)) = (self.focus.get(level), self.indices.get(level)) {
//         (index + 1) < node.len()
//     } else {
//         false
//     }
// }
// }

impl<T: Clone> Iterator for RrbTreeIter<T> {
    type Item = ([Option<T>; BRANCH_FACTOR], usize);

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(mut node), Some(idx)) = (self.focus.last_mut(), self.indices.last_mut()) {
            loop {
                match *node {
                    Node::RelaxedBranch(ref mut branch_arc) => {
                        let branch = SharedPtr::make_mut(branch_arc);
                        node = branch.children[0].as_mut().unwrap();
                    }
                    Node::Branch(ref mut branch_arc) => {
                        let branch = SharedPtr::make_mut(branch_arc);
                        node = branch.children[0].as_mut().unwrap();
                    }
                    Node::Leaf(ref mut leaf_arc) => {
                        // let leaf = SharedPtr::make_mut(leaf_arc);
                        //return Some((leaf.elements, leaf.len))
                    }
                }
            }
        };

        return None;
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T: Clone + Debug> From<Option<Node<T>>> for RrbTreeIter<T> {
    fn from(root: Option<Node<T>>) -> Self {
        // experiment with predefined capacity of vectors (i.e. with_capacity)
        let mut focus = Vec::new();
        let mut indices = Vec::new();
        let mut len = 0;

        if let Some(root) = root {
            len = root.len();
            focus.push(root);
            indices.push(0);
        };

        RrbTreeIter {
            focus,
            indices,
            len,
        }
    }
}

impl<T: Clone + Debug> IntoIterator for RrbTree<T> {
    type Item = ([Option<T>; BRANCH_FACTOR], usize);
    type IntoIter = RrbTreeIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        RrbTreeIter::from(self.root)
    }
}
