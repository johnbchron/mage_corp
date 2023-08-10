use bevy::prelude::*;

use crate::toon::ConvertToToonMaterial;

pub fn setup_fox_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn((
    SceneBundle {
      scene: asset_server.load("fox.glb#Scene0"),
      ..default()
    },
    ConvertToToonMaterial,
  ));
}
