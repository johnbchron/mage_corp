use parry3d::shape::SharedShape;
use serde::{Deserialize, Serialize};

use crate::mesher::FullMesh;

#[derive(Clone, Debug, Default, Hash, Serialize, Deserialize)]
pub enum ColliderSettings {
  #[default]
  ConvexDecomposition,
  TriMesh,
}

pub fn generate_collider(
  full_mesh: FullMesh,
  settings: &ColliderSettings,
) -> Option<SharedShape> {
  if full_mesh.triangles.is_empty() {
    return None;
  }

  match settings {
    ColliderSettings::ConvexDecomposition => {
      Some(SharedShape::convex_decomposition(
        full_mesh
          .vertices
          .into_iter()
          .map(|v| v.to_array().into())
          .collect::<Vec<_>>()
          .as_slice(),
        full_mesh
          .triangles
          .into_iter()
          .map(|v| v.to_array())
          .collect::<Vec<_>>()
          .as_slice(),
      ))
    }
    ColliderSettings::TriMesh => Some(SharedShape::trimesh(
      full_mesh
        .vertices
        .into_iter()
        .map(|v| v.to_array().into())
        .collect::<Vec<_>>(),
      full_mesh
        .triangles
        .into_iter()
        .map(|v| v.to_array())
        .collect::<Vec<_>>(),
    )),
  }
}
