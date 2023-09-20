use bevy::{prelude::*, tasks::Task};
use planiscope::shape::Shape;

use crate::materials::toon::ToonMaterial;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Foliage {
  shape:    Shape,
  material: Handle<ToonMaterial>,
  #[reflect(ignore)]
  status:   FoliageGenerationStatus,
}

pub enum FoliageGenerationStatus {
  NotStarted,
  InProgress { task: Task<Mesh> },
  Completed,
}

impl Default for FoliageGenerationStatus {
  fn default() -> Self {
    Self::NotStarted
  }
}

pub struct FoliagePlugin;

impl Plugin for FoliagePlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<Foliage>()
      .add_systems(Startup, spawn_test_foliage)
      .add_systems(Update, flush_foliage);
  }
}

fn spawn_test_foliage(
  mut commands: Commands,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  commands.spawn((
    SpatialBundle::from_transform(Transform::from_xyz(5.0, 5.0, 5.0)),
    Foliage {
      shape:    Shape::new_expr("sqrt(square(x) + square(y) + square(z)) - 2"),
      material: toon_materials.add(ToonMaterial {
        color: Color::rgb(1.0, 1.0, 0.0),
        ..default()
      }),
      status:   FoliageGenerationStatus::NotStarted,
    },
    Name::new("foliage_test"),
  ));
}

fn flush_foliage(mut commands: Commands, mut foliage_q: Query<&mut Foliage>) {}
