extern crate serde;
extern crate serde_json;

use self::serde::ser::{Serialize, SerializeStruct, Serializer};
use super::PVec;

impl<T> Serialize for PVec<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S>::Ok, <S>::Error>
    where
        S: Serializer,
    {
        let mut serde_state = serializer.serialize_struct("PVec", 1)?;
        serde_state.serialize_field("tree", &self.tree)?;
        serde_state.serialize_field("tail", &self.tail)?;
        serde_state.serialize_field("tail_len", &self.tail_len)?;
        serde_state.end()
    }
}
