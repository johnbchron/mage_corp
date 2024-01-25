use bevy::prelude::*;

use crate::{spawnable::Spawnable, Primitive};

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
}

impl Spawnable for RenderedPrimitive {
  type SpawnContext = Entity;

  fn spawn(&self, world: &mut World, context: Self::SpawnContext) {
    self.primitive.spawn(world, (context, self.transform));
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
}

impl Spawnable for RenderedFragment {
  type SpawnContext = (Entity, Transform);

  fn spawn(
    &self,
    world: &mut World,
    (comp_entity, transform): Self::SpawnContext,
  ) {
    // spawn the fragment entity by itself.
    let fragment_entity = world
      .spawn((
        SpatialBundle::from_transform(transform),
        RenderedFragmentMarker,
        Name::new("building_fragment"),
      ))
      .id();
    // add the fragment entity as a child of the composition entity.
    world
      .entity_mut(comp_entity)
      .push_children(&[fragment_entity]);

    // spawn each primitive into the fragment entity. they'll add themselves as
    // children of the fragment entity.
    for primitive in self.primitives.iter() {
      primitive.spawn(world, fragment_entity);
    }

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
