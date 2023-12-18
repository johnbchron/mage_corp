#![feature(associated_type_defaults)]
#![warn(missing_docs)]

//! # Whave
//!
//! `whave` is a Rust library for solving the Wave Function Collapse problem.
//! The Wave Function Collapse problem is a constraint satisfaction problem that
//! takes a grid of values and attempts to collapse the grid to a single
//! solution. The grid can be collapsed by iteratively applying constraints to
//! the grid. The constraints are defined by the element type, which must
//! implement the [`Element`] trait. See the [`Element`] trait for more
//! information.
//!
//! `whave` is opinionated in some aspects and generic in others. It's
//! explicitly 3D, and very permissive on its element type, as these were the
//! main impetus for its design. Make sure you research sufficiently before
//! deciding to use this library.
//!
//! `whave` currently only implements a naive solving approach. This solver will
//! collapse the grid until it can make no more progress, then it will make a
//! guess and continue collapsing. It notably does not support backtracking, so
//! it may not terminate successfully for inputs that should be solvable.
//!
//! ## Examples
//!
//! ```rust
//! use std::collections::{HashSet, HashMap};
//! use whave::{Element, Grid, Position, Solver, Constraint, Sampler};
//!
//! #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
//! enum Color {
//!   Red,
//!   Green,
//!   Blue,
//! }
//!
//! impl Element for Color {
//!   // returns the full set of possible values for the element type
//!   fn full_set() -> std::collections::HashSet<Self> {
//!     vec![Self::Red, Self::Green, Self::Blue]
//!       .into_iter()
//!       .collect()
//!   }
//!   // returns a map of constraints for each value in the element type
//!   fn constraints() -> HashMap<Self, Vec<Constraint<Self>>> {
//!     let mut constraints: HashMap<Self, Vec<Constraint<Self>>> = HashMap::new();
//!     // red can't be adjacent to red
//!     constraints.insert(Self::Red, vec![Box::new(
//!       |sampler: &Sampler<Self>| {
//!         sampler
//!           .test_neighbors(|possibility| *possibility != Self::Red)
//!           .iter()
//!           .all(|v| *v)
//!       },
//!     )]);
//!     // green can't be adjacent to blue
//!     constraints.insert(Self::Green, vec![Box::new(
//!       |sampler: &Sampler<Self>| {
//!         sampler
//!           .test_neighbors(|possibility| *possibility != Self::Blue)
//!           .iter()
//!           .all(|v| *v)
//!       },
//!     )]);
//!     // blue has no constraints
//!     constraints.insert(Self::Blue, vec![]);
//!     constraints
//!   }
//! }
//!
//! let mut init_values = Grid::new_with_fill(None, Position::new(3, 3, 1));
//! init_values.set(Position::new(0, 0, 0), Some(Color::Red));
//! init_values.set(Position::new(1, 0, 0), Some(Color::Green));
//! init_values.set(Position::new(2, 2, 0), Some(Color::Blue));
//!
//! let mut solver = Solver::new(init_values);
//! let solution = solver.naive_solve().unwrap();
//!
//! dbg!(&solution);
//!
//! // [ Red,   Green, Blue,
//! //   Blue,  Blue,  Red,
//! //   Red,   Blue,  Blue  ]
//! ```

mod generation;
mod grid;
mod position;
mod sampler;

use std::{fmt::Debug, hash::Hash};

use ahash::{AHashMap as HashMap, AHashSet as HashSet};

use self::generation::Generation;
pub use crate::{grid::Grid, position::Position, sampler::Sampler};

/// Type alias for element constraint closures.
pub type Constraint<T> = Box<dyn Fn(&Sampler<T>) -> bool>;

/// Represents a type that can be used as an element for Wave Function Collapse.
///
/// The element type must implement `Clone`, `Eq`, `Hash`, and `Debug`.
pub trait Element: Clone + Eq + Hash + Debug {
  /// Returns the full set of possible values for the element type. For example,
  /// if the element type is an enum representing a color, this function would
  /// return a set containing all of the listed colors. This forces the set to
  /// be discrete and numerable. If there are more precise math terms, forgive
  /// me -- I am not a mathematician.
  fn full_set() -> HashSet<Self>;
  /// Returns a map of constraints for the element type. Each key in the map
  /// represents a possible value for the element type. Each value in the map
  /// represents a list of constraints for that value. Each constraint is a
  /// closure that takes a `Sampler` and returns a boolean. The closure should
  /// return true if the constraint is satisfied, and false otherwise. The
  /// `Sampler` can be used to sample the grid in various ways to test the
  /// constraint. See the `Sampler` documentation for more information.
  fn constraints() -> HashMap<Self, Vec<Constraint<Self>>>;
}

