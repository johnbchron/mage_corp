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

  /// Returns the pairs of vertices that make up the edges of this face.
  pub fn pairs(&self) -> [(u32, u32); 3] {
    [
      (self.vertices.x, self.vertices.y),
      (self.vertices.y, self.vertices.z),
      (self.vertices.z, self.vertices.x),
    ]
  }
}
