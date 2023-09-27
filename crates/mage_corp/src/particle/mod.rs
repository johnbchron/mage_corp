use nanorand::Rng;
pub mod descriptor;

use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_xpbd_3d::prelude::*;

use self::descriptor::{
  ParticleAcceleration, ParticleDescriptor, ParticleLinearVelocity,
  ParticleSizeBehavior,
};
use crate::{
  materials::toon::ToonMaterial, utils::timer_lifetime::TimerLifetime,
};

/// Describes the region over which particles are emitted
#[derive(Reflect)]
pub enum ParticleEmitterRegion {
  Point { offset: Option<Vec3> },
}

impl Default for ParticleEmitterRegion {
  fn default() -> Self {
    Self::Point { offset: None }
  }
}

/// A component for emitting particles.
///
/// Requires a `Transform` to emit particles.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ParticleEmitter {
  /// A particle descriptor. Serves as instructions for spawning emitted
  /// particles.
  pub descriptor:  ParticleDescriptor,
  /// The region over which particles are emitted
  pub region:      ParticleEmitterRegion,
  /// How many particles are emitted per second
  pub rate:        f32,
  /// Keeps track of leftover unspawned particles between frames. It should not
  /// be modified manually.
  #[reflect(ignore)]
  pub accumulator: f32,
  /// Whether the emitter is enabled or not
  pub enabled:     bool,
}

impl ParticleEmitter {
  /// Creates a new particle emitter
  pub fn new(
    descriptor: ParticleDescriptor,
    pattern: ParticleEmitterRegion,
    rate: f32,
    enabled: bool,
  ) -> Self {
    Self {
      descriptor,
      region: pattern,
      rate,
      accumulator: 0.0,
      enabled,
    }
  }
}

/// A component for emitted particles.
///
/// This component both serves as a marker for emitted particles and contains
/// information about their original state, used to interpolate their properties
/// over their lifetime.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Particle {
  original_scale:   Vec3,
  shrink_with_life: bool,
}

/// A bundle for spawning emitted particles
#[derive(Bundle, Default)]
pub struct ParticleBundle {
  pub particle:        Particle,
  pub material:        Handle<ToonMaterial>,
  pub mesh:            Handle<Mesh>,
  pub transform:       Transform,
  pub position:        Position,
  pub velocity:        LinearVelocity,
  pub collider:        Collider,
  pub mass_properties: MassPropertiesBundle,
  pub lifetime:        TimerLifetime,
  pub computed:        ComputedVisibility,
  pub visibility:      Visibility,
  pub global:          GlobalTransform,
  pub no_shadows:      NotShadowCaster,
}

impl Default for ParticleEmitter {
  fn default() -> Self {
    Self {
      descriptor:  ParticleDescriptor {
        size: 1.0,
        ..default()
      },
      region:      ParticleEmitterRegion::Point { offset: None },
      rate:        1.0,
      accumulator: 0.0,
      enabled:     true,
    }
  }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn spawn_particles(
  mut commands: Commands,
  mut emitter_query: Query<(&mut ParticleEmitter, &Transform)>,
  time: Res<Time>,
) {
  for (mut emitter, transform) in &mut emitter_query {
    if !emitter.enabled {
      continue;
    }

    emitter.accumulator += emitter.rate * time.delta_seconds();
    let new_particle_count = emitter.accumulator as u16;
    emitter.accumulator -= f32::from(new_particle_count);

    let mut rng = nanorand::tls_rng();

    for _ in 0..new_particle_count {
      // calculate the transform of the new particle
      let transform: Transform = match emitter.region {
        ParticleEmitterRegion::Point { offset } => Transform::from_translation(
          transform.translation + offset.unwrap_or(Vec3::ZERO),
        ),
      }
      .with_scale(Vec3::ONE * emitter.descriptor.size);

      // calculate the velocity of the new particle
      let velocity: LinearVelocity =
        match &emitter.descriptor.behavior.initial_linear_velocity {
          ParticleLinearVelocity::SingleDirection {
            direction,
            magnitude,
          } => LinearVelocity(*direction * *magnitude),
          ParticleLinearVelocity::Spherical { magnitude } => LinearVelocity(
            Vec3::new(
              rng.generate::<f32>() * 2.0 - 1.0,
              rng.generate::<f32>() * 2.0 - 1.0,
              rng.generate::<f32>() * 2.0 - 1.0,
            )
            .normalize()
              * *magnitude,
          ),
          ParticleLinearVelocity::Conic {
            cone_angle,
            direction: cone_direction,
            magnitude: strength,
          } => {
            let cone_angle = *cone_angle;
            let cone_direction = (*cone_direction).normalize();
            let strength = *strength;

            let angle =
              f32::to_radians((rng.generate::<f32>() * 2.0 - 1.0) * cone_angle);
            let axis = Vec3::new(
              rng.generate::<f32>() * 2.0 - 1.0,
              rng.generate::<f32>() * 2.0 - 1.0,
              rng.generate::<f32>() * 2.0 - 1.0,
            )
            .normalize();

            let rotation = Quat::from_axis_angle(axis, angle);
            let direction = Mat3::from_quat(rotation) * cone_direction;

            LinearVelocity(direction * strength)
          }
          ParticleLinearVelocity::None => LinearVelocity::ZERO,
        };

      let mut particle_entity = commands.spawn((
        ParticleBundle {
          particle: Particle {
            original_scale:   Vec3::ONE * emitter.descriptor.size,
            shrink_with_life: matches!(
              emitter.descriptor.behavior.size_behavior,
              ParticleSizeBehavior::LinearShrink
            ),
          },
          material: emitter.descriptor.material.clone(),
          mesh: emitter.descriptor.shape.clone(),
          velocity,
          transform,
          position: Position(transform.translation),
          collider: Collider::ball(emitter.descriptor.size),
          mass_properties: MassPropertiesBundle::new_computed(
            &Collider::ball(emitter.descriptor.size),
            0.1,
          ),
          lifetime: TimerLifetime::new(emitter.descriptor.behavior.lifetime),
          ..default()
        },
        match emitter.descriptor.behavior.acceleration {
          ParticleAcceleration::None => RigidBody::Kinematic,
          ParticleAcceleration::Ballistic => RigidBody::Dynamic,
        },
      ));
      let id = particle_entity.id();
      particle_entity.insert(Name::new(format!("particle_{id:?}")));
    }
  }
}

fn update_particle(
  mut query: Query<(&Particle, &mut Transform, &TimerLifetime)>,
) {
  query.par_iter_mut().for_each_mut(
    |(particle, mut transform, timer_lifetime)| {
      if !particle.shrink_with_life {
        return;
      }
      transform.scale =
        particle.original_scale * timer_lifetime.remaining_frac();
    },
  );
}

/// A plugin for managing particles
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, spawn_particles)
      .add_systems(Update, update_particle)
      .register_type::<ParticleEmitter>()
      .register_type::<Particle>();
  }
}
