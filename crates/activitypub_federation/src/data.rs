use std::{ops::Deref, sync::Arc};

/// This type can be used to pass your own data into library functions and traits. It can be useful
/// to pass around database connections or other context.
#[derive(Debug)]
pub struct Data<T: ?Sized>(Arc<T>);

impl<T> Data<T> {
  /// Create new `Data` instance.
  pub fn new(state: T) -> Data<T> {
    Data(Arc::new(state))
  }

  /// Get reference to inner app data.
  pub fn get_ref(&self) -> &T {
    self.0.as_ref()
  }

  /// Convert to the internal Arc<T>
  pub fn into_inner(self) -> Arc<T> {
    self.0
  }
}

impl<T: ?Sized> Deref for Data<T> {
  type Target = Arc<T>;

  fn deref(&self) -> &Arc<T> {
    &self.0
  }
}

impl<T: ?Sized> Clone for Data<T> {
  fn clone(&self) -> Data<T> {
    Data(self.0.clone())
  }
}
