use std::ops::Rem;

use bevy::prelude::*;
use spatialtree::{tree::OctTree, OctVec};

use super::*;

#[derive(Debug, Clone, Copy, Reflect)]
pub struct TerrainRegion {
  pub position:    Vec3,
  pub scale:       Vec3,
  pub max_subdivs: u8,
  pub min_subdivs: u8,
}

pub fn calculate_regions(
  config: &TerrainConfig,
  target_location: Vec3,
) -> Vec<TerrainRegion> {
  let render_cube_origin = target_location
    - target_location.rem(config.render_cube_translation_increment);
  // the target relative to the render cube
  let offset_target = target_location - render_cube_origin;
  // map to 0.0..1.0
  let target_float_coords = ((offset_target / config.render_dist) + 1.0) / 2.0;
  let target_lod_coords =
    OctVec::from_float_coords(target_float_coords.into(), config.n_sizes);

  let mut tree: OctTree<(), OctVec> = OctTree::with_capacity(32, 32);
  tree.lod_update(
    &[target_lod_coords],
    config.n_same_size_meshes.into(),
    |_| (),
    |_, _| {},
  );

  tree
    .iter_chunks()
    .map(|(_, chunk)| {
      // take the chunk's coords, map them from 0.0..1.0 to -1.0..1.0, then
      // un-normalize them from the render cube
      let pos = (Vec3::from_array(chunk.position().float_coords()) * 2.0 - 1.0)
        * config.render_dist * 2.0
        + render_cube_origin;
      let scale = chunk.position().float_size() * config.render_dist * 2.0;
      TerrainRegion {
        position:    pos,
        scale:       Vec3::splat(scale),
        max_subdivs: config.mesh_max_subdivs,
        min_subdivs: config.mesh_min_subdivs,
      }
    })
    .collect()
}
