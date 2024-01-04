/// A face in a [`MizuMesh`].
pub struct Face {
  vertices: glam::UVec3,
  normal:   glam::Vec3A,
}

impl Face {
  /// Creates a new face from the given vertices and normal.
  pub fn new(vertices: glam::UVec3, normal: glam::Vec3A) -> Self {
    Self { vertices, normal }
  }

  /// Returns the indices of the vertices that make up this face.
  pub fn vertices(&self) -> &glam::UVec3 { &self.vertices }
  /// Returns a mutable reference to the indices of the vertices that make up
  /// this face.
  pub fn vertices_mut(&mut self) -> &mut glam::UVec3 { &mut self.vertices }
  /// Returns the normal of this face.
  pub fn normal(&self) -> glam::Vec3A { self.normal }

  /// Returns the pairs of vertices that make up the edges of this face.
  pub fn pairs(&self) -> [(u32, u32); 3] {
    [
      (self.vertices.x, self.vertices.y),
      (self.vertices.y, self.vertices.z),
      (self.vertices.z, self.vertices.x),
    ]
  }
  /// Returns the requested pair of vertices that make up the edges of this
  /// face.
  pub fn pair(&self, index: u8) -> (u32, u32) {
    match index {
      0 => (self.vertices.x, self.vertices.y),
      1 => (self.vertices.y, self.vertices.z),
      2 => (self.vertices.z, self.vertices.x),
      _ => panic!("Invalid index: {}", index),
    }
  }
  /// Returns the index of the vertex opposite the given edge.
  pub fn vertex_index_opposite_pair(&self, index: u8) -> u32 {
    match index {
      0 => self.vertices.z,
      1 => self.vertices.x,
      2 => self.vertices.y,
      _ => panic!("Invalid index: {}", index),
    }
  }

  /// Returns whether the faces are coplanar.
  pub fn are_coplanar(&self, other: &Self) -> bool {
    self.normal.dot(other.normal) > 0.9999
  }
}
