use sharedptr::{SharedPtr, Take};
use std::cmp;
use std::fmt::Debug;
use std::mem;

#[cfg(not(feature = "small_branch"))]
pub const BRANCH_FACTOR: usize = 32;

#[cfg(feature = "small_branch")]
pub const BRANCH_FACTOR: usize = 4;

#[cfg(not(feature = "small_branch"))]
const BITS_PER_LEVEL: usize = 5;

#[cfg(feature = "small_branch")]
const BITS_PER_LEVEL: usize = 2;

#[cfg(not(feature = "small_branch"))]
macro_rules! new_branch {
    () => {
        [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None,
        ]
    };
}

#[cfg(feature = "small_branch")]
macro_rules! new_branch {
    () => {
        [None, None, None, None]
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    fn is_level_with_leaves(self) -> bool {
        self.0 == BITS_PER_LEVEL
    }

    #[inline(always)]
    fn is_leaf_level(self) -> bool {
        self.0 == 0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

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

#[derive(Debug)]
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
    RelaxedBranch(SharedPtr<RelaxedBranch<T>>),
    Branch(SharedPtr<Branch<T>>),
    Leaf(SharedPtr<Leaf<T>>),
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

        let that_len = that.len;

        while index_l < BRANCH_FACTOR && index_r < that_len {
            leaf_l.add(that.take(index_r));

            index_l += 1;
            index_r += 1;
        }

        let that_len = that.len;

        for _ in 0..that_len {
            leaf_r.add(that.take(index_r));
            index_r += 1;
        }

        if leaf_l.is_full() && leaf_r.is_full() {
            let mut children = new_branch!();
            children[0] = Some(Node::Leaf(SharedPtr::new(leaf_l)));
            children[1] = Some(Node::Leaf(SharedPtr::new(leaf_r)));

            Node::Branch(SharedPtr::new(Branch { children, len: 2 }))
        } else {
            let mut sizes = new_branch!();
            sizes[0] = Some(leaf_l.len);
            sizes[1] = Some(leaf_l.len + leaf_r.len);

            let mut children = new_branch!();
            children[0] = Some(Node::Leaf(SharedPtr::new(leaf_l)));
            children[1] = Some(Node::Leaf(SharedPtr::new(leaf_r)));

            Node::RelaxedBranch(SharedPtr::new(RelaxedBranch {
                children,
                sizes,
                len: 2,
            }))
        }
    }

    // ToDo: explore a way to avoid re-allocating left tree
    // ToDo: this would probably require taking owned type in
    // #[inline(always)]
    // fn split_at(&mut self, index: Index) -> (Node<T>, Node<T>) {
    //     let mut leaf_left = Leaf::new();
    //     let mut leaf_right = Leaf::new();

    //     let self_len = self.len;

    //     for i in 0..index.0 {
    //         leaf_left.add(self.take(i));
    //     }

    //     for i in index.0..self_len {
    //         leaf_right.add(self.take(i));
    //     }

    //     (
    //         Node::Leaf(SharedPtr::new(leaf_left)),
    //         Node::Leaf(SharedPtr::new(leaf_right)),
    //     )
    // }

    // ToDo: explore a way to avoid re-allocating left tree
    // ToDo: this would probably require taking owned type in
    #[inline(always)]
    fn split_right_at(&mut self, index: Index) -> Node<T> {
        let mut leaf_left = Leaf::new();

        for i in 0..index.0 {
            leaf_left.add(self.take(i));
        }

        Node::Leaf(SharedPtr::new(leaf_left))
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

        let builder_subtree_shift = shift.dec();

        let mut new_root = BranchBuilder::new(shift);
        let mut new_subtree = BranchBuilder::new(builder_subtree_shift);
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

                        new_subtree.push(Node::Leaf(SharedPtr::new(new_leaf)));
                        new_leaf = Leaf::new();
                    }

                    new_leaf.add(SharedPtr::make_mut(&mut old_leaf).take(i).take());
                }
            }
        }

        check_subtree(&mut new_root, &mut new_subtree);

        if !new_leaf.is_empty() {
            new_subtree.push(Node::Leaf(SharedPtr::new(new_leaf)));
        }

        if !new_subtree.is_empty() {
            new_root.push(new_subtree.build());
        }

        new_root.build()
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
            Node::RelaxedBranch(SharedPtr::new(RelaxedBranch {
                children,
                sizes,
                len,
            }))
        } else {
            Node::Branch(SharedPtr::new(Branch { children, len }))
        }
    }

    #[inline(always)]
    fn push(&mut self, child: Node<T>) {
        self.is_relaxed = self.is_relaxed || child.is_relaxed_node();
        self.children[self.len] = Some(child);
        self.len += 1;
    }

    #[inline(always)]
    fn give(&mut self, child: Option<Node<T>>) {
        if let Some(ref node) = child.as_ref() {
            self.is_relaxed = self.is_relaxed || node.is_relaxed_node();
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
            Node::Leaf(ref leaf) => {
                debug_assert_eq!(shift.0, 0);
                leaf.len
            }
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

        let builder_subtree_shift = shift.dec();
        let builder_node_shift = builder_subtree_shift.dec();

        let mut builder_root = BranchBuilder::new(shift);
        let mut builder_subtree = BranchBuilder::new(builder_subtree_shift);
        let mut builder_node = BranchBuilder::new(builder_node_shift);

        for mut old_node in merged {
            if builder_node.is_empty() && old_node.is_full() {
                check_subtree(&mut builder_root, &mut builder_subtree);
                builder_subtree.push(old_node);
            } else {
                for old_child_node in old_node.as_mut_children() {
                    if builder_node.is_full() {
                        check_subtree(&mut builder_root, &mut builder_subtree);
                        builder_subtree.push(builder_node.build());
                    }

                    builder_node.give(old_child_node.take());
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
    fn add(&mut self, child: Option<Node<T>>) {
        self.children[self.len] = child;
        self.len += 1;
    }

    #[inline(always)]
    fn take(&mut self, i: usize) -> Option<Node<T>> {
        self.len -= 1;
        self.children[i].take()
    }

    #[inline(always)]
    fn push_leaf(&mut self, index: Index, shift: Shift, leaf: Leaf<T>) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let mut branch = self;
        let mut shift = shift;

        while shift.0 > BITS_PER_LEVEL {
            let i = index.child(shift);

            let child = &mut branch.children[i];
            let len = &mut branch.len;

            let node = child.get_or_insert_with(|| {
                *len += 1;

                Node::Branch(SharedPtr::new(Branch {
                    children: new_branch!(),
                    len: 0,
                }))
            });

            branch = SharedPtr::make_mut(node.as_mut_branch());
            shift = shift.dec();
        }

        debug_assert_eq!(shift.0, BITS_PER_LEVEL);

        branch.len += 1;
        branch.children[index.child(shift)] = Some(Node::Leaf(SharedPtr::new(leaf)));
    }

    #[inline(always)]
    fn pop_leaf(&mut self, shift: Shift) -> (Leaf<T>, usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let index = self.len - 1;

        if shift.is_level_with_leaves() {
            self.len -= 1;

            let leaf_node = self.children[index].take().unwrap();
            let leaf = leaf_node.into_leaf().take();

            (leaf, self.len)
        } else {
            let (leaf, child_len) = self.children[index]
                .as_mut()
                .map(|child| child.pop_leaf(shift.dec()))
                .unwrap();

            if child_len == 0 {
                self.len -= 1;
                self.children[index] = None;
            }

            (leaf, self.len)
        }
    }

    #[inline(always)]
    fn split_right_at(&mut self, index: Index) -> Branch<T> {
        let mut branch_left = Branch::new();

        for i in 0..index.0 {
            branch_left.add(self.take(i));
        }

        branch_left
    }
}

impl<T: Clone + Debug> RelaxedBranch<T> {
    #[inline(always)]
    fn push_leaf(
        &mut self,
        index: Index,
        shift: Shift,
        shift_new_branch: Option<Shift>,
        leaf: Leaf<T>,
    ) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);
        debug_assert!(self.len > 0);

        let mut branch = self;
        let mut index = index;
        let mut shift = shift;

        while shift.0 > BITS_PER_LEVEL {
            let mut branch_index = branch.len - 1;

            if let Some(shift_new_branch_value) = shift_new_branch {
                if shift == shift_new_branch_value {
                    branch_index += 1;
                }
            }

            shift = shift.dec();

            let len = &mut branch.len;
            let child_node = &mut branch.children[branch_index];
            let child_node_size = branch.sizes[branch_index];

            if let Some(ref size) = child_node_size {
                branch.sizes[branch_index] = Some(size + leaf.len);
            } else {
                if branch_index == 0 {
                    branch.sizes[branch_index] = Some(leaf.len);
                } else {
                    branch.sizes[branch_index] =
                        Some(branch.sizes[branch_index - 1].unwrap() + leaf.len);
                }
            }

            let node = child_node.get_or_insert_with(|| {
                *len += 1;

                Node::Branch(SharedPtr::new(Branch {
                    children: new_branch!(),
                    len: 0,
                }))
            });

            if branch_index != 0 {
                index = Index(index.0 - branch.sizes[branch_index - 1].unwrap());
            }

            branch = match node {
                Node::RelaxedBranch(ref mut branch_arc) => SharedPtr::make_mut(branch_arc),
                Node::Branch(ref mut branch_arc) => {
                    SharedPtr::make_mut(branch_arc).push_leaf(index, shift, leaf);
                    return;
                }
                Node::Leaf(..) => unreachable!(),
            }
        }

        debug_assert_eq!(shift.0, BITS_PER_LEVEL);

        let branch_index = branch.len;

        if branch_index == 0 {
            branch.sizes[branch_index] = Some(leaf.len);
        } else {
            branch.sizes[branch_index] = Some(branch.sizes[branch_index - 1].unwrap() + leaf.len);
        }

        branch.len += 1;
        branch.children[branch_index] = Some(Node::Leaf(SharedPtr::new(leaf)));
    }

    #[inline(always)]
    fn pop_leaf(&mut self, shift: Shift) -> (Leaf<T>, usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        let index = self.len - 1;

        if shift.is_level_with_leaves() {
            self.len -= 1;

            let leaf_node = self.children[index].take().unwrap();
            let leaf = leaf_node.into_leaf().take();

            let size = self.sizes[index].as_mut().unwrap();
            *size -= leaf.len;

            (leaf, self.len)
        } else {
            let (leaf, child_len) = self.children[index]
                .as_mut()
                .map(|child| child.pop_leaf(shift.dec()))
                .unwrap();

            let size = self.sizes[index].as_mut().unwrap();
            *size -= leaf.len;

            if child_len == 0 {
                self.len -= 1;
                self.children[index] = None;
                self.sizes[index] = None;
            }

            (leaf, self.len)
        }
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
    fn is_relaxed_node(&self) -> bool {
        match self {
            Node::RelaxedBranch(..) => true,
            Node::Branch(ref branch) => branch.len != BRANCH_FACTOR,
            Node::Leaf(ref leaf) => leaf.len != BRANCH_FACTOR,
        }
    }

    #[inline(always)]
    fn as_mut_children(&mut self) -> &mut [Option<Node<T>>] {
        match self {
            Node::Branch(ref mut node) => {
                let branch = SharedPtr::make_mut(node);
                &mut branch.children[..branch.len]
            }
            Node::RelaxedBranch(ref mut node) => {
                let branch = SharedPtr::make_mut(node);
                &mut branch.children[..branch.len]
            }
            Node::Leaf(..) => unreachable!(),
        }
    }

    #[inline]
    fn has_enough_capacity(&self, shift: Shift, index: Index) -> (bool, Option<Shift>) {
        let mut node = self;
        let mut shift = shift;
        let mut idx = index;

        let mut shift_has_enough_capacity = false;
        let mut shift_new_branch = shift.clone();

        while shift.0 > BITS_PER_LEVEL {
            match *node {
                Node::RelaxedBranch(ref relaxed_branch) => {
                    debug_assert!(shift.0 > 0);

                    let sizes = &relaxed_branch.sizes;
                    let child_index = relaxed_branch.len - 1;

                    if child_index != 0 {
                        idx = Index(idx.0 - sizes[child_index - 1].unwrap());
                    }

                    node = relaxed_branch.children[child_index].as_ref().unwrap();

                    if child_index < BRANCH_FACTOR - 1 {
                        shift_has_enough_capacity = true;
                        shift_new_branch = shift.clone();
                    }

                    shift = shift.dec();
                }
                Node::Branch(..) => {
                    debug_assert!(shift.0 > 0);
                    let branch_has_enough_capacity = !(idx.0 >> shift.inc().0 > 0);

                    if !branch_has_enough_capacity && shift_has_enough_capacity {
                        return (shift_has_enough_capacity, Some(shift_new_branch));
                    }

                    return (branch_has_enough_capacity, None);
                }
                Node::Leaf(..) => unreachable!(),
            }
        }

        if node.len() < BRANCH_FACTOR {
            shift_has_enough_capacity = true;
            shift_new_branch = shift;
        }

        return (shift_has_enough_capacity, Some(shift_new_branch));
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
                SharedPtr::make_mut(self.as_mut_leaf()).merge(that.into_leaf().take())
            } else {
                let branch_l = self.as_mut_children();
                let branch_r = that.as_mut_children();

                let (child_l, init) = branch_l.split_last_mut().unwrap();
                let (child_r, tail) = branch_r.split_first_mut().unwrap();

                let child_node_l = child_l.as_mut().unwrap();
                let child_node_r = child_r.take().unwrap();

                let mut branch_c = if self_shift.is_level_with_leaves() {
                    SharedPtr::make_mut(child_node_l.as_mut_leaf())
                        .merge(child_node_r.into_leaf().take())
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

        if shift.is_level_with_leaves() {
            Leaf::rebalance(merged, shift)
        } else {
            BranchBuilder::rebalance(merged, shift)
        }
    }
}

impl<T: Clone + Debug> Node<T> {
    #[inline(always)]
    fn as_mut_branch(&mut self) -> &mut SharedPtr<Branch<T>> {
        if let Node::Branch(ref mut branch_arc) = self {
            branch_arc
        } else {
            unreachable!()
        }
    }

    #[inline(always)]
    fn as_mut_leaf(&mut self) -> &mut SharedPtr<Leaf<T>> {
        if let Node::Leaf(ref mut leaf_arc) = self {
            leaf_arc
        } else {
            unreachable!();
        }
    }

    #[inline(always)]
    fn into_leaf(self) -> SharedPtr<Leaf<T>> {
        if let Node::Leaf(leaf_arc) = self {
            leaf_arc
        } else {
            unreachable!()
        }
    }
}

impl<T: Clone + Debug> Node<T> {
    fn push(&mut self, index: Index, shift: Shift, shift_new_branch: Option<Shift>, leaf: Leaf<T>) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        match self {
            Node::RelaxedBranch(ref mut branch_arc) => {
                SharedPtr::make_mut(branch_arc).push_leaf(index, shift, shift_new_branch, leaf);
            }
            Node::Branch(ref mut branch_arc) => {
                SharedPtr::make_mut(branch_arc).push_leaf(index, shift, leaf);
            }
            Node::Leaf(..) => unreachable!(),
        }
    }

    fn pop(&mut self, shift: Shift) -> Leaf<T> {
        self.pop_leaf(shift).0
    }

    fn pop_leaf(&mut self, shift: Shift) -> (Leaf<T>, usize) {
        debug_assert!(shift.0 >= BITS_PER_LEVEL);

        match self {
            Node::RelaxedBranch(ref mut branch_arc) => {
                SharedPtr::make_mut(branch_arc).pop_leaf(shift)
            }
            Node::Branch(ref mut branch_arc) => SharedPtr::make_mut(branch_arc).pop_leaf(shift),
            Node::Leaf(..) => unreachable!(),
        }
    }

    fn get(&self, index: Index, shift: Shift) -> Option<&T> {
        #[inline(always)]
        fn get_branch_index(sizes: &[Option<usize>], index: Index) -> usize {
            let mut candidate = 0;

            while candidate < BRANCH_FACTOR - 1 && sizes[candidate].unwrap() <= index.0 {
                candidate += 1;
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
        #[inline(always)]
        fn get_branch_index(sizes: &mut [Option<usize>], index: Index) -> usize {
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
                Node::RelaxedBranch(ref mut branch_arc) => {
                    debug_assert!(shift.0 > 0);

                    let branch = SharedPtr::make_mut(branch_arc);

                    let sizes = &mut branch.sizes;
                    let branch_index = get_branch_index(sizes, idx);

                    if branch_index != 0 {
                        idx = Index(idx.0 - sizes[branch_index - 1].unwrap());
                    }

                    node = branch.children[branch_index].as_mut().unwrap();
                    shift = shift.dec();
                }
                Node::Branch(ref mut branch_arc) => {
                    debug_assert!(shift.0 > 0);

                    let branch = SharedPtr::make_mut(branch_arc);

                    node = branch.children[idx.child(shift)].as_mut().unwrap();
                    shift = shift.dec();
                }
                Node::Leaf(ref mut leaf_arc) => {
                    debug_assert_eq!(shift.0, 0);

                    let leaf = SharedPtr::make_mut(leaf_arc);
                    return leaf.elements[idx.element()].as_mut();
                }
            }
        }
    }
}

impl<T: Clone + Debug> Node<T> {
    // ToDo: add has_left param as an optimisation
    fn split_right_at(&mut self, shift: Shift, index: Index) -> (Self, Index, Shift) {
        return if shift.is_leaf_level() {
            let left = SharedPtr::make_mut(self.as_mut_leaf()).split_right_at(index);
            let left_len = Index(left.len());

            (left, left_len, shift)
        } else {
            let subshift = shift.dec();
            let subidx = index.child(shift);

            let branch = SharedPtr::make_mut(self.as_mut_branch());
            let child = branch.children[subidx].as_mut().unwrap();

            let (left, left_len, _) = child.split_right_at(subshift, index);

            if subidx == 0 {
                unreachable!()
            } else {
                let mut root = branch.split_right_at(Index(subidx));
                root.children[subidx] = Some(left);

                // ToDo: check that the left_len or len return value matches expectations
                let left = Node::Branch(SharedPtr::new(root));
                (left, left_len, shift)
            }
        };
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RrbTree<T> {
    root: Option<Node<T>>,
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

    #[cold]
    pub fn push(&mut self, tail: [Option<T>; BRANCH_FACTOR], tail_len: usize) {
        let shift = self.shift;
        let root_len = self.root_len;

        if let Some(ref mut root) = self.root {
            let (shift_has_enough_capacity, shift_new_branch) =
                root.has_enough_capacity(shift, root_len);

            if !shift_has_enough_capacity {
                let mut new_children = new_branch!();
                new_children[0] = Some(root.clone());

                self.shift = self.shift.inc();

                *root = match root {
                    Node::RelaxedBranch(ref branch) => {
                        let mut new_sizes = new_branch!();
                        new_sizes[0] = branch.sizes[branch.len - 1];
                        new_sizes[1] = branch.sizes[branch.len - 1];

                        new_children[1] = Some(Node::Branch(SharedPtr::new(Branch {
                            children: new_branch!(),
                            len: 0,
                        })));

                        Node::RelaxedBranch(SharedPtr::new(RelaxedBranch {
                            children: new_children,
                            sizes: new_sizes,
                            len: 2,
                        }))
                    }
                    Node::Branch(..) => Node::Branch(SharedPtr::new(Branch {
                        children: new_children,
                        len: 1,
                    })),
                    Node::Leaf(..) => Node::Branch(SharedPtr::new(Branch {
                        children: new_children,
                        len: 1,
                    })),
                }
            }

            root.push(
                self.root_len,
                self.shift,
                shift_new_branch,
                Leaf {
                    elements: tail,
                    len: tail_len,
                },
            );
        } else {
            self.root = Some(Node::Leaf(SharedPtr::new(Leaf {
                elements: tail,
                len: tail_len,
            })));
        }

        self.root_len.0 += tail_len;
    }

    pub fn pop(&mut self) -> ([Option<T>; BRANCH_FACTOR], usize) {
        if self.shift.is_leaf_level() {
            let leaf = self.root.take().unwrap().into_leaf().take();

            self.root_len.0 -= leaf.len;
            return (leaf.elements, leaf.len);
        }

        let root = self.root.as_mut().unwrap();

        let leaf = root.pop(self.shift);
        self.root_len.0 -= leaf.len;

        if root.len() == 1 {
            self.shift = self.shift.dec();

            *root = match root {
                Node::RelaxedBranch(ref mut branch_arc) => {
                    SharedPtr::make_mut(branch_arc).children[0].take().unwrap()
                }
                Node::Branch(ref mut branch_arc) => {
                    SharedPtr::make_mut(branch_arc).children[0].take().unwrap()
                }
                Node::Leaf(..) => unreachable!(),
            };
        }

        (leaf.elements, leaf.len)
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

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.root_len.0
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        } else {
            unreachable!();
        }
    }

    pub fn split_right_at(&mut self, mid: usize) -> Self {
        return if let Some(root) = self.root.as_mut() {
            let (left_root, left_len, left_shift) = root.split_right_at(self.shift, Index(mid));

            let left = RrbTree {
                root: Some(left_root),
                root_len: left_len,
                shift: left_shift,
            };

            left
        } else {
            panic!()
        };
    }
}

pub mod iter;
mod serializer;

#[cfg(test)]
#[macro_use]
mod test {
    use super::RrbTree;
    use super::BRANCH_FACTOR;

    #[test]
    #[should_panic]
    fn split_right_empty_tree() {
        let mut tree: RrbTree<usize> = RrbTree::new();
        tree.split_right_at(0);
    }

    #[test]
    fn split_right_when_root_is_leaf() {
        let mut elements = new_branch!();

        for i in 0..BRANCH_FACTOR {
            elements[i] = Some(i);
        }

        let mut tree = RrbTree::new();
        tree.push(elements, BRANCH_FACTOR);

        let left = tree.split_right_at(BRANCH_FACTOR / 2);
        assert_eq!(left.len(), BRANCH_FACTOR / 2);

        for index in 0..BRANCH_FACTOR / 2 {
            assert_eq!(index, left.get(index).cloned().unwrap());
        }
    }
}
