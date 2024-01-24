use bevy::prelude::*;

pub(crate) trait FindOrAdd {
  type A: Asset + Reflect;
  fn find_or_add(&mut self, asset: Self::A) -> Handle<Self::A>;
}

impl<A: Asset + Reflect> FindOrAdd for Assets<A> {
  type A = A;
  fn find_or_add(&mut self, asset: Self::A) -> Handle<Self::A> {
    let handle = self
      .ids()
      .find(|id| self.get(*id).unwrap().reflect_partial_eq(&asset).unwrap())
      .map(Handle::Weak);
    handle.unwrap_or_else(|| self.add(asset))
  }
}
