use std::cmp;
use std::cmp::Ordering;
use std::fmt::Debug;
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

struct BranchBuilder<T> {
    children: [Option<Node<T>>; BRANCH_FACTOR],
    is_relaxed: bool,
    shift: Shift,
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
        Leaf {
            elements: new_branch!(),
            len: 0,
        }
    }

    #[inline(always)]
    fn add(&mut self, element: Option<T>) {
        self.elements[self.len] = element;
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

        for _ in 0..that.len {
            leaf_r.add(that.take(index_r));

            index_r += 1;
        }

        let mut children = new_branch!();

        children[0] = Some(Node::Leaf(Arc::new(leaf_l)));
        children[1] = Some(Node::Leaf(Arc::new(leaf_r)));

        Node::Branch(Arc::new(Branch { children, len: 2 }))
    }

    #[inline(always)]
    fn rebalance(merged: Vec<Node<T>>) -> Node<T> {
        #[inline(always)]
        fn check_subtree<P: Clone + Debug>(root: &mut Branch<P>, subtree: &mut Branch<P>) {
            if subtree.is_full() {
                root.push(Node::Branch(Arc::new(mem::replace(subtree, Branch::new()))));
            }
        }

        let mut new_root = Branch::new();
        let mut new_subtree = Branch::new();
        let mut new_leaf = Leaf::new();

        for old_node in merged {
            if new_leaf.is_empty() && old_node.is_full() {
                check_subtree(&mut new_root, &mut new_subtree);
                new_subtree.push(old_node);
            } else {
                let mut old_leaf = old_node.into_leaf();

                for i in 0..old_leaf.len {
                    if new_leaf.is_full() {
                        check_subtree(&mut new_root, &mut new_subtree);

                        new_subtree.push(Node::Leaf(Arc::new(new_leaf)));
                        new_leaf = Leaf::new();
                    }

                    new_leaf.add(Arc::make_mut(&mut old_leaf).take(i).take());
                }
            }
        }

        check_subtree(&mut new_root, &mut new_subtree);

        if !new_leaf.is_empty() {
            new_subtree.push(Node::Leaf(Arc::new(new_leaf)));
        }

        if !new_subtree.is_empty() {
            new_root.push(Node::Branch(Arc::new(new_subtree)));
        }

        println!("Leaf::rebalance - new_root={:?}", new_root);
        Node::Branch(Arc::new(new_root))
    }
}

impl<T: Clone + Debug> BranchBuilder<T> {
    #[inline(always)]
    fn new(shift: Shift) -> Self {
        BranchBuilder {
            children: new_branch!(),
            is_relaxed: false,
            shift,
            len: 0,
        }
    }

    #[inline(always)]
    fn build(&mut self) -> Node<T> {
        let is_relaxed = mem::replace(&mut self.is_relaxed, false);
        let children = mem::replace(&mut self.children, new_branch!());
        let len = mem::replace(&mut self.len, 0);

        if is_relaxed {
            let sizes = BranchBuilder::compute_sizes(&children, &self.shift, &len);
            Node::RelaxedBranch(Arc::new(RelaxedBranch {
                children,
                sizes,
                len,
            }))
        } else {
            Node::Branch(Arc::new(Branch { children, len }))
        }
    }

    #[inline(always)]
    fn push(&mut self, child: Node<T>) {
        self.is_relaxed = self.is_relaxed || child.is_relaxed_branch();
        self.children[self.len] = Some(child);
        self.len += 1;
    }

