use super::position::Position;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grid<T> {
  elements: Vec<T>,
  size:     Position,
}

impl<T> Grid<T> {
  pub fn new(elements: Vec<T>, size: Position) -> Self {
    assert_eq!(elements.len(), size.grid_count());
    Self { elements, size }
  }
  pub fn size(&self) -> Position { self.size }
  pub fn get(&self, position: Position) -> Option<&T> {
    let index = position.index(&self.size);
    self.elements.get(index)
  }
  pub fn get_indexed(&self, index: usize) -> Option<&T> {
    self.elements.get(index)
  }
  pub fn set(&mut self, position: Position, value: T) {
    let index = position.index(&self.size);
    self.elements[index] = value;
  }
  pub fn iter_values(&self) -> impl Iterator<Item = &T> { self.elements.iter() }
  pub fn iter_values_mut(&mut self) -> impl Iterator<Item = &mut T> {
    self.elements.iter_mut()
  }
  pub fn iter_entries(&self) -> impl Iterator<Item = (Position, &T)> {
    self.elements.iter().enumerate().map(move |(index, value)| {
      (Position::from_index(index, &self.size), value)
    })
  }
  pub fn iter_entries_mut(
    &mut self,
  ) -> impl Iterator<Item = (&mut T, Position)> {
    self.elements.iter_mut().zip(self.size.iter_positions())
  }
  pub(crate) fn values(&self) -> &Vec<T> { &self.elements }
}

impl<T: Clone> Grid<T> {
  pub fn new_with_fill(default: T, size: Position) -> Self {
    let elements = vec![default; (size.x() * size.y() * size.z()) as usize];
    Self::new(elements, size)
  }
}

impl<T: Clone> Grid<Option<T>> {
  pub fn unwrap_all(self) -> Grid<T> {
    let elements = self.elements.into_iter().map(|v| v.unwrap()).collect();
    Grid::new(elements, self.size)
  }
}
