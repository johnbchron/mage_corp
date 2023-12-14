use std::ops::Rem;

use bevy::prelude::*;
use bevy_implicits::prelude::*;
use spatialtree::{OctTree, OctVec};

use super::config::TerrainConfig;

pub fn calculate_regions(
  config: &TerrainConfig,
  target_location: Vec3,
) -> Vec<MesherRegion> {
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
      MesherRegion {
        position: pos.into(),
        scale:    Vec3::splat(scale).into(),
        detail:   MesherDetail::Subdivs(config.mesh_subdivs),
        prune:    false,
      }
    })
    .collect()
}
