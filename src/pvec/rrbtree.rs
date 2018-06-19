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
const MAX_HEIGHT: usize = 8;

#[cfg(small_branch)]
const MAX_HEIGHT: usize = 32;

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
        (self.0 >> shift.0) & BRANCH_FACTOR - 1
    }

    fn element(self) -> usize {
        self.0 & BRANCH_FACTOR - 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Node<T> {
    Branch {
        children: [Option<Arc<Node<T>>>; BRANCH_FACTOR],
        len: usize,
    },
    RelaxedBranch {
        children: [Option<Arc<Node<T>>>; BRANCH_FACTOR],
        sizes: [Option<usize>; BRANCH_FACTOR],
        len: usize,
    },
    Leaf {
        elements: [Option<T>; BRANCH_FACTOR],
        len: usize,
    },
}

impl<T: Clone + Debug> Node<T> {
    fn push(&mut self, index: Index, shift: Shift, tail: [Option<T>; BRANCH_FACTOR]) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let mut node = self;
        let mut shift = shift;

        while shift.0 != BITS_PER_LEVEL {
            let cnode = node; // FIXME: NLL

            let child = match *cnode {
                Node::Leaf { .. } => unreachable!(),
                Node::RelaxedBranch { .. } => unreachable!(),
                Node::Branch { ref mut children, ref mut len } => {
                    let i = index.child(shift);

                    if children[i].is_none() {
                        *len = *len + 1;

                        children[i] = Some(Arc::new(Node::Branch {
                            children: new_branch!(),
                            len: 0,
                        }));
                    }

                    children[i].as_mut().unwrap()
                }
            };

            node = Arc::make_mut(child);
            shift = shift.dec();
        }

        debug_assert_eq!(shift.0, BITS_PER_LEVEL);

        if let Node::Branch { ref mut children, ref mut len } = *node {
            *len = *len + 1;

            children[index.child(shift)] = Some(Arc::new(
                Node::Leaf { elements: tail, len: 0 }
            ));
        }
    }

    fn pop(&mut self, index: Index, shift: Shift) -> [Option<T>; BRANCH_FACTOR] {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);
        return self.pop_recursive(index, shift).1;
    }

    fn pop_recursive(&mut self, index: Index, shift: Shift) -> (usize, [Option<T>; BRANCH_FACTOR]) {
        if let Node::Branch { ref mut children, ref mut len } = *self {
            let i = index.child(shift);

            if shift.0 == BITS_PER_LEVEL {
                *len = *len - 1;

                println!("branch factor if {}", shift.0);

                let mut leaf_node = children[i].take().unwrap();

                println!("leaf_node_references {}", Arc::strong_count(&leaf_node));

                Arc::make_mut(&mut leaf_node);

                let elements =
                    if let Node::Leaf { elements, len: _ } = Arc::try_unwrap(leaf_node).unwrap() {
                        elements
                    } else {
                        unreachable!();
                    };

                return (*len, elements);
            } else {
                println!("branch factor else {}, {}", shift.0, BRANCH_FACTOR);
                let (child_size, popped_branch) = children[i].as_mut()
                    .map(|child|
                        Arc::make_mut(child).pop_recursive(index, shift.dec()))
                    .unwrap();

                if child_size == 0 {
                    *len = *len - 1;
                    children[i] = None;
                }

                return (*len, popped_branch);
            }
        }

        println!("branch factor {}", shift.0);
        unreachable!();
    }

    fn get(&self, index: Index, shift: Shift) -> Option<&T> {
        let mut node = self;
        let mut shift = shift;

        loop {
            match *node {
                Node::Branch { ref children, len: _ } => {
                    debug_assert!(shift.0 > 0);
                    node = match children[index.child(shift)] {
                        Some(ref child) => &*child,
                        None => unreachable!()
                    };

                    shift = shift.dec();
                }
                Node::RelaxedBranch { .. } => unreachable!(),
                Node::Leaf { ref elements, ref len } => {
                    debug_assert_eq!(shift.0, 0);
                    return elements[index.element()].as_ref();
                }
            }
        }
    }

    fn get_mut(&mut self, index: Index, shift: Shift) -> Option<&mut T> {
        let mut node = self;
        let mut shift = shift;

        loop {
            let cnode = node; // FIXME: NLL

            match *cnode {
                Node::Branch { ref mut children, len: _ } => {
                    debug_assert!(shift.0 > 0);
                    node = match children[index.child(shift)] {
                        Some(ref mut child) => Arc::make_mut(child),
                        None => unreachable!()
                    };

                    shift = shift.dec();
                }
                Node::RelaxedBranch { .. } => unreachable!(),
                Node::Leaf { ref mut elements, len: _ } => {
                    debug_assert_eq!(shift.0, 0);
                    return elements[index.element()].as_mut();
                }
            }
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbTree<T> {
    root: Option<Arc<Node<T>>>,
    root_len: Index,
    shift: Shift,
}

impl<T: Clone + Debug> RrbTree<T> {
    pub fn new() -> Self {
        RrbTree {
            root: None,
            root_len: Index(0),
            shift: Shift(0),
        }
    }

    pub fn push(&mut self, tail: [Option<T>; BRANCH_FACTOR]) {
        debug!("---------------------------------------------------------------------------");
        debug!("RrbTree::push(tail={:?})", tail);

        if self.root.is_none() {
            self.root = Some(Arc::new(Node::Branch { children: new_branch!(), len: 0 }));
        }

        if let Some(root) = self.root.as_mut() {
            let capacity = BRANCH_FACTOR << self.shift.0;

            if capacity == self.root_len.0 + BRANCH_FACTOR {
                let mut nodes = new_branch!();
                nodes[0] = Some(root.clone());

                self.shift = self.shift.inc();
                *root = Arc::new(Node::Branch { children: nodes, len: 1 });
            }

            Arc::make_mut(root).push(self.root_len, self.shift, tail);
        } else {
            unreachable!()
        }

        self.root_len.0 += BRANCH_FACTOR;
    }

    pub fn pop(&mut self) -> [Option<T>; BRANCH_FACTOR] {
        debug!("---------------------------------------------------------------------------");
        debug!("RrbTree::pop() capacity={} root_len={} shift={}",
               BRANCH_FACTOR << self.shift.0, self.root_len.0, self.shift.0);

        self.root_len.0 -= BRANCH_FACTOR;

        let new_tail = if let Some(root) = self.root.as_mut() {
            Arc::make_mut(root).pop(self.root_len, self.shift)
        } else {
            unreachable!()
        };

        debug!("RrbTree::pop() -> ({:?})", new_tail);

        if self.root_len.0 == 0 {
            self.root = None;
            self.shift = self.shift.dec();

            debug!("RrbTree::lower_trie -> ()");

            return new_tail;
        }

        if let Some(root) = self.root.as_mut() {
            let capacity = BRANCH_FACTOR << self.shift.dec().0;

            debug!("RrbTree::pop() capacity={} root_len={} shift={}",
                   capacity, self.root_len.0, self.shift.0);

            if capacity == self.root_len.0 + BRANCH_FACTOR {
                self.shift = self.shift.dec();

                *root = if let Node::Branch { ref mut children, ref len } = Arc::make_mut(root) {
                    debug!("RrbTree::lower_trie -> ({:?})", children);

                    children[0].take().unwrap()
                } else {
                    unreachable!();
                };
            }
        } else {
            unreachable!()
        }

        return new_tail;
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
                        Node::Branch { children: _, ref len } => {
                            json!({
                                "branch": child,
                                "refs": refs,
                                "len": len
                            })
                        }
                        Node::RelaxedBranch { children: _, ref sizes, ref len } => {
                            json!({
                                "relaxed_branch": child,
                                "sizes": sizes,
                                "refs": refs,
                                "len": len
                            })
                        }
                        Node::Leaf { elements: _, ref len } => {
                            json!({
                                "leaf": child,
                                "refs": refs,
                                "len": len
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
                Node::Branch { ref children, len: _ } => Node::serialize_branch(children, serializer),
                Node::RelaxedBranch { ref children, sizes: _, len: _ } => Node::serialize_branch(children, serializer),
                Node::Leaf { ref elements, len: _ } => Node::serialize_leaf(elements, serializer)
            }
        }
    }

    impl<T> Serialize for RrbTree<T> where T: Serialize {
        fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error> where S: Serializer {
            let root_json_value = self.root.as_ref().map_or(None, |root| {
                let refs = Some(Arc::strong_count(root));

                let json = match root.as_ref() {
                    Node::Branch { children: _, ref len } => {
                        json!({
                            "branch": root,
                            "refs":  refs,
                            "len": len
                        })
                    }
                    Node::RelaxedBranch { children: _, ref sizes, ref len } => {
                        json!({
                            "relaxed_branch": root,
                            "sizes": sizes,
                            "refs":  refs,
                            "len": len
                        })
                    }
                    Node::Leaf { elements: _, ref len } => {
                        json!({
                            "leaf": root,
                            "refs":  refs,
                            "len": len
                        })
                    }
                };

                Some(json)
            });

            let mut serde_state = serializer.serialize_struct("RrbTree", 1)?;
            serde_state.serialize_field("root_len", &self.root_len.0)?;
            serde_state.serialize_field("shift", &self.shift.0)?;
            serde_state.serialize_field("root", &root_json_value)?;
            serde_state.end()
        }
    }

    #[test]
    fn hello_world() {
        let mut values = new_branch!();
        for i in 0..BRANCH_FACTOR {
            values[i] = Some(i);
        }

        let mut tree = RrbTree::new();
        println!("tree = {}", serde_json::to_string(&tree).unwrap());

        tree.push(values);

        println!("tree = {}", serde_json::to_string(&tree).unwrap());
    }
}
