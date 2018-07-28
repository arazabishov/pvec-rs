use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter::Filter;
use std::mem;
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

    #[inline(always)]
    fn is_leaf_level(self) -> bool {
        self.0 == BITS_PER_LEVEL
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
    #[inline(always)]
    fn new() -> Self {
        Leaf { elements: new_branch!(), len: 0 }
    }

    #[inline(always)]
    fn add(&mut self, element: Option<T>) {
        self.elements[self.len] = element;
        self.len += 1;
    }

    #[inline(always)]
    fn push(&mut self, element: T) {
        self.elements[self.len] = Some(element);
        self.len += 1;
    }

    #[inline(always)]
    fn take(&mut self, i: usize) -> Option<T> {
        self.len -= 1;
        self.elements[i].take()
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        self.len == BRANCH_FACTOR
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    fn merge(&mut self, mut that: Leaf<T>) -> Node<T> {
        let mut leaf_l = Leaf::new();
        let mut leaf_r = Leaf::new();

        for i in 0..self.len {
            leaf_l.add(self.take(i));
        }

        let mut index_l = leaf_l.len;
        let mut index_r = 0;

        while index_l < BRANCH_FACTOR && index_r < that.len {
            leaf_l.add(that.take(index_r));

            index_l += 1;
            index_r += 1;
        }

        for i in 0..that.len {
            leaf_r.add(that.take(index_r));

            index_r += 1;
        }

        let mut children = new_branch!();

        children[0] = Some(Node::Leaf(Arc::new(leaf_l)));
        children[1] = Some(Node::Leaf(Arc::new(leaf_r)));

        Node::Branch(Arc::new(
            Branch { children, len: 2 }
        ))
    }

    #[inline(always)]
    fn rebalance(merged: Vec<Option<Node<T>>>) -> Node<T> {
        let mut new_root = Branch::new();
        let mut new_subtree = Branch::new();
        let mut new_leaf = Leaf::new();

        for mut subtree in merged {
            println!("{:?}", subtree);

            let mut old_leaf = subtree.take().unwrap().into_leaf();

            if new_leaf.is_empty() && old_leaf.is_full() {
                if new_subtree.is_full() {
                    new_root.push(Node::Branch(Arc::new(new_subtree)));
                    new_subtree = Branch::new();
                }

                new_subtree.push(Node::Leaf(old_leaf));
            } else {
                for i in 0..old_leaf.len {
                    if new_leaf.is_full() {
                        if new_subtree.is_full() {
                            new_root.push(Node::Branch(Arc::new(new_subtree)));
                            new_subtree = Branch::new();
                        }

                        new_subtree.push(Node::Leaf(Arc::new(new_leaf)));
                        new_leaf = Leaf::new();
                    }

                    new_leaf.add(
                        Arc::make_mut(&mut old_leaf).take(i).take()
                    );
                }
            }
        }

        if new_subtree.is_full() {
            new_root.push(Node::Branch(Arc::new(new_subtree)));
            new_subtree = Branch::new();
        }

        new_subtree.push(Node::Leaf(Arc::new(new_leaf)));
        new_root.push(Node::Branch(Arc::new(new_subtree)));

        println!("new_root={:?}", new_root);
        Node::Branch(Arc::new(new_root))
    }
}

impl<T: Clone + Debug> Branch<T> {
    #[inline(always)]
    fn new() -> Self {
        Branch { children: new_branch!(), len: 0 }
    }

    #[inline(always)]
    fn push(&mut self, child: Node<T>) {
        self.children[self.len] = Some(child);
        self.len += 1;
    }

    #[inline(always)]
    fn take(&mut self, i: usize) -> Option<Node<T>> {
        self.len -= 1;
        self.children[i].take()
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        self.len == BRANCH_FACTOR
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

trait Take<T: Clone + Debug> {
    fn take(self) -> T;
}

impl<T: Clone + Debug> Take<T> for Arc<T> {
    fn take(mut self) -> T {
        // ToDo: is this thread safe? Probably not because it is not atomic
        // ToDo: investigate how to reason about thread safety.
        Arc::make_mut(&mut self);
        Arc::try_unwrap(self).unwrap()
    }
}

impl<T: Clone + Debug> Node<T> {
    #[inline(always)]
    fn as_mut_children(&mut self) -> &mut [Option<Node<T>>] {
        match self {
            Node::Branch(ref mut node) => {
                let mut branch = Arc::make_mut(node);
                &mut branch.children[..branch.len]
            }
            Node::RelaxedBranch(ref mut node) => {
                let mut branch = Arc::make_mut(node);
                &mut branch.children[..branch.len]
            }
            Node::Leaf(..) => unreachable!(),
        }
    }

    // ToDo: make sure that you keep len properties of tree in sync
    // ToDo: reconsider getting rid of outer if-else block in this function
    fn merge(&mut self, mut that: Node<T>, self_shift: Shift, that_shift: Shift) -> Node<T> {
        if self_shift > that_shift {
            let branch_l = self.as_mut_branch_internals();
            let (mut init, mut child_l) =
                branch_l.children.split_at_mut(branch_l.len - 1);

            let mut child_node_l = child_l[0].as_mut().unwrap();

            // ToDo: avoid unnecessary allocations
            let mut branch_c = child_node_l.merge(that, self_shift.dec(), that_shift);
            Node::rebalance(
                init, branch_c.as_mut_children(), &mut [], self_shift,
            )
        } else if that_shift < self_shift {
            let branch_r = that.as_mut_branch_internals();

            let (mut child_r, mut tail) =
                branch_r.children.split_first_mut().unwrap();
            let mut child_node_r = child_r.take().unwrap();

            // ToDo: avoid unnecessary allocations
            let mut branch_c = self.merge(child_node_r, self_shift, that_shift.dec());
            Node::rebalance(
                &mut [], branch_c.as_mut_children(), tail, that_shift,
            )
        } else {
            // ToDo: take care of decending correctly
            if self_shift.0 == 0 {
                Arc::make_mut(self.as_mut_leaf())
                    .merge(that.into_leaf().take())
            } else {
                let mut branch_l = self.as_mut_branch_internals();
                let mut branch_r = that.as_mut_branch_internals();

                // ToDo: you can underflow here in case when len is zero
                let (mut init, mut child_l) =
                    branch_l.children.split_at_mut(branch_l.len - 1);

                let (mut child_r, mut tail) =
                    branch_r.children.split_first_mut().unwrap();

                // ToDo: need to be more careful here with unwrapping (might be None after-all)
                let mut child_node_l = child_l[0].as_mut().unwrap();
                let mut child_node_r = child_r.take().unwrap();

                let mut branch_c = if self_shift.0 == BITS_PER_LEVEL {
                    Arc::make_mut(child_node_l.as_mut_leaf()).merge(child_node_r.into_leaf().take())
                } else {
                    child_node_l.merge(child_node_r, self_shift.dec(), that_shift.dec())
                };

                Node::rebalance(
                    init, branch_c.as_mut_children(), tail, self_shift,
                )
            }
        }
    }

    #[inline(always)]
    fn merge_all(
        node_l: &mut [Option<Node<T>>],
        node_c: &mut [Option<Node<T>>],
        node_r: &mut [Option<Node<T>>],
    ) -> Vec<Option<Node<T>>> {
        println!("merge_all: node_l={:?}", node_l);
        println!("merge_all: node_c={:?}", node_c);
        println!("merge_all: node_r={:?}", node_r);

        let mut merged = Vec::with_capacity(
            node_l.len() + node_c.len() + node_r.len()
        );

        let chained = node_l.iter_mut()
            .chain(node_c.iter_mut())
            .chain(node_r.iter_mut())
            .map(|it| it.take());

        for item in chained {
            merged.push(item);
        }

        return merged;
    }

    // ToDo: consider getting rid of the RelaxedBranch variant in
    // ToDo: favor of Branch with array (check the overall size of enum)

    // ToDo: computing sizes
    // ToDo: creating relaxed nodes where necessary
    // ToDo: eliminating boilerplate
    fn rebalance(
        node_l: &mut [Option<Node<T>>],
        node_c: &mut [Option<Node<T>>],
        node_r: &mut [Option<Node<T>>],
        shift: Shift,
    ) -> Node<T> {
        // ToDo: add optimisation for skipping balanced nodes
        // ToDo: compute sizes
        let merged = Node::merge_all(
            node_l, node_c, node_r,
        );

        if shift.is_leaf_level() {
            Leaf::rebalance(merged)
        } else {
            let mut new_root = new_branch!();
            let mut new_root_i = 0;

            let mut new_node = new_branch!();
            let mut new_node_i = 0;

            let mut new_subtree = new_branch!();
            let mut new_subtree_i = 0;

            for mut subtree in merged {
                println!("{:?}", subtree);

                let mut branch = subtree.as_mut().unwrap().as_mut_branch();

                for i in 0..branch.len {
                    if new_node_i == BRANCH_FACTOR {
                        if new_subtree_i == BRANCH_FACTOR {
                            new_root[new_root_i] = Some(Node::Branch(Arc::new(Branch {
                                children: new_subtree,
                                len: new_subtree_i,
                            })));
                            new_root_i += 1;

                            new_subtree = new_branch!();
                            new_subtree_i = 0;
                        }

                        new_subtree[new_subtree_i] = Some(Node::Branch(Arc::new(Branch {
                            children: new_node,
                            len: new_node_i,
                        })));
                        new_subtree_i += 1;

                        new_node = new_branch!();
                        new_node_i = 0;
                    }

                    new_node[new_node_i] = Arc::make_mut(branch).take(i).take();
                    new_node_i += 1;
                }
            }

            if new_subtree_i == BRANCH_FACTOR {
                new_root[new_root_i] = Some(Node::Branch(Arc::new(Branch {
                    children: new_subtree,
                    len: new_subtree_i,
                })));
                new_root_i += 1;

                new_subtree = new_branch!();
                new_subtree_i = 0;
            }

            new_subtree[new_subtree_i] = Some(Node::Branch(Arc::new(Branch {
                children: new_node,
                len: new_node_i,
            })));
            new_subtree_i += 1;

            new_root[new_root_i] = Some(Node::Branch(Arc::new(Branch {
                children: new_subtree,
                len: new_subtree_i,
            })));
            new_root_i += 1;

            println!("new_root={:?}", new_root);
            Node::Branch(Arc::new(Branch { children: new_root, len: new_root_i }))
        }
    }
}

impl<T: Clone + Debug> Node<T> {
    #[inline(always)]
    fn as_mut_branch_internals(&mut self) -> &mut Branch<T> {
        if let Node::Branch(ref mut branch) = self {
            Arc::make_mut(branch)
        } else {
            unreachable!()
        }
    }

    #[inline(always)]
    fn as_mut_branch(&mut self) -> &mut Arc<Branch<T>> {
        if let Node::Branch(ref mut branch) = self {
            branch
        } else {
            unreachable!()
        }
    }

    #[inline(always)]
    fn as_mut_leaf(&mut self) -> &mut Arc<Leaf<T>> {
        if let Node::Leaf(ref mut leaf_arc) = self {
            leaf_arc
        } else {
            unreachable!();
        }
    }

    #[inline(always)]
    fn into_leaf(self) -> Arc<Leaf<T>> {
        if let Node::Leaf(mut leaf_arc) = self {
            leaf_arc
        } else {
            unreachable!()
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

            let branch = node.as_mut_branch_internals();
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

        let branch = node.as_mut_branch_internals();

        branch.len += 1;
        branch.children[index.child(shift)] = Some(Node::Leaf(Arc::new(leaf)));
    }

    fn pop(&mut self, index: Index, shift: Shift) -> Leaf<T> {
        self.remove(index, shift).0
    }

    fn remove(&mut self, index: Index, shift: Shift) -> (Leaf<T>, usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let branch = self.as_mut_branch_internals();
        let i = index.child(shift);

        if shift.0 == BITS_PER_LEVEL {
            branch.len -= 1;

            let leaf_node = branch.children[i].take().unwrap();
            let leaf = leaf_node.into_leaf().take();

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

            let branch = root.as_mut_branch_internals();
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

    pub fn append(&mut self, that: &mut RrbTree<T>) {
        // ToDo: be careful with various corner cases when tree is not full on left or right side
        // ToDo: or even when one or both of them are empty

        // you're supposed to empty up 'that' tree,
        // including all of its nodes, len and shift properties

        let new_root = self
            .root
            .as_mut()
            .unwrap()
            .merge(that.root.take().unwrap(), self.shift, that.shift);

        // ToDo: prune tree if necessary

        self.root = Some(new_root);
        self.shift = self.shift.inc();
        self.root_len.0 += that.root_len.0;
    }
}

#[cfg(test)]
mod tests {
    extern crate serde;
    extern crate serde_json;

    use self::serde::ser::{Serialize, Serializer, SerializeSeq, SerializeStruct};
    use std::sync::Arc;
    use super::{Branch, Leaf, Node, RrbTree, Shift};
    use super::BITS_PER_LEVEL;
    use super::BRANCH_FACTOR;

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
    #[ignore]
    fn merge_leaves_must_return_balanced_result() {
        //        let mut elements_one: [Option<usize>; BRANCH_FACTOR] = new_branch!();
        //        let mut elements_two: [Option<usize>; BRANCH_FACTOR] = new_branch!();
        //
        //        for i in 0..BRANCH_FACTOR / 2 {
        //            elements_one[i] = Some(i);
        //        }
        //
        //        for i in 0..BRANCH_FACTOR {
        //            elements_two[i] = Some(BRANCH_FACTOR / 2 + i);
        //        }
        //
        //        let leaf_l = Leaf {
        //            elements: elements_one,
        //            len: BRANCH_FACTOR / 2,
        //        };
        //        let leaf_r = Leaf {
        //            elements: elements_two,
        //            len: BRANCH_FACTOR,
        //        };
        //
        //        println!("{:?}", leaf_l);
        //        println!("{:?}", leaf_r);
        //
        //        Leaf::merge(leaf_l, leaf_r);
        //
        //        let branch = leaf_l.merge(leaf_r);
        //
        //        let leaf_l = branch.children[0].as_ref().unwrap();
        //        let leaf_r = branch.children[1].as_ref().unwrap();
        //
        //        if let (Node::Leaf(ref this), Node::Leaf(ref that)) = (leaf_l, leaf_r) {
        //            assert_eq!(this.len, BRANCH_FACTOR);
        //            assert_eq!(that.len, BRANCH_FACTOR / 2);
        //
        //            for i in 0..BRANCH_FACTOR {
        //                assert_eq!(this.elements[i].unwrap(), i);
        //            }
        //
        //            for i in 0..BRANCH_FACTOR / 2 {
        //                assert_eq!(that.elements[i].unwrap(), BRANCH_FACTOR + i);
        //            }
        //
        //            for i in BRANCH_FACTOR / 2..BRANCH_FACTOR {
        //                assert_eq!(that.elements[i], None);
        //            }
        //        }
        //
        //        for i in 2..BRANCH_FACTOR {
        //            assert_eq!(branch.children[i], None);
        //        }
        //
        //        println!("{:?}", branch);
    }

    #[test]
    fn merge_branches_must_return_balanced_result() {
        let mut item = 0;

        let mut branch_l = new_branch!();
        for i in 0..BRANCH_FACTOR {
            let mut items = new_branch!();
            for j in 0..BRANCH_FACTOR {
                items[j] = Some(item);
                item += 1;
            }

            branch_l[i] = Some(Node::Leaf(Arc::new(Leaf {
                elements: items,
                len: BRANCH_FACTOR,
            })));
        }

        let mut branch_r = new_branch!();
        for i in 0..BRANCH_FACTOR {
            let mut items = new_branch!();
            for j in 0..BRANCH_FACTOR {
                items[j] = Some(item);
                item += 1;
            }

            branch_r[i] = Some(Node::Leaf(Arc::new(Leaf {
                elements: items,
                len: BRANCH_FACTOR,
            })));
        }

        let mut node_l = Node::Branch(Arc::new(Branch {
            children: branch_l,
            len: BRANCH_FACTOR,
        }));

        let mut node_r = Node::Branch(Arc::new(Branch {
            children: branch_r,
            len: BRANCH_FACTOR,
        }));

        // node_l.merge(&mut node_r, Shift(BITS_PER_LEVEL));
    }

    #[test]
    fn concat_must_return_expected_result() {
        let mut tree_l = RrbTree::new();
        let mut tree_r = RrbTree::new();

        let mut branch_i = 0;

        for i in 0..BRANCH_FACTOR * BRANCH_FACTOR {
            let mut items = new_branch!();

            for j in 0..BRANCH_FACTOR {
                items[j] = Some(branch_i);
                branch_i += 1;
            }

            tree_l.push(items, BRANCH_FACTOR);
        }

        for i in 0..BRANCH_FACTOR {
            let mut items = new_branch!();

            for j in 0..BRANCH_FACTOR {
                items[j] = Some(branch_i);
                branch_i += 1;
            }

            tree_r.push(items, BRANCH_FACTOR);
        }

        let mut tree_l_clone = tree_l.clone();
        let mut tree_r_clone = tree_r.clone();

        tree_l.append(&mut tree_r);

        for i in 0..tree_l_clone.len() {
            println!("tree_l_clone: item={:?}", tree_l_clone.get(i))
        }

        println!("=====");

        for i in 0..tree_r_clone.len() {
            println!("tree_r_clone: item={:?}", tree_r_clone.get(i));
        }

        for i in 0..tree_l.len() {
            println!("tree_l: item={:?}", tree_l.get(i))
        }

//        for i in 0..tree_r.len() {
//            println!("tree_r: item={:?}", tree_r.get(i))
//        }

        println!("tree_l={}", serde_json::to_string(&tree_l).unwrap());
        println!(
            "tree_l_clone={}",
            serde_json::to_string(&tree_l_clone).unwrap()
        );
        println!(
            "tree_r_clone={}",
            serde_json::to_string(&tree_r_clone).unwrap()
        );
    }
}
