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
                // Serialize the tail as a node-like struct so the visualization
                // layer can treat it uniformly with tree nodes (e.g. stamp a color
                // on it directly rather than passing color out-of-band).
                let tail_node = serde_json::json!({
                    "elements": &self.tail,
                    "len": self.tail_len
                });

                let mut serde_state = serializer.serialize_struct($name, 2)?;
                serde_state.serialize_field("tree", &self.tree)?;
                serde_state.serialize_field("tail", &tail_node)?;
                serde_state.end()
            }
        }
    };
}

impl_serializer!(RrbVec, "RrbVec");
impl_serializer!(RbVec, "RbVec");
