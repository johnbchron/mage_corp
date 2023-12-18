#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
  x: u32,
  y: u32,
  z: u32,
}

impl Position {
  pub fn new(x: u32, y: u32, z: u32) -> Self { Self { x, y, z } }
  pub fn index(&self, extent: &Position) -> usize {
    (self.x + self.y * extent.x + self.z * extent.x * extent.y) as usize
  }
  pub fn from_index(index: usize, extent: &Position) -> Self {
    let index = index as u32;
    let x = index % extent.x;
    let y = (index / extent.x) % extent.y;
    let z = index / (extent.x * extent.y);
    Self { x, y, z }
  }
  pub fn valid(&self, extent: &Position) -> bool {
    self.x < extent.x && self.y < extent.y && self.z < extent.z
  }
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
  pub fn grid_count(&self) -> usize { (self.x * self.y * self.z) as usize }
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
  pub fn x(&self) -> u32 { self.x }
  pub fn y(&self) -> u32 { self.y }
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
