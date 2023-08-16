use rand::Rng;
pub mod descriptor;

use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use descriptor::{ParticleDescriptor, ParticleVelocity};

use crate::{toon::ToonMaterial, utils::timer_lifetime::TimerLifetime};

#[derive(Reflect)]
pub enum ParticleEmitterRegion {
  Point { offset: Option<Vec3> },
}

impl Default for ParticleEmitterRegion {
  fn default() -> Self {
    Self::Point { offset: None }
  }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ParticleEmitter {
  pub descriptor:  ParticleDescriptor,
  pub region:      ParticleEmitterRegion,
  pub rate:        f32,
  pub accumulator: f32,
  pub enabled:     bool,
}

impl ParticleEmitter {
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

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Particle {
  original_scale:   Vec3,
  shrink_with_life: bool,
}

#[derive(Bundle, Default)]
pub struct ParticleBundle {
  pub particle:   Particle,
  pub material:   Handle<ToonMaterial>,
  pub mesh:       Handle<Mesh>,
  pub velocity:   Velocity,
  pub rigid_body: RigidBody,
  pub transform:  Transform,
  pub lifetime:   TimerLifetime,
  pub computed:   ComputedVisibility,
  pub visibility: Visibility,
  pub global:     GlobalTransform,
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

fn spawn_particles(
  mut commands: Commands,
  mut emitter_query: Query<(&mut ParticleEmitter, &Transform)>,
  time: Res<Time>,
) {
  for (mut emitter, transform) in emitter_query.iter_mut() {
    if !emitter.enabled {
      continue;
    }

    emitter.accumulator += emitter.rate * time.delta_seconds();
    let new_particle_count = emitter.accumulator.floor() as u32;
    emitter.accumulator -= new_particle_count as f32;

    for _ in 0..new_particle_count {
      // calculate the transform of the new particle
      let transform: Transform = match emitter.region {
        ParticleEmitterRegion::Point { offset } => Transform::from_translation(
          transform.translation + offset.unwrap_or(Vec3::ZERO),
        ),
      }
      .with_scale(Vec3::ONE * emitter.descriptor.size);

      // calculate the velocity of the new particle
      let velocity: Velocity =
        match &emitter.descriptor.behavior.initial_velocity {
          ParticleVelocity::SingleDirection {
            direction,
            magnitude,
          } => Velocity::linear(*direction * *magnitude),
          ParticleVelocity::Spherical { magnitude } => Velocity::linear(
            Vec3::new(
              rand::random::<f32>() * 2.0 - 1.0,
              rand::random::<f32>() * 2.0 - 1.0,
              rand::random::<f32>() * 2.0 - 1.0,
            )
            .normalize()
              * *magnitude,
          ),
          ParticleVelocity::Conic {
            cone_angle,
            direction: cone_direction,
            magnitude: strength,
          } => {
            let cone_angle = *cone_angle;
            let cone_direction = (*cone_direction).normalize();
            let strength = *strength;

            let mut rng = rand::thread_rng();
            let angle =
              f32::to_radians((rng.gen::<f32>() * 2.0 - 1.0) * cone_angle);
            let axis = Vec3::new(
              rng.gen::<f32>() * 2.0 - 1.0,
              rng.gen::<f32>() * 2.0 - 1.0,
              rng.gen::<f32>() * 2.0 - 1.0,
            )
            .normalize();

            let rotation = Quat::from_axis_angle(axis, angle);
            let direction = Mat3::from_quat(rotation) * cone_direction;

            Velocity::linear(direction * strength)
          }
          ParticleVelocity::None => Velocity::zero(),
        };

      let mut particle_entity = commands.spawn((
        ParticleBundle {
          particle: Particle {
            original_scale:   Vec3::ONE * emitter.descriptor.size,
            shrink_with_life: false,
          },
          material: emitter.descriptor.material.clone(),
          mesh: emitter.descriptor.shape.clone(),
          velocity,
          rigid_body: RigidBody::Dynamic,
          transform,
          lifetime: TimerLifetime::new(Duration::from_secs_f32(5.0)),
          ..default()
        },
        AdditionalMassProperties::Mass(0.1),
      ));
      let id = particle_entity.id();
      particle_entity.insert(Name::new(format!("particle_{:?}", id)));
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
