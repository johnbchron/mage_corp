use bevy::prelude::*;

use crate::toon::ToonMaterial;

#[derive(Default, Reflect)]
pub struct ParticleDescriptor {
  pub size:     f32,
  pub material: Handle<ToonMaterial>,
  pub shape:    Handle<Mesh>,
  pub behavior: ParticleBehavior,
}

#[derive(Default, Reflect)]
pub struct ParticleBehavior {
  pub initial_velocity: ParticleVelocity,
  pub acceleration:     ParticleAcceleration,
  pub contact_response: ParticleContactResponseType,
}

#[derive(Reflect)]
pub enum ParticleVelocity {
  SingleDirection {
    /// The direction the smoke will travel. This will be normalized.
    direction: Vec3,
    /// The strength with which the particle will exit.
    strength:  f32,
  },
  Spherical {
    /// The strength with which the particle will exit.
    strength: f32,
  },
  Conic {
    /// The angle of the cone.
    cone_angle:     f32,
    /// The direction of the center of the cone. This will be normalized.
    #[reflect(ignore)]
    cone_direction: Vec3,
    /// The strength with which the particle will exit.
    #[reflect(ignore)]
    strength:       f32,
  },
  None,
}

impl Default for ParticleVelocity {
  fn default() -> Self {
    Self::Spherical {
      strength: 1.0,
    }
  }
}

#[derive(Default, Reflect)]
pub enum ParticleAcceleration {
  #[default]
  None,
}

#[derive(Default, Reflect)]
pub enum ParticleContactResponseType {
  #[default]
  None,
}
