/// A trait for vertex data.
///
/// This trait is to allow arbitrary vertex data to be used with
/// [`MizuMesh`](super::MizuMesh). The only visibility requirement is that the
/// vertex data must contain a position, which is used to compute the normal of
/// a face.
pub trait VertexData: Clone + Sync {
  /// Returns the position of the vertex.
  fn pos(&self) -> glam::Vec3A;
}

/// A vertex in a [`MizuMesh`](super::MizuMesh).
#[derive(Clone)]
pub struct Vertex<D: VertexData> {
  data: D,
}

impl<D: VertexData> Vertex<D> {
  /// Creates a new vertex with the given data.
  pub fn new(data: D) -> Self { Self { data } }

  /// Returns the position of the vertex.
  pub fn pos(&self) -> glam::Vec3A { self.data.pos() }

  /// Returns the data of the vertex.
  pub fn data(&self) -> &D { &self.data }

  /// Determines if the given vertices are collinear.
  pub fn are_collinear(a: &Self, b: &Self, c: &Self) -> bool {
    glam::Mat3::from_cols(a.pos().into(), b.pos().into(), c.pos().into())
      .determinant()
      .abs()
      < 0.0001
  }
}
