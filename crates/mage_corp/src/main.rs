#![warn(clippy::all, clippy::pedantic)]
#![allow(
  clippy::wildcard_imports,
  clippy::needless_pass_by_value,
  clippy::module_name_repetitions
)]
#![feature(trivial_bounds)]

mod camera;
mod debug;
mod foliage;
mod markers;
mod materials;
mod particle;
mod player;
mod terrain;
mod test_scene;
mod utils;

use bevy::{
  asset::ChangeWatcher,
  ecs::schedule::{LogLevel, ScheduleBuildSettings},
  prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::prelude::*;

fn main() {
  App::new()
    // explicitly report system ordering ambiguities
    .edit_schedule(Main, |schedule| {
      schedule.set_build_settings(ScheduleBuildSettings {
        ambiguity_detection: LogLevel::Warn,
        ..default()
      });
    })
    // add default plugins
    .add_plugins(
      DefaultPlugins
        .set(AssetPlugin {
          watch_for_changes: Some(ChangeWatcher {
            delay: std::time::Duration::from_secs(1),
          }),
          ..default()
        })
        .set(ImagePlugin::default_nearest()),
    )
    // semantics
    .add_plugins(markers::MarkerPlugin)
    // graphics
    .add_plugins(materials::MaterialsPlugin)
    .add_plugins(camera::low_res::LowResPlugin)
    // physics
    .add_plugins(PhysicsPlugins::default())
    // player
    .add_plugins(player::PlayerPlugin)
    // QoL
    .add_plugins(WorldInspectorPlugin::new())
    .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
    // background logic
    .add_plugins(terrain::TerrainPlugin)
    .add_plugins(camera::posing::CameraPosePlugin)
    //.add_plugins(foliage::FoliagePlugin)
    .add_plugins(particle::ParticlePlugin)
    .add_plugins(utils::timer_lifetime::TimerLifetimePlugin)
    .add_plugins(utils::despawn::DespawnPlugin)
    // scene setup
    .add_plugins(test_scene::TestScenePlugin)
    // debug
    .add_plugins(debug::DebugPlugin)
    .run();
}
