mod debug;
mod low_res;
mod particle;
mod player;
mod test_scene;
mod toon;
mod utils;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

fn main() {
  App::new()
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
          }),
          ..Default::default()
        })
        .set(ImagePlugin::default_nearest()),
    )
    // graphics
    .add_plugins(toon::ToonPlugin)
    .add_plugins(low_res::LowResPlugin)
    .insert_resource(Msaa::Off)
    // physics
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    // player
    .add_plugins(player::PlayerPlugin)
    // QoL
    .add_plugins(WorldInspectorPlugin::new())
    // background logic
    .add_plugins(particle::ParticlePlugin)
    .add_plugins(utils::timer_lifetime::TimerLifetimePlugin)
    .add_plugins(utils::despawn::DespawnPlugin)
    // scene setup
    .add_plugins(test_scene::TestScenePlugin)
    // debug
    .add_plugins(debug::DebugPlugin)
    .run();
}
