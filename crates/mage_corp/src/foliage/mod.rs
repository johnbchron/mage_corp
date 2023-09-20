use bevy::{prelude::*, tasks::Task};
use planiscope::shape::Shape;

use crate::materials::toon::ToonMaterial;

#[derive(Component)]
pub struct Foliage {
  shape:    Shape,
  material: ToonMaterial,
  status:   FoliageGenerationStatus,
}

pub enum FoliageGenerationStatus {
  NotStarted,
  InProgress(Task<Mesh>),
  Completed,
}

pub struct FoliagePlugin;

impl Plugin for FoliagePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, flush_foliage);
  }
}

pub fn flush_foliage(
  mut commands: Commands,
  mut foliage_q: Query<&mut Foliage>,
) {
}
