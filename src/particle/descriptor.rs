use std::time::Duration;

use bevy::prelude::*;

use crate::toon::ToonMaterial;

/// Describes the properties of emitted particles
#[derive(Default, Reflect)]
pub struct ParticleDescriptor {
  pub size:     f32,
  pub material: Handle<ToonMaterial>,
  pub shape:    Handle<Mesh>,
  pub behavior: ParticleBehavior,
}

/// Describes the behavior of emitted particles
#[derive(Reflect)]
pub struct ParticleBehavior {
  pub initial_linear_velocity:  ParticleLinearVelocity,
  pub initial_angular_velocity: ParticleAngularVelocity,
  pub acceleration:             ParticleAcceleration,
  pub contact_response:         ParticleContactResponseType,
  pub lifetime:                 Duration,
}

impl Default for ParticleBehavior {
  fn default() -> Self {
    Self {
      initial_linear_velocity:  ParticleLinearVelocity::default(),
      initial_angular_velocity: ParticleAngularVelocity::default(),
      acceleration:             ParticleAcceleration::default(),
      contact_response:         ParticleContactResponseType::default(),
      lifetime:                 Duration::from_secs(2),
    }
  }
}

/// Describes the initial velocity of emitted particles
#[derive(Reflect)]
pub enum ParticleLinearVelocity {
  SingleDirection {
    /// The direction emitted particles will travel. This will be normalized.
    direction: Vec3,
    /// The magnitude of the emitted particle's velocity.
    magnitude: f32,
  },
  Spherical {
    /// The magnitude of the emitted particle's velocity.
    magnitude: f32,
  },
  Conic {
    /// The angle of the cone.
    cone_angle: f32,
    /// The direction of the center of the cone. This will be normalized.
    direction:  Vec3,
    /// The magnitude of the emitted particle's velocity.
    magnitude:  f32,
  },
  None,
}

impl Default for ParticleLinearVelocity {
  fn default() -> Self {
    Self::Spherical { magnitude: 1.0 }
  }
}

/// Describes the initial velocity of emitted particles
#[derive(Default, Reflect)]
pub enum ParticleAngularVelocity {
  #[default]
  None,
}

/// Describes the acceleration acting on emitted particles
#[derive(Default, Reflect)]
pub enum ParticleAcceleration {
  #[default]
  None,
}

/// Describes the contact response of emitted particles
#[derive(Default, Reflect)]
pub enum ParticleContactResponseType {
  #[default]
  None,
}
