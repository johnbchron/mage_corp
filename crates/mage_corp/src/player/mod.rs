use bevy::prelude::*;

use crate::{
  camera::states::CameraStateTarget, materials::toon::ConvertToToonMaterial,
  terrain::TerrainDetailTarget,
};

pub fn spawn_player(mut commands: Commands, asset_server: ResMut<AssetServer>) {
  commands
    .spawn((
      SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
      CameraStateTarget,
      TerrainDetailTarget,
      Name::new("player"),
    ))
    .with_children(|parent| {
      parent.spawn((
        SceneBundle {
          scene: asset_server.load("scenes/fox.glb#Scene0"),
          transform: Transform::from_xyz(0.0, -0.5, 0.0),
          ..default()
        },
        ConvertToToonMaterial {
          outline_scale: Some(1.0),
          ..default()
        },
      ));
    });
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, spawn_player);
  }
}
