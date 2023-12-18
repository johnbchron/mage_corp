use ahash::AHashSet as HashSet;

use super::{grid::Grid, position::Position, Element};

/// A type to help with building adjacency constraints.
///
/// The sampler has methods beginning with `test_` that can be used to build
/// constraints. These methods test the value one or more cells with a boolean
/// closure. Under the hood, the test is applied to the cell's value if it has a
/// value, or to every possible value if it doesn't. If the condition is true
/// for any possible value in the given cell, the constraint is satisfied.
pub struct Sampler<'a, T: Element> {
  /// The position of the cell being sampled.
  pub here: Position,
  values:   &'a Grid<Option<T>>,
  domains:  &'a Grid<HashSet<T>>,
}

impl<'a, T: Element> Sampler<'a, T> {
  pub(crate) fn new(
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

  /// Tests a condition against the value at the given absolute position within
  /// the grid. If the given position is out of bounds, the test returns false.
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

  /// Tests a condition against the value at the given position relative to the
  /// current cell. If the given position is out of bounds, the test returns
  /// false.
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

  /// Tests a condition against the neighbors of the current cell. This function
  /// returns a vector of booleans, one for each neighbor. This uses the
  /// Moore neighborhood, which includes diagonals.
  ///
  /// If the given neighbor is out of bounds, it's excluded, so the returned
  /// vector may be shorter than 26 elements, and may not be in order. If you
  /// need to test against a specific neighbor, use `test_relative` instead.
  pub fn test_neighbors<F: Fn(&T) -> bool>(&self, f: F) -> Vec<bool> {
    self
      .here
      .neighbors(&self.values.size())
      .iter()
      .map(|neighbor| self.test_absolute(*neighbor, &f))
      .collect()
  }
  /// Tests a condition against the direct neighbors of the current cell. This
  /// function returns a vector of booleans, one for each neighbor. This uses
  /// the Von Neumann neighborhood, which excludes diagonals.
  ///
  /// If the given neighbor is out of bounds, it's excluded, so the returned
  /// vector may be shorter than 6 elements, and may not be in order. If you
  /// need to test against a specific neighbor, use `test_relative` instead.
  pub fn test_direct_neighbors<F: Fn(&T) -> bool>(&self, f: F) -> Vec<bool> {
    self
      .here
      .direct_neighbors(&self.values.size())
      .iter()
      .map(|neighbor| self.test_absolute(*neighbor, &f))
      .collect()
  }
}
