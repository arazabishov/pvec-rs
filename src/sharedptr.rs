#[cfg(not(feature = "arc"))]
use std::rc::Rc;
#[cfg(feature = "arc")]
use std::sync::Arc;

use std::fmt::Debug;

#[cfg(feature = "arc")]
pub type SharedPtr<K> = Arc<K>;

#[cfg(not(feature = "arc"))]
pub type SharedPtr<K> = Rc<K>;

pub trait Take<T: Clone + Debug> {
    fn take(self) -> T;
}

impl<T: Clone + Debug> Take<T> for SharedPtr<T> {
    fn take(mut self) -> T {
        // ToDo: you have to verify whether this method is thread-safe
        SharedPtr::make_mut(&mut self);
        SharedPtr::try_unwrap(self).unwrap()
    }
}
