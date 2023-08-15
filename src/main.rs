mod debug;
mod low_res;
mod player;
mod test_scene;
mod toon;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

use crate::{
  debug::DebugPlugin,
  low_res::{LowResCamera, LowResPlugin},
  player::PlayerPlugin,
  test_scene::TestScenePlugin,
  toon::{ToonMaterial, ToonPlugin},
};

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
    .add_plugins(ToonPlugin)
    .add_plugins(LowResPlugin)
    .insert_resource(Msaa::Off)
    // physics
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    // player
    .add_plugins(PlayerPlugin)
    // QoL
    .add_plugins(WorldInspectorPlugin::new())
    // setup
    .add_plugins(TestScenePlugin)
    // debug
    .add_plugins(DebugPlugin)
    .run();
}