    #[inline(always)]
    fn give(&mut self, child: Option<Node<T>>) {
        if let Some(ref node) = child.as_ref() {
            self.is_relaxed = self.is_relaxed || node.is_relaxed_branch();
            self.children[self.len] = child;
            self.len += 1;
        }
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
    fn compute_sizes(
        children: &[Option<Node<T>>; BRANCH_FACTOR],
        shift: &Shift,
        len: &usize,
    ) -> [Option<usize>; BRANCH_FACTOR] {
        let mut size_sum = 0;
        let mut size_table = new_branch!();

        for i in 0..*len {
            size_sum += BranchBuilder::size_sub_trie(children[i].as_ref().unwrap(), shift);
            size_table[i] = Some(size_sum);
        }

        size_table
    }

    fn size_sub_trie(node: &Node<T>, shift: &Shift) -> usize {
        match node {
            Node::Branch(ref branch) => {
                let last_size = BranchBuilder::size_sub_trie(
                    branch.children[branch.len - 1].as_ref().unwrap(),
                    &shift.dec(),
                );

                ((branch.len - 1) << shift.0) + last_size
            }
            Node::RelaxedBranch(ref relaxed_branch) => {
                relaxed_branch.sizes[relaxed_branch.len - 1].unwrap()
            }
            Node::Leaf(ref leaf) => leaf.len,
        }
    }

    #[inline(always)]
    fn rebalance(merged: Vec<Node<T>>, shift: Shift) -> Node<T> {
        #[inline(always)]
        fn check_subtree<P: Clone + Debug>(
            root: &mut BranchBuilder<P>,
            subtree: &mut BranchBuilder<P>,
        ) {
            if subtree.is_full() {
                root.push(subtree.build());
            }
        }

        let mut builder_root = BranchBuilder::new(shift);
        let mut builder_subtree = BranchBuilder::new(shift.dec());
        let mut builder_node = BranchBuilder::new(shift.dec());

        for old_node in merged {
            if builder_node.is_empty() && old_node.is_full() {
                check_subtree(&mut builder_root, &mut builder_subtree);
                builder_subtree.push(old_node);
            } else {
                let mut old_branch = old_node.into_branch();

                for i in 0..old_branch.len {
                    if builder_node.is_full() {
                        check_subtree(&mut builder_root, &mut builder_subtree);
                        builder_subtree.push(builder_node.build());
                    }

                    builder_node.give(Arc::make_mut(&mut old_branch).take(i).take());
                }
            }
        }

        check_subtree(&mut builder_root, &mut builder_subtree);

        if !builder_node.is_empty() {
            builder_subtree.push(builder_node.build());
        }

        if !builder_subtree.is_empty() {
            builder_root.push(builder_subtree.build());
        }

        builder_root.build()
    }
}

impl<T: Clone + Debug> Branch<T> {
    #[inline(always)]
    fn new() -> Self {
        Branch {
            children: new_branch!(),
            len: 0,
        }
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
        // ToDo: This is definitely not thread safe, so you need to be careful with
        // ToDo: where you call this method
        Arc::make_mut(&mut self);
        Arc::try_unwrap(self).unwrap()
    }
}

impl<T: Clone + Debug> Node<T> {
    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            Node::Branch(ref node) => node.len,
            Node::RelaxedBranch(ref node) => node.len,
            Node::Leaf(ref leaf) => leaf.len,
        }
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        self.len() == BRANCH_FACTOR
    }

    #[inline(always)]
    fn is_relaxed_branch(&self) -> bool {
        match self {
            Node::RelaxedBranch(..) => true,
            Node::Branch(..) => false,
            Node::Leaf(..) => false,
        }
    }

    #[inline(always)]
    fn as_mut_children(&mut self) -> &mut [Option<Node<T>>] {
        match self {
            Node::Branch(ref mut node) => {
                let branch = Arc::make_mut(node);
                &mut branch.children[..branch.len]
            }
            Node::RelaxedBranch(ref mut node) => {
                let branch = Arc::make_mut(node);
                &mut branch.children[..branch.len]
            }
            Node::Leaf(..) => unreachable!(),
        }
    }

    fn merge(&mut self, mut that: Node<T>, self_shift: Shift, that_shift: Shift) -> Node<T> {
        if self_shift > that_shift {
            let branch_l = self.as_mut_children();

            let (child_l, init) = branch_l.split_last_mut().unwrap();
            let child_node_l = child_l.as_mut().unwrap();

            let mut branch_c = child_node_l.merge(that, self_shift.dec(), that_shift);
            Node::rebalance(
                Some(init),
                Some(branch_c.as_mut_children()),
                None,
                self_shift,
            )
        } else if self_shift < that_shift {
            let branch_r = that.as_mut_children();

            let (child_r, tail) = branch_r.split_first_mut().unwrap();
            let child_node_r = child_r.take().unwrap();

            let mut branch_c = self.merge(child_node_r, self_shift, that_shift.dec());
            Node::rebalance(
                None,
                Some(branch_c.as_mut_children()),
                Some(tail),
                that_shift,
            )
        } else {
            if self_shift.0 == 0 {
                Arc::make_mut(self.as_mut_leaf()).merge(that.into_leaf().take())
            } else {
                let branch_l = self.as_mut_children();
                let branch_r = that.as_mut_children();

                let (child_l, init) = branch_l.split_last_mut().unwrap();
                let (child_r, tail) = branch_r.split_first_mut().unwrap();

                let child_node_l = child_l.as_mut().unwrap();
                let child_node_r = child_r.take().unwrap();

                let mut branch_c = if self_shift.0 == BITS_PER_LEVEL {
                    Arc::make_mut(child_node_l.as_mut_leaf()).merge(child_node_r.into_leaf().take())
                } else {
                    child_node_l.merge(child_node_r, self_shift.dec(), that_shift.dec())
                };

                Node::rebalance(
                    Some(init),
                    Some(branch_c.as_mut_children()),
                    Some(tail),
                    self_shift,
                )
            }
        }
    }

    #[inline(always)]
    fn merge_all(
        node_l: Option<&mut [Option<Node<T>>]>,
        node_c: Option<&mut [Option<Node<T>>]>,
        node_r: Option<&mut [Option<Node<T>>]>,
    ) -> Vec<Node<T>> {
        println!("merge_all: node_l={:?}", node_l);
        println!("merge_all: node_c={:?}", node_c);
        println!("merge_all: node_r={:?}", node_r);

        let mut merged = Vec::with_capacity(
            node_l.as_ref().map_or(0, |it| it.len())
                + node_c.as_ref().map_or(0, |it| it.len())
                + node_r.as_ref().map_or(0, |it| it.len()),
        );

        let mut merge_nodes = |mut node: Option<&mut [Option<Node<T>>]>| {
            if let Some(items) = node.as_mut() {
                for item in items.iter_mut() {
                    merged.push(item.take().unwrap());
                }
            }
        };

        merge_nodes(node_l);
        merge_nodes(node_c);
        merge_nodes(node_r);

        merged
    }

    fn rebalance(
        node_l: Option<&mut [Option<Node<T>>]>,
        node_c: Option<&mut [Option<Node<T>>]>,
        node_r: Option<&mut [Option<Node<T>>]>,
        shift: Shift,
    ) -> Node<T> {
        let merged = Node::merge_all(node_l, node_c, node_r);

        if shift.is_leaf_level() {
            Leaf::rebalance(merged)
        } else {
            BranchBuilder::rebalance(merged, shift)
        }
    }
}

