use std::cmp::Ordering;
use std::fmt::Debug;
use std::sync::Arc;

#[cfg(not(small_branch))]
pub const BRANCH_FACTOR: usize = 32;

#[cfg(small_branch)]
pub const BRANCH_FACTOR: usize = 4;

#[cfg(not(small_branch))]
const BITS_PER_LEVEL: usize = 5;

#[cfg(small_branch)]
const BITS_PER_LEVEL: usize = 2;

#[cfg(not(small_branch))]
macro_rules! new_branch {
    () => {
        [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None,
        ]
    };
}

#[cfg(small_branch)]
macro_rules! new_branch {
    () => {
        [None, None, None, None]
    };
}

macro_rules! debug {
    ($($t:tt)*) => {
        // println!($($t)*);
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Shift(usize);

impl Shift {
    #[inline(always)]
    fn inc(self) -> Shift {
        Shift(self.0 + BITS_PER_LEVEL)
    }

    #[inline(always)]
    fn dec(self) -> Shift {
        Shift(self.0 - BITS_PER_LEVEL)
    }

    #[inline(always)]
    fn capacity(self) -> usize {
        BRANCH_FACTOR << self.0
    }
}

impl PartialEq<usize> for Shift {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<usize> for Shift {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

impl PartialEq<usize> for Index {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<usize> for Index {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Index {
    #[inline(always)]
    fn child(self, shift: Shift) -> usize {
        (self.0 >> shift.0) & (BRANCH_FACTOR - 1)
    }

    #[inline(always)]
    fn element(self) -> usize {
        self.0 & (BRANCH_FACTOR - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RelaxedBranch<T> {
    children: [Option<Node<T>>; BRANCH_FACTOR],
    sizes: [Option<usize>; BRANCH_FACTOR],
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Branch<T> {
    children: [Option<Node<T>>; BRANCH_FACTOR],
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Leaf<T> {
    elements: [Option<T>; BRANCH_FACTOR],
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Node<T> {
    RelaxedBranch(Arc<RelaxedBranch<T>>),
    Branch(Arc<Branch<T>>),
    Leaf(Arc<Leaf<T>>),
}

impl<T: Clone + Debug> Leaf<T> {
    fn merge(mut self, mut that: Leaf<T>) -> RelaxedBranch<T> {
        let mut index_l = self.len;
        let mut index_r = 0;

        let len_r = that.len;

        while index_l < BRANCH_FACTOR && index_r < len_r {
            self.elements[index_l] = that.elements[index_r].take();

            index_l += 1;
            index_r += 1;

            self.len += 1;
            that.len -= 1;
        }

        for i in 0..that.len {
            that.elements[i] = that.elements[index_r].take();
            index_r += 1;
        }

        let mut branch_children = new_branch!();
        let mut branch_sizes = new_branch!();

        branch_sizes[0] = Some(self.len);
        branch_sizes[1] = Some(that.len);

        branch_children[0] = Some(Node::Leaf(Arc::new(self)));
        branch_children[1] = Some(Node::Leaf(Arc::new(that)));

        RelaxedBranch {
            children: branch_children,
            sizes: branch_sizes,
            len: 2,
        }
    }
}

impl<T: Clone + Debug> Node<T> {
    #[inline(always)]
    fn as_mut_branch(&mut self) -> &mut Branch<T> {
        match self {
            Node::Leaf(..) => unreachable!(),
            Node::RelaxedBranch(..) => unreachable!(),
            Node::Branch(ref mut branch) => Arc::make_mut(branch),
        }
    }

    #[inline(always)]
    fn into_leaf(self) -> Leaf<T> {
        match self {
            Node::Leaf(mut leaf_arc) => {
                Arc::make_mut(&mut leaf_arc);
                Arc::try_unwrap(leaf_arc).unwrap()
            }
            Node::RelaxedBranch(..) => unreachable!(),
            Node::Branch(..) => unreachable!(),
        }
    }
}

impl<T: Clone + Debug> Node<T> {
    fn push(&mut self, index: Index, shift: Shift, leaf: Leaf<T>) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let mut node = self;
        let mut shift = shift;

        while shift.0 > BITS_PER_LEVEL {
            let i = index.child(shift);

            let branch = node.as_mut_branch();
            let child = &mut branch.children[i];
            let len = &mut branch.len;

            node = child.get_or_insert_with(|| {
                *len += 1;

                Node::Branch(Arc::new(Branch {
                    children: new_branch!(),
                    len: 0,
                }))
            });

            shift = shift.dec();
        }

        debug_assert_eq!(shift.0, BITS_PER_LEVEL);

        let branch = node.as_mut_branch();

        branch.len += 1;
        branch.children[index.child(shift)] = Some(Node::Leaf(Arc::new(leaf)));
    }

    fn pop(&mut self, index: Index, shift: Shift) -> Leaf<T> {
        self.remove(index, shift).0
    }

    fn remove(&mut self, index: Index, shift: Shift) -> (Leaf<T>, usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let branch = self.as_mut_branch();
        let i = index.child(shift);

        if shift.0 == BITS_PER_LEVEL {
            branch.len -= 1;

            let leaf_node = branch.children[i].take().unwrap();
            let leaf = leaf_node.into_leaf();

            (leaf, branch.len)
        } else {
            let (leaf, child_len) = branch.children[i]
                .as_mut()
                .map(|child| child.remove(index, shift.dec()))
                .unwrap();

            if child_len == 0 {
                branch.len -= 1;
                branch.children[i] = None;
            }

            (leaf, branch.len)
        }
    }

    fn get(&self, index: Index, shift: Shift) -> Option<&T> {
        let mut node = self;
        let mut shift = shift;

        loop {
            match *node {
                Node::RelaxedBranch(..) => unreachable!(),
                Node::Branch(ref branch) => {
                    debug_assert!(shift.0 > 0);

                    node = branch.children[index.child(shift)].as_ref().unwrap();
                    shift = shift.dec();
                }
                Node::Leaf(ref leaf) => {
                    debug_assert_eq!(shift.0, 0);

                    return leaf.elements[index.element()].as_ref();
                }
            }
        }
    }

    fn get_mut(&mut self, index: Index, shift: Shift) -> Option<&mut T> {
        let mut node = self;
        let mut shift = shift;

        loop {
            match *node {
                Node::RelaxedBranch(..) => unreachable!(),
                Node::Branch(ref mut branch_arc) => {
                    debug_assert!(shift.0 > 0);

                    let branch = Arc::make_mut(branch_arc);

                    node = branch.children[index.child(shift)].as_mut().unwrap();
                    shift = shift.dec();
                }
                Node::Leaf(ref mut leaf_arc) => {
                    debug_assert_eq!(shift.0, 0);

                    let leaf = Arc::make_mut(leaf_arc);
                    return leaf.elements[index.element()].as_mut();
                }
            }
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbTree<T> {
    root: Option<Node<T>>,
    root_len: Index,
    root_len_max: Index,
    shift: Shift,
}

impl<T: Clone + Debug> RrbTree<T> {
    pub fn new() -> Self {
        RrbTree {
            root: None,
            root_len: Index(0),
            root_len_max: Index(0),
            shift: Shift(0),
        }
    }

    #[cold]
    pub fn push(&mut self, tail: [Option<T>; BRANCH_FACTOR], tail_len: usize) {
        debug!("---------------------------------------------------------------------------");
        debug!("RrbTree::push(tail={:?})", tail);

        if let Some(ref mut root) = self.root {
            if self.shift.capacity() == self.root_len_max.0 {
                debug!(
                    "RrbTree::push() - growing tree; capacity={}",
                    self.shift.capacity()
                );

                let mut nodes = new_branch!();
                nodes[0] = Some(root.clone());

                self.shift = self.shift.inc();
                *root = Node::Branch(Arc::new(Branch {
                    children: nodes,
                    len: 1,
                }));
            }

            root.push(
                self.root_len_max,
                self.shift,
                Leaf {
                    elements: tail,
                    len: tail_len,
                },
            );
        } else {
            self.root = Some(Node::Leaf(Arc::new(Leaf {
                elements: tail,
                len: tail_len,
            })));
        }

        self.root_len.0 += tail_len;
        self.root_len_max.0 += BRANCH_FACTOR;
    }

    pub fn pop(&mut self) -> [Option<T>; BRANCH_FACTOR] {
        debug!("---------------------------------------------------------------------------");
        debug!(
            "RrbTree::pop() capacity={} root_len_max={} shift={}",
            self.shift.capacity(),
            self.root_len_max.0,
            self.shift.0
        );

        self.root_len_max.0 -= BRANCH_FACTOR;

        let leaf = self
            .root
            .as_mut()
            .unwrap()
            .pop(self.root_len_max, self.shift);

        self.root_len.0 -= leaf.len;

        debug!("RrbTree::pop() -> ({:?})", leaf.elements);
        debug!("RrbTree::pop() -> len ({:?})", leaf.len);

        if self.root_len_max.0 == 0 {
            self.root = None;
            self.shift = self.shift.dec();

            debug!("RrbTree::lower_trie -> ()");

            return leaf.elements;
        }

        let root = self.root.as_mut().unwrap();

        debug!("RrbTree::pop() -> self.shift.dec().capacity()={} self.root_len_max + BRANCH_FACTOR={} shift={}",
               self.shift.dec().capacity(), self.root_len_max.0 + BRANCH_FACTOR, self.shift.0);

        if self.shift.dec().capacity() == self.root_len_max.0 + BRANCH_FACTOR {
            self.shift = self.shift.dec();

            debug!("RrbTree::pop() -> trying to lower the tree");

            let branch = root.as_mut_branch();
            *root = branch.children[0].take().unwrap();
        }

        leaf.elements
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.root.as_ref().unwrap().get(Index(index), self.shift)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.root
            .as_mut()
            .unwrap()
            .get_mut(Index(index), self.shift)
    }

    pub fn len(&self) -> usize {
        self.root_len.0
    }
}

#[cfg(test)]
mod tests {
    extern crate serde;
    extern crate serde_json;

    use self::serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};
    use super::BRANCH_FACTOR;
    use super::{Branch, Leaf, Node, RrbTree};
    use std::sync::Arc;

    impl<T> Branch<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
        where
            S: Serializer,
        {
            let mut children_refs = Vec::with_capacity(BRANCH_FACTOR);

            for i in 0..BRANCH_FACTOR {
                if let Some(child) = self.children[i].as_ref() {
                    let child_json_value = match child {
                        Node::RelaxedBranch(..) => unreachable!(),
                        Node::Branch(ref branch) => json!({
                                "branch": child,
                                "refs": Arc::strong_count(branch),
                                "len": branch.len
                            }),
                        Node::Leaf(ref leaf) => json!({
                                "leaf": child,
                                "refs": Arc::strong_count(leaf),
                                "len": leaf.len
                            }),
                    };

                    children_refs.push(child_json_value);
                }
            }

            let mut serde_state = serializer.serialize_seq(Some(BRANCH_FACTOR))?;

            for child in children_refs {
                serde_state.serialize_element(&child)?;
            }

            return serde_state.end();
        }
    }

    impl<T> Leaf<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
        where
            S: Serializer,
        {
            let mut serde_state = serializer.serialize_seq(Some(BRANCH_FACTOR))?;

            for element in self.elements.iter() {
                serde_state.serialize_element(&element)?;
            }

            return serde_state.end();
        }
    }

    impl<T> Serialize for Node<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
        where
            S: Serializer,
        {
            match *self {
                Node::RelaxedBranch(..) => unreachable!(),
                Node::Branch(ref branch) => branch.serialize(serializer),
                Node::Leaf(ref leaf) => leaf.serialize(serializer),
            }
        }
    }

    impl<T> Serialize for RrbTree<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
        where
            S: Serializer,
        {
            let root_json_value = self.root.as_ref().map_or(None, |root| {
                let json = match root {
                    Node::RelaxedBranch(..) => unreachable!(),
                    Node::Branch(ref branch) => json!({
                            "branch": root,
                            "refs":  Arc::strong_count(branch),
                            "len": branch.len
                        }),
                    Node::Leaf(ref leaf) => json!({
                            "leaf": root,
                            "refs": Arc::strong_count(leaf),
                            "len": leaf.len
                        }),
                };

                Some(json)
            });

            let mut serde_state = serializer.serialize_struct("RrbTree", 1)?;
            serde_state.serialize_field("root_len", &self.root_len.0)?;
            serde_state.serialize_field("root_len_max", &self.root_len_max.0)?;
            serde_state.serialize_field("shift", &self.shift.0)?;
            serde_state.serialize_field("root", &root_json_value)?;
            serde_state.end()
        }
    }

    #[test]
    fn serialized_state_should_match_to_valid_rb_tree_after_clone() {
        let mut tree_1 = RrbTree::new();
        let mut value = 1;

        for _i in 0..(BRANCH_FACTOR * BRANCH_FACTOR - 2) {
            let mut values = new_branch!();

            for j in 0..BRANCH_FACTOR {
                values[j] = Some(value);
                value = value + 1;
            }

            tree_1.push(values, BRANCH_FACTOR);
        }

        let mut tree_2 = tree_1.clone();
        let mut values_2 = new_branch!();

        for j in 0..BRANCH_FACTOR {
            values_2[j] = Some(value);
            value = value + 1;
        }

        tree_2.push(values_2, BRANCH_FACTOR);

        let mut tree_3 = tree_2.clone();
        let mut values_3 = new_branch!();

        for j in 0..BRANCH_FACTOR {
            values_3[j] = Some(value);
            value = value + 1;
        }

        tree_3.push(values_3, BRANCH_FACTOR);

        debug!("{}", serde_json::to_string(&tree_1).unwrap());
        debug!("{}", serde_json::to_string(&tree_2).unwrap());
        debug!("{}", serde_json::to_string(&tree_3).unwrap());
    }

    #[test]
    fn serialized_state_should_match_to_valid_rb_tree() {
        let mut tree = RrbTree::new();

        let mut value = 1;

        for _i in 0..(BRANCH_FACTOR * BRANCH_FACTOR) {
            let mut values = new_branch!();

            for j in 0..(BRANCH_FACTOR) {
                values[j] = Some(value);
                value = value + 1;
            }

            tree.push(values, BRANCH_FACTOR);
        }

        debug!("{}", serde_json::to_string(&tree).unwrap());

        for _i in 0..(BRANCH_FACTOR * BRANCH_FACTOR / 2 + 5) {
            tree.pop();
        }

        debug!("{}", serde_json::to_string(&tree).unwrap());
    }

    #[test]
    fn merge_leaves_must_return_balanced_result() {
        let mut elements_one: [Option<usize>; BRANCH_FACTOR] = new_branch!();
        let mut elements_two: [Option<usize>; BRANCH_FACTOR] = new_branch!();

        for i in 0..BRANCH_FACTOR / 2 {
            elements_one[i] = Some(i);
        }

        for i in 0..BRANCH_FACTOR {
            elements_two[i] = Some(BRANCH_FACTOR / 2 + i);
        }

        let leaf_l = Leaf {
            elements: elements_one,
            len: BRANCH_FACTOR / 2,
        };
        let leaf_r = Leaf {
            elements: elements_two,
            len: BRANCH_FACTOR,
        };

        println!("{:?}", leaf_l);
        println!("{:?}", leaf_r);

        let branch = leaf_l.merge(leaf_r);

        let leaf_l = branch.children[0].as_ref().unwrap();
        let leaf_r = branch.children[1].as_ref().unwrap();

        if let (Node::Leaf(ref this), Node::Leaf(ref that)) = (leaf_l, leaf_r) {
            assert_eq!(this.len, BRANCH_FACTOR);
            assert_eq!(that.len, BRANCH_FACTOR / 2);

            for i in 0..BRANCH_FACTOR {
                assert_eq!(this.elements[i].unwrap(), i);
            }

            for i in 0..BRANCH_FACTOR / 2 {
                assert_eq!(that.elements[i].unwrap(), BRANCH_FACTOR + i);
            }

            for i in BRANCH_FACTOR / 2..BRANCH_FACTOR {
                assert_eq!(that.elements[i], None);
            }
        }

        for i in 2..BRANCH_FACTOR {
            assert_eq!(branch.children[i], None);
        }

        println!("{:?}", branch);
    }
}
