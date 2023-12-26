use hashbrown::{hash_map::rayon::ParKeys, HashMap};

use super::keys::OpaqueKey;

pub trait Storable: Clone {
  type Key: OpaqueKey;
}

#[derive(Clone, Debug)]
pub struct Storage<T: OpaqueKey, S: Clone> {
  map:        HashMap<T, S>,
  running_id: u64,
}

impl<T: OpaqueKey, S: Clone> Storage<T, S> {
  pub fn new() -> Self {
    Self {
      map:        HashMap::new(),
      running_id: 0,
    }
  }
  pub fn add(&mut self, value: S) -> T {
    let id = T::new(self.running_id);
    self.running_id += 1;
    self.map.insert(id, value);
    id
  }
  pub fn iter(&self) -> impl Iterator<Item = &S> { self.map.values() }
  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut S> {
    self.map.values_mut()
  }
  pub fn iter_keys(&self) -> impl Iterator<Item = T> {
    self.map.keys().copied().collect::<Vec<_>>().into_iter()
  }
  pub fn get(&self, key: T) -> Option<&S> { self.map.get(&key) }
  pub fn get_mut(&mut self, key: T) -> Option<&mut S> { self.map.get_mut(&key) }
  pub fn remove(&mut self, key: T) -> Option<S> { self.map.remove(&key) }
  pub fn retain<F: FnMut(&T, &mut S) -> bool>(&mut self, f: F) {
    self.map.retain(f);
  }
}

impl<T: OpaqueKey + Sync, S: Clone + Sync> Storage<T, S> {
  pub fn par_iter_keys(&self) -> ParKeys<T, S> { self.map.par_keys() }
}
