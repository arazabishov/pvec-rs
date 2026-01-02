#[cfg(not(feature = "arc"))]
use std::rc::Rc;
#[cfg(feature = "arc")]
use std::sync::Arc;

use std::fmt::Debug;
use std::ops::Deref;

#[cfg(feature = "vis")]
use uuid::Uuid;

#[cfg(feature = "arc")]
type StdSharedPtr<K> = Arc<K>;

#[cfg(not(feature = "arc"))]
type StdSharedPtr<K> = Rc<K>;

#[cfg(not(feature = "vis"))]
pub type SharedPtr<K> = StdSharedPtr<K>;

#[cfg(feature = "vis")]
pub type SharedPtr<K> = IdentifiableSharedPtr<K>;

#[cfg(feature = "vis")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentifiableSharedPtr<T> {
    data: StdSharedPtr<T>,
    uuid: String,
}

#[cfg(feature = "vis")]
impl<T> IdentifiableSharedPtr<T> {
    pub fn new(data: T) -> IdentifiableSharedPtr<T> {
        IdentifiableSharedPtr {
            data: StdSharedPtr::new(data),
            uuid: Uuid::new_v4().to_string(),
        }
    }

    pub fn get_uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn try_unwrap(this: Self) -> Result<T, Self>
    where
        T: Clone,
    {
        match StdSharedPtr::try_unwrap(this.data) {
            Ok(data) => Ok(data),
            Err(data) => Err(Self {
                data,
                uuid: this.uuid,
            }),
        }
    }
    pub fn strong_count(&self) -> usize {
        StdSharedPtr::strong_count(&self.data)
    }
}

#[cfg(feature = "vis")]
impl<T: Clone> IdentifiableSharedPtr<T> {
    pub fn make_mut(&mut self) -> &mut T {
        StdSharedPtr::make_mut(&mut self.data)
    }
}

#[cfg(feature = "vis")]
impl<T> Deref for IdentifiableSharedPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(feature = "vis")]
impl<T> Clone for IdentifiableSharedPtr<T> {
    fn clone(&self) -> Self {
        Self {
            data: StdSharedPtr::clone(&self.data),
            uuid: format!("{}-cloned", Uuid::new_v4().to_string()),
        }
    }
}

pub trait Take<T: Clone> {
    fn take(self) -> T;
}

impl<T: Clone + Debug> Take<T> for SharedPtr<T> {
    /// Takes the ownership of the underlying value if the reference count is one.
    /// Otherwise, clones the value and returns it.
    fn take(self) -> T {
        SharedPtr::try_unwrap(self).unwrap_or_else(|ptr| (*ptr).clone())
    }
}
