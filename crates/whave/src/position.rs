/// A position within a [`Grid`](crate::Grid).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
  x: u32,
  y: u32,
  z: u32,
}

impl Position {
  /// Creates a new position.
  pub fn new(x: u32, y: u32, z: u32) -> Self { Self { x, y, z } }
  /// Returns the index of the position in a grid with the given extent.
  pub fn index(&self, extent: &Position) -> usize {
    (self.x + self.y * extent.x + self.z * extent.x * extent.y) as usize
  }
  /// Returns the position at the given index in a grid with the given extent.
  pub fn from_index(index: usize, extent: &Position) -> Self {
    let index = index as u32;
    let x = index % extent.x;
    let y = (index / extent.x) % extent.y;
    let z = index / (extent.x * extent.y);
    Self { x, y, z }
  }
  /// Returns whether the position is valid within the given extent.
  pub fn valid(&self, extent: &Position) -> bool {
    self.x < extent.x && self.y < extent.y && self.z < extent.z
  }
  /// Returns the position transformed by the given offset, if it is valid
  pub fn transform(
    self,
    x: i32,
    y: i32,
    z: i32,
    extent: &Position,
  ) -> Option<Self> {
    let pos = Self {
      x: self.x.wrapping_add(x as u32),
      y: self.y.wrapping_add(y as u32),
      z: self.z.wrapping_add(z as u32),
    };
    if pos.valid(extent) {
      Some(pos)
    } else {
      None
    }
  }
  /// Returns the number of positions in the grid, assuming that `self` is a
  /// grid extent.
  pub fn grid_count(&self) -> usize { (self.x * self.y * self.z) as usize }
  /// Returns the positions of all neighbors of the position, excluding
  /// neighbors that are not valid within the given extent. This is the Moore
  /// neighborhood in 3D.
  pub fn neighbors(&self, extent: &Position) -> Vec<Self> {
    let mut neighbors = Vec::new();
    for x in -1..=1 {
      for y in -1..=1 {
        for z in -1..=1 {
          if x == 0 && y == 0 && z == 0 {
            continue;
          }
          if let Some(pos) = self.transform(x, y, z, extent) {
            neighbors.push(pos);
          }
        }
      }
    }
    neighbors
  }
  /// Returns the positions of all direct neighbors of the position, excluding
  /// neighbors that are not valid within the given extent. This is the von
  /// Neumann neighborhood in 3D.
  pub fn direct_neighbors(&self, extent: &Position) -> Vec<Self> {
    let mut neighbors = Vec::new();
    for x in -1..=1 {
      for y in -1..=1 {
        for z in -1..=1 {
          if x == 0 && y == 0 && z == 0 {
            continue;
          }
          if x != 0 && y != 0 && z != 0 {
            continue;
          }
          if let Some(pos) = self.transform(x, y, z, extent) {
            neighbors.push(pos);
          }
        }
      }
    }
    neighbors
  }
  /// Returns an iterator over all positions in the grid, assuming that `self`
  /// is a grid extent.
  pub fn iter_positions(&self) -> impl Iterator<Item = Self> {
    let extent = *self;
    (0..self.grid_count()).map(move |index| Self::from_index(index, &extent))
  }
  /// The x coordinate of the position.
  pub fn x(&self) -> u32 { self.x }
  /// The y coordinate of the position.
  pub fn y(&self) -> u32 { self.y }
  /// The z coordinate of the position.
  pub fn z(&self) -> u32 { self.z }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_index() {
    let extent = Position { x: 3, y: 5, z: 5 };
    assert_eq!(Position::new(1, 0, 0).index(&extent), 1);
    assert_eq!(Position::new(0, 1, 0).index(&extent), 3);
    assert_eq!(Position::new(0, 0, 1).index(&extent), 15);
    assert_eq!(Position::new(1, 1, 0).index(&extent), 4);
    assert_eq!(Position::new(1, 0, 1).index(&extent), 16);
    assert_eq!(Position::new(0, 1, 1).index(&extent), 18);
    assert_eq!(Position::new(1, 1, 1).index(&extent), 19);
    assert_eq!(
      Position::new(2, 4, 4).index(&extent),
      extent.grid_count() - 1
    );
  }
}
