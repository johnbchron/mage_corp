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
#[derive(Default, Reflect)]
pub struct ParticleBehavior {
  pub initial_velocity: ParticleVelocity,
  pub acceleration:     ParticleAcceleration,
  pub contact_response: ParticleContactResponseType,
}

/// Describes the initial velocity of emitted particles
#[derive(Reflect)]
pub enum ParticleVelocity {
  SingleDirection {
    /// The direction emitted particles will travel. This will be normalized.
    direction: Vec3,
    /// The strength with which the particle will exit.
    magnitude:  f32,
  },
  Spherical {
    /// The strength with which the particle will exit.
    magnitude: f32,
  },
  Conic {
    /// The angle of the cone.
    cone_angle:     f32,
    /// The direction of the center of the cone. This will be normalized.
    direction: Vec3,
    /// The strength with which the particle will exit.
    magnitude:       f32,
  },
  None,
}

impl Default for ParticleVelocity {
  fn default() -> Self {
    Self::Spherical {
      magnitude: 1.0,
    }
  }
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
