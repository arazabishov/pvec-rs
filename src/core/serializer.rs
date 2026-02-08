use super::RbVec;
use super::RrbVec;
use serde::ser::{Serialize, SerializeStruct, Serializer};

macro_rules! impl_serializer {
    ($vec:ident, $name:literal) => {
        impl<T> Serialize for $vec<T>
        where
            T: Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
            where
                S: Serializer,
            {
                let mut serde_state = serializer.serialize_struct($name, 1)?;
                serde_state.serialize_field("tree", &self.tree)?;
                serde_state.serialize_field("tail", &self.tail)?;
                serde_state.serialize_field("tail_len", &self.tail_len)?;
                serde_state.end()
            }
        }
    };
}

impl_serializer!(RrbVec, "RrbVec");
impl_serializer!(RbVec, "RbVec");
