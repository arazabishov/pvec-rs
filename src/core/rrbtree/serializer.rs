extern crate serde;

use super::SharedPtr;
use super::BRANCH_FACTOR;
use super::{Branch, Leaf, Node, RelaxedBranch, RrbTree};

use self::serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

#[cfg(feature = "vis")]
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
                        "refs": SharedPtr::strong_count(relaxed_branch),             
                        "addr": relaxed_branch.get_uuid(),
                        "len": relaxed_branch.len
                    }),
                    Node::Branch(ref branch) => json!({
                        "branch": child,
                        "refs": SharedPtr::strong_count(branch),
                        "addr": branch.get_uuid(),
                        "len": branch.len
                    }),
                    Node::Leaf(ref leaf) => json!({
                        "leaf": child,
                        "refs": SharedPtr::strong_count(leaf),
                        "addr": leaf.get_uuid(),
                        "len": leaf.len
                    }),
                };

                children_refs.push(child_json_value);
            } else {
                children_refs.push(json!(null));
            }
        }

        let mut serde_state = serializer.serialize_seq(Some(BRANCH_FACTOR))?;

        for child in children_refs {
            serde_state.serialize_element(&child)?;
        }

        serde_state.end()
    }
}

#[cfg(feature = "vis")]
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
                            "refs": SharedPtr::strong_count(relaxed_branch),
                            "addr": relaxed_branch.get_uuid(),
                            "len": relaxed_branch.len
                    }),
                    Node::Branch(ref branch) => json!({
                            "branch": child,
                            "refs": SharedPtr::strong_count(branch),
                            "addr": branch.get_uuid(),
                            "len": branch.len
                    }),
                    Node::Leaf(ref leaf) => json!({
                            "leaf": child,
                            "refs": SharedPtr::strong_count(leaf),
                            "addr": leaf.get_uuid(),
                            "len": leaf.len
                    }),
                };

                children_refs.push(child_json_value);
            } else {
                children_refs.push(json!(null));
            }
        }

        let mut serde_state = serializer.serialize_seq(Some(BRANCH_FACTOR))?;

        for child in children_refs {
            serde_state.serialize_element(&child)?;
        }

        serde_state.end()
    }
}

#[cfg(feature = "vis")]
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

        serde_state.end()
    }
}

#[cfg(feature = "vis")]
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

#[cfg(feature = "vis")]
impl<T> Serialize for RrbTree<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
    where
        S: Serializer,
    {
        let root_json_value = self.root.as_ref().and_then(|root| {
            let json = match root {
                Node::RelaxedBranch(ref relaxed_branch) => json!({
                    "relaxedBranch": root,
                    "sizes": relaxed_branch.sizes,
                    "refs": SharedPtr::strong_count(relaxed_branch),
                    "addr": relaxed_branch.get_uuid(),
                    "len": relaxed_branch.len
                }),
                Node::Branch(ref branch) => json!({
                    "branch": root,
                    "refs":  SharedPtr::strong_count(branch),
                    "addr": branch.get_uuid(),
                    "len": branch.len
                }),
                Node::Leaf(ref leaf) => json!({
                    "leaf": root,
                    "refs": SharedPtr::strong_count(leaf),
                    "addr": leaf.get_uuid(),
                    "len": leaf.len
                }),
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
