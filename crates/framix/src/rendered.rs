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
pub struct RenderedFragment {
  primitives: Vec<RenderedPrimitive>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RenderedFragmentMarker;

impl RenderedFragment {
  pub fn new(primitives: Vec<RenderedPrimitive>) -> Self { Self { primitives } }

  pub fn spawn(
    &self,
    parent: &mut ChildBuilder,
    materials: &mut Assets<ToonMaterial>,
    transform: Transform,
  ) {
    parent
      .spawn((
        SpatialBundle::from_transform(transform),
        RenderedFragmentMarker,
        Name::new("building_fragment"),
      ))
      .with_children(|p| {
        for primitive in &self.primitives {
          primitive.spawn(p, materials);
        }
      });

    debug!(
      "spawned rendered fragment with {} primitives",
      self.primitives.len()
    );
  }
}

/// Debug plugin for rendering cubes around fragments.
pub struct FragmentDebugPlugin;

impl Plugin for FragmentDebugPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<RenderedFragmentMarker>()
      .add_systems(Update, render_fragment_debug_cubes);
  }
}

fn render_fragment_debug_cubes(
  mut gizmos: Gizmos,
  q: Query<&GlobalTransform, With<RenderedFragmentMarker>>,
) {
  for transform in q.iter() {
    gizmos.cuboid(*transform, Color::WHITE);
  }
}
