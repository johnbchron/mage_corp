use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Player;

pub struct MarkerPlugin;

impl Plugin for MarkerPlugin {
  fn build(&self, app: &mut App) { app.register_type::<Player>(); }
}
