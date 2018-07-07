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
        [None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,]
    }
}

#[cfg(small_branch)]
macro_rules! new_branch {
    () => {
        [None, None, None, None]
    }
}

macro_rules! debug {
    ($($t:tt)*) => {
         // println!($($t)*);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Shift(usize);

impl Shift {
    fn inc(self) -> Shift {
        Shift(self.0 + BITS_PER_LEVEL)
    }

    fn dec(self) -> Shift {
        Shift(self.0 - BITS_PER_LEVEL)
    }

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
    fn child(self, shift: Shift) -> usize {
        (self.0 >> shift.0) & (BRANCH_FACTOR - 1)
    }

    fn element(self) -> usize {
        self.0 & (BRANCH_FACTOR - 1)
    }
}

// ToDo: Arc<> contents of the node instead of Box<>

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Branch<T> {
    children: [Option<Arc<Node<T>>>; BRANCH_FACTOR],
    len: usize,
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Leaf<T> {
    elements: [Option<T>; BRANCH_FACTOR],
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Node<T> {
    Branch(Box<Branch<T>>),
    Leaf(Box<Leaf<T>>),
}

impl<T: Clone + Debug> Node<T> {
    fn push(&mut self, index: Index, shift: Shift, tail: [Option<T>; BRANCH_FACTOR], tail_len: usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let mut node = self;
        let mut shift = shift;

        while shift.0 > BITS_PER_LEVEL {
            node = match *node {
                Node::Leaf(..) => unreachable!(),
                Node::Branch(ref mut branch) => {
                    let i = index.child(shift);

                    if branch.children[i].is_none() {
                        branch.len += 1;

                        branch.children[i] = Some(Arc::new(Node::Branch(
                            Box::new(Branch {
                                children: new_branch!(),
                                len: 0,
                            })
                        )));
                    }

                    Arc::make_mut(branch.children[i].as_mut().unwrap())
                }
            };

            shift = shift.dec();
        }

        debug_assert_eq!(shift.0, BITS_PER_LEVEL);

        if let Node::Branch(ref mut branch) = *node {
            branch.len += 1;
            branch.children[index.child(shift)] = Some(Arc::new(
                Node::Leaf(Box::new(Leaf { elements: tail, len: tail_len }))
            ));
        }
    }

    fn pop(&mut self, index: Index, shift: Shift) -> ([Option<T>; BRANCH_FACTOR], usize) {
        let (
            tail,
            tail_len,
            _child_len
        ) = self.remove(index, shift);

        (tail, tail_len)
    }

    fn remove(&mut self, index: Index, shift: Shift) -> ([Option<T>; BRANCH_FACTOR], usize, usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        if let Node::Branch(ref mut branch) = *self {
            let i = index.child(shift);

            if shift.0 == BITS_PER_LEVEL {
                branch.len -= 1;

                let mut leaf_node = branch.children[i].take().unwrap();
                Arc::make_mut(&mut leaf_node);

                let (elements, elements_len) =
                    if let Node::Leaf(leaf) = Arc::try_unwrap(leaf_node).unwrap() {
                        (leaf.elements, leaf.len)
                    } else {
                        unreachable!();
                    };

                return (elements, elements_len, branch.len);
            } else {
                let (tail, tail_len, child_len) = branch.children[i].as_mut()
                    .map(|child|
                        Arc::make_mut(child).remove(index, shift.dec()))
                    .unwrap();

                if child_len == 0 {
                    branch.len -= 1;
                    branch.children[i] = None;
                }

                return (tail, tail_len, branch.len);
            }
        }

        unreachable!();
    }

    fn get(&self, index: Index, shift: Shift) -> Option<&T> {
        let mut node = self;
        let mut shift = shift;

        loop {
            match *node {
                Node::Branch(ref branch) => {
                    debug_assert!(shift.0 > 0);
                    node = match branch.children[index.child(shift)] {
                        Some(ref child) => &*child,
                        None => unreachable!()
                    };

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
                Node::Branch(ref mut branch) => {
                    debug_assert!(shift.0 > 0);

                    node = match branch.children[index.child(shift)] {
                        Some(ref mut child) => Arc::make_mut(child),
                        None => unreachable!()
                    };

                    shift = shift.dec();
                }
                Node::Leaf(ref mut leaf) => {
                    debug_assert_eq!(shift.0, 0);
                    return leaf.elements[index.element()].as_mut();
                }
            }
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbTree<T> {
    root: Option<Arc<Node<T>>>,
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

        if self.root.is_none() {
            self.root = Some(Arc::new(Node::Branch(
                Box::new(
                    Branch { children: new_branch!(), len: 0 }
                ))
            ));
            self.shift = self.shift.inc();
        }

        let root = self.root.as_mut().unwrap();
        Arc::make_mut(root).push(self.root_len_max, self.shift, tail, tail_len);

        if self.shift.capacity() == self.root_len_max.0 + BRANCH_FACTOR {
            debug!("RrbTree::push() - growing tree; capacity={}", self.shift.capacity());

            let mut nodes = new_branch!();
            nodes[0] = Some(root.clone());

            self.shift = self.shift.inc();
            *root = Arc::new(Node::Branch(
                Box::new(
                    Branch { children: nodes, len: 1 }
                )
            ));
        }

        self.root_len.0 += tail_len;
        self.root_len_max.0 += BRANCH_FACTOR;
    }

    pub fn pop(&mut self) -> [Option<T>; BRANCH_FACTOR] {
        debug!("---------------------------------------------------------------------------");
        debug!("RrbTree::pop() capacity={} root_len_max={} shift={}",
               self.shift.capacity(), self.root_len_max.0, self.shift.0);

        self.root_len_max.0 -= BRANCH_FACTOR;

        let (new_tail, new_tail_len) = Arc::make_mut(
            self.root.as_mut().unwrap()
        ).pop(self.root_len_max, self.shift);

        self.root_len.0 -= new_tail_len;

        debug!("RrbTree::pop() -> ({:?})", new_tail);
        debug!("RrbTree::pop() -> len ({:?})", new_tail_len);

        if self.root_len_max.0 == 0 {
            self.root = None;
            self.shift = self.shift.dec();

            debug!("RrbTree::lower_trie -> ()");

            return new_tail;
        }

        let root = self.root.as_mut().unwrap();
//
//        debug!("RrbTree::pop() - 2 capacity={} root_len_max={} shift={}",
//               self.shift.dec().capacity(), self.root_len_max.0, self.shift.0);

        debug!("RrbTree::pop() -> self.shift.dec().capacity()={} self.root_len_max + BRANCH_FACTOR={} shift={}",
               self.shift.dec().capacity(), self.root_len_max.0 + BRANCH_FACTOR, self.shift.0);

        if self.shift.dec().capacity() == self.root_len_max.0 + BRANCH_FACTOR {
            self.shift = self.shift.dec();

            debug!("RrbTree::pop() -> trying to lower the tree");

            *root = if let Node::Branch(ref mut branch) = Arc::make_mut(root) {
                debug!("RrbTree::lower_trie -> ({:?})", branch.children);

                branch.children[0].take().unwrap()
            } else {
                unreachable!();
            };
        }

        new_tail
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.root.as_ref().unwrap().get(Index(index), self.shift)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        Arc::make_mut(self.root.as_mut().unwrap()).get_mut(Index(index), self.shift)
    }

    pub fn len(&self) -> usize {
        self.root_len.0
    }
}

#[cfg(test)]
mod tests {
    extern crate serde;
    extern crate serde_json;

    use self::serde::ser::{Serialize, Serializer, SerializeSeq, SerializeStruct};
    use std::sync::Arc;
    use super::{Node, RrbTree};
    use super::BRANCH_FACTOR;

    impl<T> Node<T> where T: Serialize {
        fn serialize_branch<S>(children: &[Option<Arc<Node<T>>>; BRANCH_FACTOR], serializer: S) -> Result<<S>::Ok, <S>::Error> where S: Serializer {
            let mut children_refs = Vec::with_capacity(BRANCH_FACTOR);

            for i in 0..BRANCH_FACTOR {
                if let Some(child) = children[i].as_ref() {
                    let refs = Arc::strong_count(child);

                    let child_json_value = match child.as_ref() {
                        Node::Branch(ref branch) => {
                            json!({
                                "branch": child,
                                "refs": refs,
                                "len": branch.len
                            })
                        }
                        Node::Leaf(ref leaf) => {
                            json!({
                                "leaf": child,
                                "refs": refs,
                                "len": leaf.len
                            })
                        }
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

        fn serialize_leaf<S>(elements: &[Option<T>; BRANCH_FACTOR], serializer: S) -> Result<<S>::Ok, <S>::Error> where S: Serializer {
            let mut serde_state = serializer.serialize_seq(Some(BRANCH_FACTOR))?;

            for element in elements {
                serde_state.serialize_element(&element)?;
            }

            return serde_state.end();
        }
    }

    impl<T> Serialize for Node<T> where T: Serialize {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error> where S: Serializer {
            match *self {
                Node::Branch(ref branch) => Node::serialize_branch(&branch.children, serializer),
                Node::Leaf(ref leaf) => Node::serialize_leaf(&leaf.elements, serializer)
            }
        }
    }

    impl<T> Serialize for RrbTree<T> where T: Serialize {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error> where S: Serializer {
            let root_json_value = self.root.as_ref().map_or(None, |root| {
                let refs = Some(Arc::strong_count(root));

                let json = match root.as_ref() {
                    Node::Branch(ref branch) => {
                        json!({
                            "branch": root,
                            "refs":  refs,
                            "len": branch.len
                        })
                    }
                    Node::Leaf(ref leaf) => {
                        json!({
                            "leaf": root,
                            "refs": refs,
                            "len": leaf.len
                        })
                    }
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
    #[ignore]
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
}

