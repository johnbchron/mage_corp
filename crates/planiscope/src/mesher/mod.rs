pub mod fsn_mesher;

use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;
use educe::Educe;
use fidget::eval::Tape;
pub use mosh::{BufMesh, FullVertex};
use serde::{Deserialize, Serialize};
use tracing::info_span;

use crate::shape::Shape;

/// The region over which a mesh is generated.
#[derive(Clone, Debug, Reflect, Educe, Serialize, Deserialize)]
#[educe(Hash)]
pub struct MesherRegion {
  /// The position in node-space around which the mesh is generated.
  #[educe(Hash(method = "hash_vec3a"))]
  pub position: glam::Vec3A,
  /// The half-extents of the mesh.
  #[educe(Hash(method = "hash_vec3a"))]
  pub scale:    glam::Vec3A,
  /// The detail of the mesh.
  pub detail:   MesherDetail,
  /// Whether to prune the mesh's vertices according to the AABB defined by
  /// `position` and `scale`.
  pub prune:    bool,
  /// Whether to use [`mosh`] to simplify the mesh.
  pub simplify: bool,
}

impl MesherRegion {
  pub fn voxel_side_length(&self) -> [u32; 3] {
    match self.detail {
      MesherDetail::Subdivs(x) => [2_u32.pow(x as u32); 3],
      MesherDetail::Resolution(x) => [
        (self.scale.x * x).ceil() as u32,
        (self.scale.y * x).ceil() as u32,
        (self.scale.z * x).ceil() as u32,
      ],
      MesherDetail::Exact(x) => [x; 3],
    }
  }
}

/// A descriptor to determine how many voxel cells to mesh with.
#[derive(Clone, Debug, Reflect, Educe, Serialize, Deserialize)]
#[educe(Hash)]
pub enum MesherDetail {
  /// Subdivides the side length `x` times to determine the voxel count; i.e. a
  /// value of 4 will mesh with 16x16x16 voxels.
  Subdivs(u8),
  /// Allocates `x` voxels per unit in mesh space; i.e. if the mesh's AABB is 4
  /// units on a side, a value of 8.0 will mesh with 32x32x32 voxels. Rounds
  /// up.
  Resolution(#[educe(Hash(trait = "decorum::hash::FloatHash"))] f32),
  /// Controls the exact number of voxels to use; i.e. a value of 32 will mesh
  /// with 32x32x32 voxels.
  Exact(u32),
}

/// All of the inputs required to build a mesh.
#[derive(Clone, Debug, Hash, Reflect, Serialize, Deserialize)]
pub struct MesherInputs {
  pub shape:        Shape,
  pub region:       MesherRegion,
  pub gen_collider: bool,
}

#[derive(Clone, Debug, Default)]
pub struct FastSurfaceNetsMesher;

pub trait Mesher {
  type EvalFamily: fidget::eval::Family;

  fn build_mesh(&self, inputs: &MesherInputs)
    -> Result<BufMesh, fidget::Error>;
}

pub fn fidget_normals<F: fidget::eval::Family>(
  vertices: &[glam::Vec3A],
  tape: &Tape<F>,
) -> Result<Vec<glam::Vec3A>, fidget::Error> {
  let _span = info_span!("planiscope::fidget_normals").entered();

  Ok(
    tape
      .new_grad_slice_evaluator()
      .eval(
        &vertices.iter().map(|v| v.x).collect::<Vec<_>>(),
        &vertices.iter().map(|v| v.y).collect::<Vec<_>>(),
        &vertices.iter().map(|v| v.z).collect::<Vec<_>>(),
        &[],
      )?
      .into_iter()
      .map(|g| glam::Vec3A::new(g.dx, g.dy, g.dz))
      .collect(),
  )
}

pub fn hash_vec3a<H: Hasher>(s: &glam::Vec3A, state: &mut H) {
  s.to_array()
    .iter()
    .for_each(|v| decorum::hash::FloatHash::float_hash(v, state));
}

pub fn hash_uvec3<H: Hasher>(s: &glam::UVec3, state: &mut H) {
  s.to_array().iter().for_each(|v| Hash::hash(v, state));
}
