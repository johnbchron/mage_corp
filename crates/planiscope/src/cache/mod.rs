pub mod disk;

use parry3d::shape::SharedShape;

use crate::mesher::{FastSurfaceNetsMesher, FullMesh, Mesher, MesherInputs};

pub struct DiskCacheProvider<M: Mesher> {
  /// The mesher to use.
  pub mesher:        M,
  /// The path prefix to use for storing meshes.
  pub mesh_path:     String,
  /// The path prefix to use for storing colliders.
  pub collider_path: String,
}

impl<M: Mesher + Default> Default for DiskCacheProvider<M> {
  fn default() -> Self {
    Self {
      mesher:        M::default(),
      mesh_path:     "cache/mesh/{:x}".to_string(),
      collider_path: "cache/collider/{:x}".to_string(),
    }
  }
}

pub trait CacheProvider {
  fn get_mesh(&self, inputs: &MesherInputs) -> Result<FullMesh, fidget::Error>;

  fn get_collider(&self, inputs: &MesherInputs) -> Option<SharedShape>;

  fn get_mesh_and_collider(
    &self,
    inputs: &MesherInputs,
  ) -> (Result<FullMesh, fidget::Error>, Option<SharedShape>);
}
