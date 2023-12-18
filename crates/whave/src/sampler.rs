use ahash::AHashSet as HashSet;

use super::{grid::Grid, position::Position, Element};

pub struct Sampler<'a, T: Element> {
  here:    Position,
  values:  &'a Grid<Option<T>>,
  domains: &'a Grid<HashSet<T>>,
}

impl<'a, T: Element> Sampler<'a, T> {
  pub fn new(
    here: Position,
    values: &'a Grid<Option<T>>,
    domains: &'a Grid<HashSet<T>>,
  ) -> Self {
    Self {
      here,
      values,
      domains,
    }
  }

  pub fn test_absolute<F: Fn(&T) -> bool>(
    &self,
    position: Position,
    f: F,
  ) -> bool {
    // abort if we're out of bounds
    let Some(value) = self.values.get(position) else {
      return false;
    };
    if let Some(value) = value {
      f(value)
    } else {
      self.domains.get(position).unwrap().iter().any(f)
    }
  }

  /// Returns all possible values that could be at the given position
  pub fn test_relative<F: Fn(&T) -> bool>(
    &self,
    x: i32,
    y: i32,
    z: i32,
    f: F,
  ) -> bool {
    // abort if we're out of bounds
    let Some(position) = self.here.transform(x, y, z, &self.values.size())
    else {
      return false;
    };
    self.test_absolute(position, f)
  }

  pub fn test_neighbors<F: Fn(&T) -> bool>(&self, f: F) -> Vec<bool> {
    self
      .here
      .neighbors(&self.values.size())
      .iter()
      .map(|neighbor| self.test_absolute(*neighbor, &f))
      .collect()
  }
  pub fn test_direct_neighbors<F: Fn(&T) -> bool>(&self, f: F) -> Vec<bool> {
    self
      .here
      .direct_neighbors(&self.values.size())
      .iter()
      .map(|neighbor| self.test_absolute(*neighbor, &f))
      .collect()
  }
}
