use mosh::BufMesh;
use parry3d::shape::SharedShape;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Hash, Serialize, Deserialize)]
pub enum ColliderSettings {
  #[default]
  ConvexDecomposition,
  TriMesh,
}

pub fn generate_collider(
  full_mesh: BufMesh,
  settings: &ColliderSettings,
) -> Option<SharedShape> {
  if full_mesh.triangles.is_empty() {
    return None;
  }

  match settings {
    ColliderSettings::ConvexDecomposition => {
      Some(SharedShape::convex_decomposition(
        full_mesh
          .positions
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
        .positions
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
