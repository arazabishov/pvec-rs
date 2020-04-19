#[cfg(not(feature = "arc"))]
use std::rc::Rc;
#[cfg(feature = "arc")]
use std::sync::Arc;

use std::fmt::Debug;

#[cfg(feature = "arc")]
pub type SharedPtr<K> = Arc<K>;

#[cfg(not(feature = "arc"))]
pub type SharedPtr<K> = Rc<K>;

pub trait Take<T: Clone> {
    fn take(self) -> T;
}

impl<T: Clone + Debug> Take<T> for SharedPtr<T> {
    /// Takes the ownership of the underlying value
    /// if the reference count is one. Otherwise,
    /// clones the value and returns it.
    fn take(self) -> T {
        SharedPtr::try_unwrap(self).unwrap_or_else(|ptr| (*ptr).clone())
    }
}
