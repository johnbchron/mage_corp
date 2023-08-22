use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Despawn;

pub fn despawn(mut commands: Commands, query: Query<Entity, With<Despawn>>) {
  for entity in query.iter() {
    commands.entity(entity).despawn_recursive();
  }
}

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, despawn).register_type::<Despawn>();
  }
}
