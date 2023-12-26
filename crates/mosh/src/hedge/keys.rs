use std::{fmt::Debug, hash::Hash};

/// A key that can be used to uniquely identify an element within a mesh.
pub trait OpaqueKey:
  Copy + PartialEq + Eq + Hash + PartialOrd + Ord + Debug + Clone
{
  /// A key that is guaranteed to be invalid.
  const INVALID: Self;

  /// Creates a new key with the given ID.
  fn new(id: u64) -> Self;
}

/// A key that can be used to uniquely identify a vertex within a mesh.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VertexKey(u64);

impl OpaqueKey for VertexKey {
  const INVALID: Self = VertexKey(u64::MAX);
  fn new(id: u64) -> Self { VertexKey(id) }
}

/// A key that can be used to uniquely identify an edge within a mesh.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeKey(u64);

impl OpaqueKey for EdgeKey {
  const INVALID: Self = EdgeKey(u64::MAX);
  fn new(id: u64) -> Self { EdgeKey(id) }
}

/// A key that can be used to uniquely identify a face within a mesh.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FaceKey(u64);

impl OpaqueKey for FaceKey {
  const INVALID: Self = FaceKey(u64::MAX);
  fn new(id: u64) -> Self { FaceKey(id) }
}
