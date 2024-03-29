use super::position::Position;

/// A 3D grid of values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grid<T> {
  elements: Vec<T>,
  size:     Position,
}

impl<T> Grid<T> {
  /// Create a new grid with the given elements and size. Panics if the number
  /// of elements does not match the size.
  pub fn new(elements: Vec<T>, size: Position) -> Self {
    assert_eq!(elements.len(), size.grid_count());
    Self { elements, size }
  }
  /// Returns the size of the grid.
  pub fn size(&self) -> Position { self.size }
  /// Returns a reference to the element at the given position. None if the
  /// position is out of bounds.
  pub fn get(&self, position: Position) -> Option<&T> {
    let index = position.index(&self.size);
    self.elements.get(index)
  }
  /// Returns a reference to the element at the given index. None if the index
  /// is out of bounds.
  pub fn get_indexed(&self, index: usize) -> Option<&T> {
    self.elements.get(index)
  }
  /// Sets the element at the given position to the given value. Panics if the
  /// position is out of bounds.
  pub fn set(&mut self, position: Position, value: T) {
    let index = position.index(&self.size);
    self.elements[index] = value;
  }
  /// Returns an iterator over the values of the grid.
  pub fn iter_values(&self) -> impl Iterator<Item = &T> { self.elements.iter() }
  /// Returns a mutable iterator over the values of the grid.
  pub fn iter_values_mut(&mut self) -> impl Iterator<Item = &mut T> {
    self.elements.iter_mut()
  }
  /// Returns an iterator over position-value pairs in the grid.
  pub fn iter_entries(&self) -> impl Iterator<Item = (Position, &T)> {
    self.elements.iter().enumerate().map(move |(index, value)| {
      (Position::from_index(index, &self.size), value)
    })
  }
  /// Returns a mutable iterator over position-value pairs in the grid.
  pub fn iter_entries_mut(
    &mut self,
  ) -> impl Iterator<Item = (&mut T, Position)> {
    self.elements.iter_mut().zip(self.size.iter_positions())
  }
  // /// Returns a reference to the underlying elements.
  // pub(crate) fn values(&self) -> &Vec<T> { &self.elements }
}

impl<T: Clone> Grid<T> {
  /// Returns a new grid with the given size, filled with the given default
  /// value.
  pub fn new_with_fill(default: T, size: Position) -> Self {
    let elements = vec![default; (size.x() * size.y() * size.z()) as usize];
    Self::new(elements, size)
  }
}

impl<T: Clone> Grid<Option<T>> {
  /// Bulk-unwraps all elements in the grid, returning a new grid with the
  /// unwrapped values. Panics if any element is None.
  pub fn unwrap_all(self) -> Grid<T> {
    let elements = self.elements.into_iter().map(|v| v.unwrap()).collect();
    Grid::new(elements, self.size)
  }
}
