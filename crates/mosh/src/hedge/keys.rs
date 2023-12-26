use std::{fmt::Debug, hash::Hash};

pub trait OpaqueKey:
  Copy + PartialEq + Eq + Hash + PartialOrd + Ord + Debug + Clone
{
  fn new(id: u64) -> Self;
  fn get(&self) -> u64;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VertexKey(u64);

impl OpaqueKey for VertexKey {
  fn new(id: u64) -> Self { VertexKey(id) }
  fn get(&self) -> u64 { self.0 }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeKey(u64);

impl OpaqueKey for EdgeKey {
  fn new(id: u64) -> Self { EdgeKey(id) }
  fn get(&self) -> u64 { self.0 }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FaceKey(u64);

impl OpaqueKey for FaceKey {
  fn new(id: u64) -> Self { FaceKey(id) }
  fn get(&self) -> u64 { self.0 }
}