impl<T: Clone + Debug> Node<T> {
    #[inline(always)]
    fn as_mut_branch(&mut self) -> &mut Arc<Branch<T>> {
        if let Node::Branch(ref mut branch_arc) = self {
            branch_arc
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
        if let Node::Leaf(leaf_arc) = self {
            leaf_arc
        } else {
            unreachable!()
        }
    }

    #[inline(always)]
    fn into_branch(self) -> Arc<Branch<T>> {
        if let Node::Branch(branch_arc) = self {
            branch_arc
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

            let branch = Arc::make_mut(node.as_mut_branch());
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

        let branch = Arc::make_mut(node.as_mut_branch());

        branch.len += 1;
        branch.children[index.child(shift)] = Some(Node::Leaf(Arc::new(leaf)));
    }

    fn pop(&mut self, index: Index, shift: Shift) -> Leaf<T> {
        self.remove(index, shift).0
    }

    fn remove(&mut self, index: Index, shift: Shift) -> (Leaf<T>, usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let branch = Arc::make_mut(self.as_mut_branch());
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
        #[inline(always)]
        fn get_branch_index(sizes: &[Option<usize>], index: Index) -> usize {
            // ToDo: use binary search to optimize index look-up.
            // ToDo: measure linear search first.
            let mut candidate = 0;

            while sizes[candidate].unwrap() <= index.0 {
                candidate += 1
            }

            candidate
        }

        let mut node = self;
        let mut shift = shift;
        let mut idx = index;

        loop {
            match *node {
                Node::RelaxedBranch(ref relaxed_branch) => {
                    debug_assert!(shift.0 > 0);

                    let sizes = &relaxed_branch.sizes;
                    let branch_index = get_branch_index(sizes, idx);

                    if branch_index != 0 {
                        idx = Index(idx.0 - sizes[branch_index - 1].unwrap());
                    }

                    node = relaxed_branch.children[branch_index].as_ref().unwrap();
                    shift = shift.dec();
                }
                Node::Branch(ref branch) => {
                    debug_assert!(shift.0 > 0);

                    node = branch.children[idx.child(shift)].as_ref().unwrap();
                    shift = shift.dec();
                }
                Node::Leaf(ref leaf) => {
                    debug_assert_eq!(shift.0, 0);

                    return leaf.elements[idx.element()].as_ref();
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

            let branch = Arc::make_mut(root.as_mut_branch());
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
        if let (Some(this_root), Some(that_root)) = (self.root.as_mut(), that.root.take()) {
            let mut merged_root = this_root.merge(that_root, self.shift, that.shift);
            let merged_shift = Shift(cmp::max(self.shift.0, that.shift.0));

            let (new_root, new_shift) = if merged_root.len() == 1 {
                (
                    merged_root.as_mut_children().first_mut().unwrap().take(),
                    merged_shift,
                )
            } else {
                (Some(merged_root), merged_shift.inc())
            };

            self.root = new_root;
            that.root = None;

            self.shift = new_shift;
            that.shift = Shift(0);

            self.root_len.0 += that.root_len.0;
            that.root_len.0 = 0;

            self.root_len_max.0 += that.root_len_max.0;
            that.root_len_max.0 = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate serde;
    extern crate serde_json;

    use self::serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};
    use super::BRANCH_FACTOR;
    use super::{Branch, Leaf, Node, RelaxedBranch, RrbTree};
    use std::sync::Arc;

    impl<T> RelaxedBranch<T>
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
                        Node::RelaxedBranch(ref relaxed_branch) => json!({
                                "relaxedBranch": child,
                                "sizes": relaxed_branch.sizes,
                                "refs": Arc::strong_count(relaxed_branch),
                                "len": relaxed_branch.len
                            }),
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
                        Node::RelaxedBranch(ref relaxed_branch) => json!({
                                "relaxedBranch": child,
                                "sizes": relaxed_branch.sizes,
                                "refs": Arc::strong_count(relaxed_branch),
                                "len": relaxed_branch.len
                            }),
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
                Node::RelaxedBranch(ref relaxed_branch) => relaxed_branch.serialize(serializer),
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
                    Node::RelaxedBranch(ref relaxed_branch) => json!({
                                "relaxedBranch": root,
                                "sizes": relaxed_branch.sizes,
                                "refs": Arc::strong_count(relaxed_branch),
                                "len": relaxed_branch.len
                            }),
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
        //        let mut item = 0;
        //
        //        let mut branch_l = new_branch!();
        //        for i in 0..BRANCH_FACTOR {
        //            let mut items = new_branch!();
        //            for j in 0..BRANCH_FACTOR {
        //                items[j] = Some(item);
        //                item += 1;
        //            }
        //
        //            branch_l[i] = Some(Node::Leaf(Arc::new(Leaf {
        //                elements: items,
        //                len: BRANCH_FACTOR,
        //            })));
        //        }
        //
        //        let mut branch_r = new_branch!();
        //        for i in 0..BRANCH_FACTOR {
        //            let mut items = new_branch!();
        //            for j in 0..BRANCH_FACTOR {
        //                items[j] = Some(item);
        //                item += 1;
        //            }
        //
        //            branch_r[i] = Some(Node::Leaf(Arc::new(Leaf {
        //                elements: items,
        //                len: BRANCH_FACTOR,
        //            })));
        //        }
        //
        //        let mut node_l = Node::Branch(Arc::new(Branch {
        //            children: branch_l,
        //            len: BRANCH_FACTOR,
        //        }));
        //
        //        let mut node_r = Node::Branch(Arc::new(Branch {
        //            children: branch_r,
        //            len: BRANCH_FACTOR,
        //        }));
        //
        //        node_l.merge(&mut node_r, Shift(BITS_PER_LEVEL));
    }

    #[test]
    fn concat_must_return_expected_result() {
        let mut tree_l = RrbTree::new();
        let mut tree_c = RrbTree::new();
        let mut tree_r = RrbTree::new();

        let mut branch_i = 0;

        for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR {
            let mut items = new_branch!();

            for j in 0..BRANCH_FACTOR {
                items[j] = Some(branch_i);
                branch_i += 1;
            }

            tree_l.push(items, BRANCH_FACTOR);
        }

        for _ in 0..BRANCH_FACTOR {
            let mut items = new_branch!();

            for j in 0..BRANCH_FACTOR {
                items[j] = Some(branch_i);
                branch_i += 1;
            }

            tree_c.push(items, BRANCH_FACTOR);
        }

        for _ in 0..BRANCH_FACTOR {
            let mut items = new_branch!();

            for j in 0..BRANCH_FACTOR {
                items[j] = Some(branch_i);
                branch_i += 1;
            }

            tree_r.push(items, BRANCH_FACTOR);
        }

        let tree_l_clone = tree_l.clone();
        let tree_c_clone = tree_c.clone();
        let tree_r_clone = tree_r.clone();

        tree_l.append(&mut tree_c);
        tree_l.append(&mut tree_r);

        for i in 0..tree_l_clone.len() {
            println!("tree_l_clone: item={:?}", tree_l_clone.get(i))
        }

        println!("=====");

        for i in 0..tree_c_clone.len() {
            println!("tree_c_clone: item={:?}", tree_c_clone.get(i));
        }

        println!("=====");

        for i in 0..tree_r_clone.len() {
            println!("tree_r_clone: item={:?}", tree_c_clone.get(i));
        }

        println!("#####");

        println!("tree_l={}", serde_json::to_string(&tree_l).unwrap());
        println!("tree_c={}", serde_json::to_string(&tree_c).unwrap());
        println!("tree_r={}", serde_json::to_string(&tree_r).unwrap());

        println!(
            "tree_l_clone={}",
            serde_json::to_string(&tree_l_clone).unwrap()
        );
        println!(
            "tree_c_clone={}",
            serde_json::to_string(&tree_c_clone).unwrap()
        );
        println!(
            "tree_r_clone={}",
            serde_json::to_string(&tree_r_clone).unwrap()
        );

        for i in 0..tree_l.len() {
            println!("tree_l: item={:?}", tree_l.get(i))
        }

        for i in 0..tree_c.len() {
            println!("tree_c: item={:?}", tree_c.get(i))
        }

        for i in 0..tree_c.len() {
            println!("tree_r: item={:?}", tree_c.get(i))
        }
    }
}
