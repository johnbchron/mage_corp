use bevy::prelude::*;
use bevy_obj::ObjPlugin;

use crate::toon::ConvertToToonMaterial;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(ObjPlugin).add_systems(Startup, spawn_scene);
  }
}

fn spawn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
  let scene_handle = asset_server.load("models/Barn.obj");

  commands.spawn((
    SceneBundle {
      scene: scene_handle,
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..default()
    },
    ConvertToToonMaterial { outline_scale: Some(1.0), ..default() },
  ));
}
