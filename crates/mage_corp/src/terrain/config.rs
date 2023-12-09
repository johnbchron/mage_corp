use bevy::prelude::*;

#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct TerrainConfig {
  /// The half-size of the render cube to build (smallest distance from player
  /// to edge).
  pub render_dist: f32,
  /// Controls how far from the previous target position the target needs to
  /// move to trigger a terrain regeneration. The trigger distance is equal
  /// to `render_dist / 2.0_f32.powf(render_cube_subdiv_trigger)`.
  /// Should be greater than or equal to
  /// `render_cube_translation_subdiv_increment`, and less than or equal to
  /// `n_sizes`.
  pub render_cube_subdiv_trigger: f32,
  /// Controls the increment in which the render cube will move from the origin
  /// to follow the player. The increment is equal to `render_dist /
  /// 2.0_f32.powf(render_cube_translation_subdiv_increment)`.
  pub render_cube_translation_subdiv_increment: f32,
  /// Controls the maximum subdivisions of each mesh.
  pub mesh_subdivs: u8,
  /// Controls how much each mesh bleeds into the next.
  pub mesh_bleed: f32,
  /// Controls the minimum number of same-sized meshes that form the border
  /// between two sizes.
  pub n_same_size_meshes: u8,
  /// Controls how many times to subdivide the world box.
  pub n_sizes: u8,
  /// Whether to place 1/8th scale cubes at the position of each mesh.
  pub debug_transform_cubes: bool,
}

impl TerrainConfig {
  pub fn render_cube_translation_increment(&self) -> f32 {
    self.render_dist
      / 2.0_f32.powf(self.render_cube_translation_subdiv_increment)
  }
  pub fn trigger_distance(&self) -> f32 {
    self.render_dist / 2.0_f32.powf(self.render_cube_subdiv_trigger)
  }
}

impl Default for TerrainConfig {
  fn default() -> Self {
    Self {
      render_dist: 1000.0,
      render_cube_subdiv_trigger: 4.0,
      render_cube_translation_subdiv_increment: 3.0,
      mesh_subdivs: 6,
      mesh_bleed: 1.05,
      n_same_size_meshes: 1,
      n_sizes: 5,
      debug_transform_cubes: false,
    }
  }
}
