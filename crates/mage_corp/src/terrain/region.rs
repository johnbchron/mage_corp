use std::ops::Rem;

use bevy::prelude::*;
use planiscope::mesher::MesherRegion;
use spatialtree::{tree::OctTree, OctVec};

use super::*;

#[derive(Debug, Clone, Copy, Reflect, PartialEq)]
pub struct TerrainRegion {
  pub position: Vec3,
  pub scale:    Vec3,
  pub subdivs:  u8,
}

impl From<TerrainRegion> for MesherRegion {
  fn from(value: TerrainRegion) -> Self {
    Self {
      position: value.position.into(),
      scale:    value.scale.into(),
      detail:   planiscope::mesher::MesherDetail::Subdivs(value.subdivs),
      prune:    false,
    }
  }
}

// TODO: this is terrible
impl Hash for TerrainRegion {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    // use the debug format to hash
    format!("{self:?}").hash(state);
  }
}

pub fn calculate_regions(
  config: &TerrainConfig,
  target_location: Vec3,
) -> Vec<TerrainRegion> {
  let render_cube_origin = target_location
    - target_location.rem(config.render_cube_translation_increment());
  // the target relative to the render cube
  let offset_target = target_location - render_cube_origin;
  // map to 0.0..1.0 within the render cube
  let target_float_coords = ((offset_target / config.render_dist) + 1.0) / 2.0;
  let target_lod_coords =
    OctVec::from_float_coords(target_float_coords.into(), config.n_sizes);

  let mut tree: OctTree<(), OctVec> = OctTree::with_capacity(32, 32);
  tree.lod_update(
    &[target_lod_coords],
    config.n_same_size_meshes.into(),
    |_| (),
    |_, ()| {},
  );

  tree
    .iter_chunks()
    .map(|(_, chunk)| {
      // take the chunk's coords, map them from 0.0..1.0 to -1.0..1.0, then
      // un-normalize them from the render cube
      let float_size = chunk.position().float_size();
      let float_coords = Vec3::from_array(chunk.position().float_coords());

      let pos = ((float_coords + float_size / 2.0) * 2.0 - 1.0)
        * config.render_dist
        + render_cube_origin;
      let scale = float_size * config.render_dist * config.mesh_bleed;
      TerrainRegion {
        position: pos,
        scale:    Vec3::splat(scale),
        subdivs:  config.mesh_subdivs,
      }
    })
    .collect()
}
