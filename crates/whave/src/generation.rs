use std::cell::OnceCell;

use ahash::AHashSet as HashSet;
use nanorand::{Rng, WyRand};

use super::{grid::Grid, position::Position, sampler::Sampler, Element};

#[derive(Debug, Clone)]
pub(crate) struct Generation<T: Element> {
  values:  Grid<Option<T>>,
  domains: OnceCell<Grid<HashSet<T>>>,
}

impl<T: Element> Generation<T> {
  pub(crate) fn new(values: Grid<Option<T>>) -> Self {
    Self {
      values,
      domains: OnceCell::new(),
    }
  }

  /// Returns the domains for each cell.
  pub(crate) fn domains(&self) -> &Grid<HashSet<T>> {
    self.domains.get_or_init(|| self.calculate_domains())
  }

  pub(crate) fn values(&self) -> &Grid<Option<T>> { &self.values }

  /// Calculate the domain for each cell.
  fn calculate_domains(&self) -> Grid<HashSet<T>> {
    let constraints = T::constraints();
    debug_assert!(
      constraints.keys().cloned().collect::<HashSet<_>>() == T::full_set(),
      "Constraints are not defined for all possible values. Add an empty \
       entry for values that should not have constraints."
    );
    let mut domains = Grid::new_with_fill(T::full_set(), self.values.size());

    // remove domain for cells that are already set
    self
      .values
      .iter_values()
      .enumerate()
      .filter(|(_, v)| v.is_some())
      .for_each(|(index, _)| {
        let position = Position::from_index(index, &self.values.size());
        domains.set(position, HashSet::new());
      });

    // begin passes
    loop {
      // iterate over all unset positions
      let old_domains = domains.clone();
      domains
        .iter_entries_mut()
        .filter(|(domain, _)| !domain.is_empty())
        .for_each(|(domain, position)| {
          // retain possibilities that satisfy all constraints
          let sampler = Sampler::new(position, &self.values, &old_domains);
          domain.retain(|value| {
            constraints
              .get(value)
              .unwrap()
              .iter()
              .all(|constraint| constraint(&sampler))
          })
        });

      // if no domains changed, we're done
      if old_domains == domains {
        break;
      }
    }

    domains
  }

  /// Collapses all cells with only one possible value
  pub(crate) fn collapse(&mut self) -> Grid<Option<T>> {
    let mut diff = Grid::new_with_fill(None, self.values.size());

    // iterate over all unset positions
    for (position, _) in self
      .values
      .clone()
      .iter_values()
      .enumerate()
      .filter(|(_, v)| v.is_none())
    {
      let position = Position::from_index(position, &self.values.size());
      let domain = self.domains().get(position).unwrap();
      if domain.len() == 1 {
        let value = domain.iter().next().unwrap().clone();
        self.values.set(position, Some(value.clone()));
        diff.set(position, Some(value));
      }
    }

    if diff.iter_values().any(|value| value.is_some()) {
      self.domains = OnceCell::new();
    }

    diff
  }

  pub(crate) fn is_unsolvable(&self) -> bool {
    // a cell is unsolvable if it isn't populated and has an empty domain
    self
      .values
      .iter_values()
      .enumerate()
      .filter(|(_, v)| v.is_none())
      .any(|(index, _)| {
        let position = Position::from_index(index, &self.values.size());
        self.domains().get(position).unwrap().is_empty()
      })
  }
  pub(crate) fn is_solved(&self) -> bool {
    self.values.iter_values().all(|value| value.is_some())
  }

  pub(crate) fn guess(&mut self) -> Grid<Option<T>> {
    // find the position with the smallest domain
    let mut smallest_domain = usize::MAX;
    let mut smallest_position = None;
    for (index, domain) in self.domains().iter_values().enumerate() {
      if domain.len() > 0 && domain.len() < smallest_domain {
        smallest_domain = domain.len();
        smallest_position =
          Some(Position::from_index(index, &self.values.size()));
      }
    }
    let smallest_position = smallest_position.unwrap();

    // make a guess
    let mut diff = Grid::new_with_fill(None, self.values.size());
    let mut choices = self
      .domains()
      .get(smallest_position)
      .unwrap()
      .clone()
      .iter()
      .cloned()
      .collect::<Vec<_>>();

    let mut rng = WyRand::new();
    rng.shuffle(&mut choices);
    let guess_value = choices.pop().unwrap();

    self
      .values
      .set(smallest_position, Some(guess_value.clone()));
    self.domains = OnceCell::new();
    diff.set(smallest_position, Some(guess_value));

    diff
  }
}

#[cfg(test)]
mod tests {
  use ahash::AHashMap as HashMap;

  use super::*;
  use crate::Constraint;

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
      constraints.insert(Self::Green, vec![]);
      constraints.insert(Self::Blue, vec![]);
      constraints
    }
  }

  #[test]
  fn test_solver() {
    let mut init_values = Grid::new_with_fill(None, Position::new(3, 3, 1));
    init_values.set(Position::new(0, 0, 0), Some(Color::Red));
    init_values.set(Position::new(1, 1, 0), Some(Color::Green));
    init_values.set(Position::new(2, 2, 0), Some(Color::Blue));

    let solver_gen = Generation::new(init_values);
    let domains = solver_gen.domains();

    // make sure that the domain is zero for all set values
    assert_eq!(domains.get(Position::new(0, 0, 0)).unwrap().len(), 0);
    assert_eq!(domains.get(Position::new(1, 1, 0)).unwrap().len(), 0);
    assert_eq!(domains.get(Position::new(2, 2, 0)).unwrap().len(), 0);

    // make sure the domains of the neighbors of the red value do not include
    // red
    let neighbors = Position::new(0, 0, 0).neighbors(&domains.size());
    for neighbor in neighbors {
      assert!(domains
        .get(neighbor)
        .unwrap()
        .iter()
        .all(|color| *color != Color::Red));
    }
  }
}
