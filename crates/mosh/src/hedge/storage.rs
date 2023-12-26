use hashbrown::{hash_map::rayon::ParKeys, HashMap};

use super::keys::OpaqueKey;

/// Marks a type as storable in a `Storage`.
pub trait Storable: Clone {
  /// The key type used to identify this storable.
  type Key: OpaqueKey;
}

/// A storage container for mesh elements.
#[derive(Clone, Debug)]
pub struct Storage<T: OpaqueKey, S: Clone> {
  map:        HashMap<T, S>,
  running_id: u64,
}

impl<T: OpaqueKey, S: Clone> Storage<T, S> {
  /// Creates a new empty storage container.
  pub fn new() -> Self {
    Self {
      map:        HashMap::new(),
      running_id: 0,
    }
  }
  /// Adds a new element to the storage container and returns its key.
  pub fn add(&mut self, value: S) -> T {
    let id = T::new(self.running_id);
    self.running_id += 1;
    self.map.insert(id, value);
    id
  }
  /// Iterates over the elements in the storage container.
  pub fn iter(&self) -> impl Iterator<Item = &S> { self.map.values() }
  /// Iterates over the elements in the storage container mutably.
  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut S> {
    self.map.values_mut()
  }
  /// Iterates over the keys in the storage container.
  pub fn iter_keys(&self) -> impl Iterator<Item = T> {
    self.map.keys().copied().collect::<Vec<_>>().into_iter()
  }
  /// Returns a reference to the element with the given key, if it exists.
  pub fn get(&self, key: T) -> Option<&S> { self.map.get(&key) }
  /// Returns a mutable reference to the element with the given key, if it
  /// exists.
  pub fn get_mut(&mut self, key: T) -> Option<&mut S> { self.map.get_mut(&key) }
  /// Removes the element with the given key from the storage container and
  /// returns it, if it existed.
  pub fn remove(&mut self, key: T) -> Option<S> { self.map.remove(&key) }
  /// Retains elements in the storage container that satisfy the given
  /// predicate.
  pub fn retain<F: FnMut(&T, &mut S) -> bool>(&mut self, f: F) {
    self.map.retain(f);
  }
  /// Returns a reference to the inner [`HashMap`].
  pub fn inner(&self) -> &HashMap<T, S> { &self.map }
  /// Returns the number of elements in the storage container.
  pub fn len(&self) -> usize { self.map.len() }
}

impl<T: OpaqueKey + Sync, S: Clone + Sync> Storage<T, S> {
  /// Iterates over the elements in the storage container in parallel.
  ///
  /// # Invariants
  /// The elements must be `Sync`.
  pub fn par_iter_keys(&self) -> ParKeys<T, S> { self.map.par_keys() }
}

impl<T: OpaqueKey, S: Clone> Default for Storage<T, S> {
  fn default() -> Self { Self::new() }
}
