pub mod despawn;
pub mod in_progress;
pub mod timer_lifetime;

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

use bevy::prelude::Mesh;
use planiscope::mesher::FullMesh;

pub fn f32_lerp(lhs: f32, rhs: f32, s: f32) -> f32 {
  lhs + ((rhs - lhs) * s)
}

pub fn hash_single<H: Hash>(value: &H) -> u64 {
  let mut hasher = DefaultHasher::new();
  value.hash(&mut hasher);
  hasher.finish()
}

pub fn bevy_mesh_from_pls_mesh(mesh: FullMesh) -> Mesh {
  let mut bevy_mesh =
    Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);

  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_POSITION,
    mesh
      .vertices
      .into_iter()
      .map(|v| v.to_array())
      .collect::<Vec<_>>(),
  );
  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_NORMAL,
    mesh
      .normals
      .into_iter()
      .map(|v| v.to_array())
      .collect::<Vec<_>>(),
  );

  bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
    mesh
      .triangles
      .into_iter()
      .flat_map(|v| v.to_array())
      .collect(),
  )));
  bevy_mesh
}
