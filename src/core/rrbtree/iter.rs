use super::{get_branch_index, Index, Leaf, Node, RrbTree, Shift, BRANCH_FACTOR};
use super::{SharedPtr, Take};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RrbTreeIter<T> {
    root: Option<Node<T>>,
    root_len: usize,
    root_shift: Shift,
    head_idx: usize,
    tail_idx: usize,
}

impl<T: Clone + Debug> Node<T> {
    #[inline(always)]
    fn take(mut node: &mut Option<Node<T>>, mut idx: Index, mut shift: Shift) -> Option<Leaf<T>> {
        while !shift.is_leaf_level() {
            if let Some(it) = node {
                match *it {
                    Node::RelaxedBranch(ref mut ptr) => {
                        debug_assert!(shift.0 > 0);

                        let branch = SharedPtr::make_mut(ptr);

                        let sizes = &mut branch.sizes;
                        let branch_index = get_branch_index(sizes, idx);

                        if branch_index != 0 {
                            idx = Index(idx.0 - sizes[branch_index - 1].unwrap());
                        }

                        node = &mut branch.children[branch_index];
                        shift = shift.dec();
                    }
                    Node::Branch(ref mut ptr) => {
                        debug_assert!(shift.0 > 0);

                        let branch = SharedPtr::make_mut(ptr);

                        node = &mut branch.children[idx.child(shift)];
                        shift = shift.dec();
                    }
                    Node::Leaf(..) => unreachable!(),
                }
            }
        }

        node.take().map(|node| node.into_leaf().take())
    }
}

impl<T: Clone + Debug> Iterator for RrbTreeIter<T> {
    type Item = ([Option<T>; BRANCH_FACTOR], usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.head_idx <= self.tail_idx {
            let head_idx = self.head_idx;
            let root_shift = self.root_shift;

            let leaf = Node::take(&mut self.root, Index(head_idx), root_shift);

            if let Some(it) = leaf {
                self.head_idx += it.len;
                return Some((it.elements, it.len));
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.root_len, Some(self.root_len))
    }
}

impl<T: Clone + Debug> DoubleEndedIterator for RrbTreeIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.head_idx <= self.tail_idx {
            let tail_idx = self.tail_idx;
            let root_shift = self.root_shift;

            let leaf = Node::take(&mut self.root, Index(tail_idx), root_shift);

            if let Some(it) = leaf {
                if it.len > self.tail_idx {
                    self.tail_idx = 0;
                } else {
                    self.tail_idx -= it.len;
                }

                return Some((it.elements, it.len));
            }
        }

        None
    }
}

impl<T: Clone + Debug> ExactSizeIterator for RrbTreeIter<T> {
    fn len(&self) -> usize {
        self.root_len
    }
}

impl<T: Clone + Debug> IntoIterator for RrbTree<T> {
    type Item = ([Option<T>; BRANCH_FACTOR], usize);
    type IntoIter = RrbTreeIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut tail_index = self.root_len.0;

        if tail_index > 0 {
            tail_index -= 1
        }

        RrbTreeIter {
            root: self.root,
            root_len: self.root_len.0,
            root_shift: self.shift,
            head_idx: 0,
            tail_idx: tail_index,
        }
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use super::RrbTree;
    use super::BRANCH_FACTOR;

    #[test]
    fn empty_tree() {
        let tree_one: RrbTree<usize> = RrbTree::new();
        let tree_two: RrbTree<usize> = RrbTree::new();

        let mut iter_one = tree_one.into_iter();
        let mut iter_two = tree_two.into_iter();

        assert_eq!(iter_one.size_hint(), (0, Some(0)));
        assert_eq!(iter_one.next(), None);

        assert_eq!(iter_two.size_hint(), (0, Some(0)));
        assert_eq!(iter_two.next(), None);
    }

    #[test]
    fn root_is_leaf() {
        let mut elements = new_branch!();

        for i in 0..BRANCH_FACTOR {
            elements[i] = Some(i);
        }

        let mut tree_one = RrbTree::new();
        let mut tree_two = RrbTree::new();

        tree_two.push(elements.clone(), BRANCH_FACTOR);
        tree_one.push(elements, BRANCH_FACTOR);

        let mut iter_one = tree_one.into_iter();
        let mut iter_two = tree_two.into_iter();

        let (chunk_one, size_one) = iter_one.next().unwrap();
        let (chunk_two, size_two) = iter_two.next_back().unwrap();

        assert_eq!(size_one, BRANCH_FACTOR);
        assert_eq!(size_two, BRANCH_FACTOR);

        assert_eq!(iter_one.next(), None);
        assert_eq!(iter_two.next(), None);

        for index in 0..BRANCH_FACTOR {
            assert_eq!(index, chunk_one[index].unwrap());
            assert_eq!(index, chunk_two[index].unwrap());
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

        let mut iter_two = tree.clone().into_iter();
        let mut iter_one = tree.into_iter();

        let (chunk_one, size_one) = iter_one.next().unwrap();
        assert_eq!(size_one, BRANCH_FACTOR);

        for index in 0..BRANCH_FACTOR {
            assert_eq!(index, chunk_one[index].unwrap());
        }

        let (chunk_one, size_one) = iter_one.next().unwrap();
        assert_eq!(size_one, BRANCH_FACTOR / 2);

        for index in 0..BRANCH_FACTOR / 2 {
            assert_eq!(index, chunk_one[index].unwrap());
        }

        let (chunk_two, size_two) = iter_two.next_back().unwrap();
        assert_eq!(size_two, BRANCH_FACTOR / 2);

        for index in 0..BRANCH_FACTOR / 2 {
            assert_eq!(index, chunk_two[index].unwrap());
        }

        let (chunk_two, size_two) = iter_two.next_back().unwrap();
        assert_eq!(size_two, BRANCH_FACTOR);

        for index in 0..BRANCH_FACTOR {
            assert_eq!(index, chunk_two[index].unwrap());
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

        let mut iter_two = tree.clone().into_iter();
        let mut iter_one = tree.into_iter();

        for _ in 0..(BRANCH_FACTOR * BRANCH_FACTOR) + BRANCH_FACTOR {
            let (chunk, size) = iter_one.next().unwrap();
            assert_eq!(size, BRANCH_FACTOR);

            for index in 0..BRANCH_FACTOR {
                assert_eq!(index, chunk[index].unwrap());
            }
        }

        for _ in (0..(BRANCH_FACTOR * BRANCH_FACTOR) + BRANCH_FACTOR).rev() {
            let (chunk, size) = iter_two.next_back().unwrap();
            assert_eq!(size, BRANCH_FACTOR);

            for index in 0..BRANCH_FACTOR {
                assert_eq!(index, chunk[index].unwrap());
            }
        }
    }
}
