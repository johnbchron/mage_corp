use bevy::prelude::*;
use common::materials::ToonMaterial;

use crate::Primitive;

/// A rendered [`Primitive`].
pub struct RenderedPrimitive {
  primitive: Box<dyn Primitive>,
  transform: Transform,
}

impl RenderedPrimitive {
  /// Create a new [`RenderedPrimitive`].
  pub fn new(primitive: Box<dyn Primitive>, transform: Transform) -> Self {
    Self {
      primitive,
      transform,
    }
  }

  fn spawn(
    &self,
    parent: &mut ChildBuilder,
    materials: &mut Assets<ToonMaterial>,
  ) {
    self.primitive.spawn(parent, materials, self.transform);
  }
}

#[derive(Reflect)]
pub struct RenderedModule {
  primitives: Vec<RenderedPrimitive>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RenderedModuleMarker;

impl RenderedModule {
  pub fn new(primitives: Vec<RenderedPrimitive>) -> Self { Self { primitives } }

  pub fn spawn(
    &self,
    parent: &mut ChildBuilder,
    materials: &mut Assets<ToonMaterial>,
    transform: Transform,
  ) {
    println!(
      "spawning rendered module with {} primitives",
      self.primitives.len()
    );

    parent
      .spawn((
        SpatialBundle::from_transform(transform),
        RenderedModuleMarker,
        Name::new("building_module"),
      ))
      .with_children(|p| {
        for primitive in &self.primitives {
          primitive.spawn(p, materials);
        }
      });
  }
}

/// Debug plugin for rendering [`RenderedModule`] cubes.
pub struct RenderedModulePlugin;

impl Plugin for RenderedModulePlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<RenderedModuleMarker>()
      .add_systems(Update, render_module_debug_cubes);
  }
}

fn render_module_debug_cubes(
  mut gizmos: Gizmos,
  q: Query<&Transform, With<RenderedModuleMarker>>,
) {
  for transform in q.iter() {
    gizmos.cuboid(*transform, Color::WHITE);
  }
}
