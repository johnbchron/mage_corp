pub enum StaticOrClosure<T: Clone> {
  Static(T),
  Closure(Box<dyn Fn() -> T + Send + Sync>),
}

impl<T: Clone> StaticOrClosure<T> {
  pub fn get(&self) -> T {
    match self {
      StaticOrClosure::Static(value) => value.clone(),
      StaticOrClosure::Closure(closure) => closure().clone(),
    }
  }
}

impl<T: Clone> From<T> for StaticOrClosure<T> {
  fn from(value: T) -> Self {
    Self::Static(value)
  }
}

impl<T: Clone> From<Box<dyn Fn() -> T + Send + Sync>> for StaticOrClosure<T> {
  fn from(value: Box<dyn Fn() -> T + Send + Sync>) -> Self {
    Self::Closure(value)
  }
}

impl<T: Default + Clone> Default for StaticOrClosure<T> {
  fn default() -> Self {
    Self::Static(T::default())
  }
}
