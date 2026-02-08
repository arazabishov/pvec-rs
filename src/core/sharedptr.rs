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

/// `Rc`/`Arc` wrapper with a stable UUID for web-vis node identity.
/// `clone()` preserves the UUID (structural sharing), `make_mut()` assigns
/// a new one when refcount > 1 (path copying).
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
    /// Clones the inner data if shared (refcount > 1) and assigns a new UUID
    /// to `self`. After the call, `self` holds the new copy; other owners
    /// keep the original data and UUID.
    pub fn make_mut(&mut self) -> &mut T {
        if StdSharedPtr::strong_count(&self.data) > 1 {
            self.uuid = Uuid::new_v4().to_string();
        }
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
    /// Preserves the UUID: both copies point to the same data and share
    /// the same visual identity.
    fn clone(&self) -> Self {
        Self {
            data: StdSharedPtr::clone(&self.data),
            uuid: self.uuid.clone(),
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