/// A Wave Function Collapse solver.
///
/// This is the main interface for the library. The solver is generic over the
/// element type `T`. This type must implement the [`Element`] trait, which is
/// used to specify the constraints for the element type. See the [`Element`]
/// trait for more information.
///
/// The solver can be created with either an initial grid of values, or with an
/// empty grid. If an initial grid is provided, the solver will attempt to
/// collapse the grid to a single solution. If an empty grid is provided, the
/// solver will attempt to generate a random solution.
///
/// Currently only a naive solver is implemented. This solver will collapse the
/// grid until it can make no more progress, then it will make a guess and
/// continue collapsing. This solver is not guaranteed to terminate, but it
/// should terminate for most inputs. It notably does not support backtracking,
/// so it may not terminate for inputs that should be solvable.
#[derive(Debug, Clone)]
pub struct Solver<T: Element> {
  stack: Vec<(Generation<T>, Grid<Option<T>>)>,
}

impl<T: Element> Solver<T> {
  /// Creates a new solver with the given initial values.
  pub fn new(initial: Grid<Option<T>>) -> Self {
    Self {
      stack: vec![(
        Generation::new(initial.clone()),
        Grid::new_with_fill(None, initial.size()),
      )],
    }
  }
  /// Creates a new solver with an empty grid of the given size.
  pub fn new_empty(size: Position) -> Self {
    Self {
      stack: vec![(
        Generation::new(Grid::new_with_fill(None, size)),
        Grid::new_with_fill(None, size),
      )],
    }
  }

  /// Attempts to collapse the grid to a single solution. Returns `None` if the
  /// grid is unsolvable, or if the solver fails to terminate. Does not support
  /// backtracking.
  pub fn naive_solve(&mut self) -> Option<Grid<T>> {
    loop {
      let (mut generation, _) = self.stack.last()?.clone();

      let diff = generation.collapse();

      // if we made progress, push the new generation onto the stack
      if diff.iter_values().any(|value| value.is_some()) {
        self.stack.push((generation, diff));
        continue;
      }

      // if we didn't make progress, check if we're done
      if generation.is_solved() {
        return Some(generation.values().clone().unwrap_all());
      }
      if generation.is_unsolvable() {
        return None;
      }

      // if we're not done, try to make a guess
      let guess = generation.guess();
      self.stack.push((generation, guess));
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{position::Position, *};

  #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
  enum Color {
    Red,
    Green,
    Blue,
  }

  impl Element for Color {
    fn full_set() -> HashSet<Self> {
      vec![Self::Red, Self::Green, Self::Blue]
        .into_iter()
        .collect()
    }
    fn constraints() -> HashMap<Self, Vec<Constraint<Self>>> {
      let mut constraints: HashMap<Self, Vec<Constraint<Self>>> =
        HashMap::new();
      constraints.insert(Self::Red, vec![Box::new(
        |sampler: &Sampler<Self>| {
          sampler
            .test_neighbors(|possibility| *possibility != Self::Red)
            .iter()
            .all(|v| *v)
        },
      )]);
      constraints.insert(Self::Green, vec![Box::new(
        |sampler: &Sampler<Self>| {
          sampler
            .test_neighbors(|possibility| *possibility != Self::Blue)
            .iter()
            .all(|v| *v)
        },
      )]);
      constraints.insert(Self::Blue, vec![]);
      constraints
    }
  }

  #[test]
  fn test_solver() {
    let mut init_values = Grid::new_with_fill(None, Position::new(3, 3, 1));
    init_values.set(Position::new(0, 0, 0), Some(Color::Red));
    init_values.set(Position::new(1, 0, 0), Some(Color::Green));
    init_values.set(Position::new(2, 2, 0), Some(Color::Blue));

    let mut solver = Solver::new(init_values);
    let solution = solver.naive_solve().unwrap();

    dbg!(&solution);

    assert_eq!(solution.get(Position::new(0, 0, 0)), Some(&Color::Red));
    assert_eq!(solution.get(Position::new(1, 0, 0)), Some(&Color::Green));
    assert_eq!(solution.get(Position::new(2, 2, 0)), Some(&Color::Blue));
  }

  #[test]
  fn solver_stress_test() {
    let mut init_values = Grid::new_with_fill(None, Position::new(10, 10, 10));
    init_values.set(Position::new(0, 0, 0), Some(Color::Red));
    init_values.set(Position::new(4, 4, 4), Some(Color::Green));
    init_values.set(Position::new(9, 9, 9), Some(Color::Blue));

    let mut solver = Solver::new(init_values);
    let _solution = solver.naive_solve().unwrap();
  }
}
