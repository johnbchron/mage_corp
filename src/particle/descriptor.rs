use bevy::prelude::*;

use crate::{toon::ToonMaterial, utils::static_or_closure::StaticOrClosure};

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
  pub acceleration:      ParticleAcceleration,
  pub contact_response: ParticleContactResponseType,
}

#[derive(Reflect)]
pub enum ParticleVelocity {
  SingleDirection {
    /// The direction the smoke will travel. This will be normalized.
    #[reflect(ignore)]
    direction: StaticOrClosure<Vec3>,
    /// The strength with which the particle will exit.
    #[reflect(ignore)]
    strength:  StaticOrClosure<f32>,
  },
  Spherical {
    /// The strength with which the particle will exit.
    #[reflect(ignore)]
    strength: StaticOrClosure<f32>,
  },
  Conic {
    /// The angle of the cone.
    #[reflect(ignore)]
    cone_angle:     StaticOrClosure<f32>,
    /// The direction of the center of the cone. This will be normalized.
    #[reflect(ignore)]
    cone_direction: StaticOrClosure<Vec3>,
    /// The strength with which the particle will exit.
    #[reflect(ignore)]
    strength:       StaticOrClosure<f32>,
  },
  None,
}

impl Default for ParticleVelocity {
  fn default() -> Self {
    Self::Spherical {
      strength: StaticOrClosure::<f32>::Static(1.0),
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
