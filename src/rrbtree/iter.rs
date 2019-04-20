use super::{Node, RrbTree, BRANCH_FACTOR};
use sharedptr::Take;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RrbTreeIter<T> {
    // - you should avoid heap allocation in iterators
    // - you can also preallocate space for nodes, to avoid
    // additional allocations

    // keep references to pointers, nodes, or elementses directly?
    branches: Vec<([Option<Node<T>>; BRANCH_FACTOR], usize)>,
    elements: Option<([Option<T>; BRANCH_FACTOR], usize)>,
    path: Vec<usize>,
    len: usize,
}

impl<T: Clone + Debug> Iterator for RrbTreeIter<T> {
    type Item = ([Option<T>; BRANCH_FACTOR], usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.elements.is_some() {
            return self.elements.take();
        }

        if self.branches.is_empty() {
            return None;
        }

        loop {
            let (nodes, len) = self.branches.last_mut().unwrap();
            let index = self.path.last().cloned().unwrap();

            if index < *len {
                let node = nodes[index].take();

                match node.unwrap() {
                    Node::Branch(branch_ptr) => {
                        let branch = branch_ptr.take();
                        self.branches.push((branch.children, branch.len));
                        self.path.push(0);
                    }
                    Node::RelaxedBranch(branch_ptr) => {
                        let branch = branch_ptr.take();
                        self.branches.push((branch.children, branch.len));
                        self.path.push(0);
                    }
                    Node::Leaf(leaf_ptr) => {
                        let leaf = leaf_ptr.take();
                        self.elements = Some((leaf.elements, leaf.len));

                        let index = self.path.pop().unwrap_or(0) + 1;
                        self.path.push(index);
                        break;
                    }
                };
            } else {
                self.branches.pop();
                self.path.pop();

                let index = self.path.pop().unwrap_or(0) + 1;
                self.path.push(index);
            }
        }

        return self.elements.take();
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

// ToDo: experiment with predefined capacity of vectors (i.e. with_capacity)
impl<T: Clone + Debug> From<Option<Node<T>>> for RrbTreeIter<T> {
    fn from(root: Option<Node<T>>) -> Self {
        if let Some(root) = root {
            let mut branches = Vec::new();
            let mut path = Vec::new();

            let len = root.len();

            return match root {
                Node::Branch(branch_ptr) => {
                    let branch = branch_ptr.take();
                    branches.push((branch.children, branch.len));
                    path.push(0);

                    RrbTreeIter {
                        branches,
                        elements: None,
                        path,
                        len,
                    }
                }
                Node::RelaxedBranch(branch_ptr) => {
                    let branch = branch_ptr.take();
                    branches.push((branch.children, branch.len));
                    path.push(0);

                    RrbTreeIter {
                        branches,
                        elements: None,
                        path,
                        len,
                    }
                }
                Node::Leaf(leaf_ptr) => {
                    let leaf = leaf_ptr.take();
                    let elements = Some((leaf.elements, leaf.len));

                    RrbTreeIter {
                        branches,
                        elements,
                        path,
                        len,
                    }
                }
            };
        }

        RrbTreeIter {
            branches: Vec::new(),
            elements: None,
            path: Vec::new(),
            len: 0,
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

#[cfg(test)]
#[macro_use]
mod test {
    use super::RrbTree;
    use super::BRANCH_FACTOR;

    #[test]
    fn empty_tree() {
        let tree: RrbTree<usize> = RrbTree::new();
        let mut iter = tree.into_iter();

        let size = iter.size_hint();
        let next = iter.next();

        assert_eq!(next, None);
        assert_eq!(size, (0, Some(0)));
    }

    #[test]
    fn root_is_leaf() {
        let mut elements = new_branch!();

        for i in 0..BRANCH_FACTOR {
            elements[i] = Some(i);
        }

        let mut tree = RrbTree::new();
        tree.push(elements, BRANCH_FACTOR);

        let (chunk, size) = tree.into_iter().next().unwrap();
        assert_eq!(size, BRANCH_FACTOR);

        for index in 0..BRANCH_FACTOR {
            assert_eq!(index, chunk[index].unwrap());
        }
    }

    #[test]
    fn root_has_two_leaves() {
        let mut elements_one = new_branch!();
        let mut elements_two = new_branch!();

        for i in 0..BRANCH_FACTOR {
            elements_one[i] = Some(i);
        }

        for i in 0..(BRANCH_FACTOR / 2) {
            elements_two[i] = Some(i);
        }

        let mut tree = RrbTree::new();
        tree.push(elements_one, BRANCH_FACTOR);
        tree.push(elements_two, BRANCH_FACTOR / 2);
        let mut iter = tree.into_iter();

        let (chunk, size) = iter.next().unwrap();
        assert_eq!(size, BRANCH_FACTOR);

        for index in 0..BRANCH_FACTOR {
            assert_eq!(index, chunk[index].unwrap());
        }

        let (chunk, size) = iter.next().unwrap();
        assert_eq!(size, BRANCH_FACTOR / 2);

        for index in 0..BRANCH_FACTOR / 2 {
            assert_eq!(index, chunk[index].unwrap());
        }
    }

    #[test]
    fn root_has_more_than_three_levels() {
        let mut tree = RrbTree::new();
        for _ in 0..(BRANCH_FACTOR * BRANCH_FACTOR) + BRANCH_FACTOR {
            let mut elements = new_branch!();

            for j in 0..BRANCH_FACTOR {
                elements[j] = Some(j);
            }

            tree.push(elements, BRANCH_FACTOR);
        }

        let mut iter = tree.into_iter();
        for _ in 0..(BRANCH_FACTOR * BRANCH_FACTOR) + BRANCH_FACTOR {
            let (chunk, size) = iter.next().unwrap();
            assert_eq!(size, BRANCH_FACTOR);

            for index in 0..BRANCH_FACTOR {
                assert_eq!(index, chunk[index].unwrap());
            }
        }
    }
}
