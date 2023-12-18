#![feature(associated_type_defaults)]

mod generation;
mod grid;
mod position;
mod sampler;

use std::{fmt::Debug, hash::Hash};

use ahash::{AHashMap as HashMap, AHashSet as HashSet};
use sampler::Sampler;

use self::{generation::Generation, grid::Grid};

pub type Constraint<T> = Box<dyn Fn(&Sampler<T>) -> bool>;
pub trait Element: Clone + Eq + Hash + Debug {
  fn full_set() -> HashSet<Self>;
  fn constraints() -> HashMap<Self, Vec<Constraint<Self>>>;
}

#[derive(Debug, Clone)]
pub struct Solver<T: Element> {
  stack: Vec<(Generation<T>, Grid<Option<T>>)>,
}

impl<T: Element> Solver<T> {
  pub fn new(initial: Grid<Option<T>>) -> Self {
    Self {
      stack: vec![(
        Generation::new(initial.clone()),
        Grid::new_with_default(None, initial.size()),
      )],
    }
  }

  pub fn naive_solve(&mut self) -> Option<Grid<T>> {
    loop {
      let (mut generation, _) = self.stack.last()?.clone();

      let diff = generation.collapse();

      // if we made progress, push the new generation onto the stack
      if diff.iter().any(|value| value.is_some()) {
        self.stack.push((generation, diff));
        continue;
      }

      // if we didn't make progress, check if we're done
      if generation.is_solved() {
        return Some(generation.values().clone().unwrap_all());
      }
      if generation.is_unsolvable() {
        dbg!(self.stack.len());
        return None;
      }

      // if we're not done, try to make a guess
      let guess = generation.guess();
      dbg!("guessed: ", &guess.values());
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
      constraints.insert(Self::Blue, vec![Box::new(
        |sampler: &Sampler<Self>| {
          sampler
            .test_neighbors(|possibility| *possibility == Self::Blue)
            .iter()
            .any(|v| *v)
        },
      )]);
      constraints
    }
  }

  #[test]
  fn test_solver() {
    let mut init_values = Grid::new_with_default(None, Position::new(3, 3, 1));
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
}
