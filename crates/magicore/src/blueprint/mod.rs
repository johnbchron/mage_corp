use bevy::prelude::*;

use super::{source::Source, spell::SourceLink, target::Target};

#[derive(Component, Clone, Reflect)]
pub struct ActiveBlueprint {
  descriptor: BlueprintDescriptor,
  stage:      BlueprintState,
}

impl ActiveBlueprint {
  pub fn new(descriptor: &BlueprintDescriptor) -> Self {
    Self {
      descriptor: descriptor.clone(),
      stage:      BlueprintState::Initialized { stored: 0.0 },
    }
  }
  pub fn saturated(&self) -> bool {
    match self.stage {
      BlueprintState::Active { deficit } => deficit <= 0.0,
      BlueprintState::Built { stored } => {
        stored >= self.descriptor.static_cost()
      }
      _ => false,
    }
  }
  pub fn spawn(
    self,
    commands: &mut Commands,
    source_link: &SourceLink,
    name: &str,
  ) -> Entity {
    commands
      .spawn(BlueprintBundle {
        active_bluep: self.clone(),
        target:       self.descriptor.base_target(),
        source_link:  source_link.clone(),
        name:         Name::from(name),
      })
      .id()
  }
  pub fn supply(&mut self, amount: f32) {
    match &mut self.stage {
      BlueprintState::Initialized { stored } => {
        *stored += amount;
        if *stored >= self.descriptor.static_cost() {
          self.stage = BlueprintState::Built { stored: *stored };
        }
      }
      BlueprintState::Built { stored } => {
        *stored += amount;
      }
      BlueprintState::Active { deficit } => {
        *deficit -= amount;
      }
    }
  }
  pub fn remaining(&self) -> f32 {
    match self.stage {
      BlueprintState::Initialized { stored } => {
        self.descriptor.static_cost() - stored
      }
      BlueprintState::Built { stored } => {
        self.descriptor.static_cost() - stored
      }
      BlueprintState::Active { deficit } => deficit,
    }
  }
}

#[derive(Clone, Reflect)]
pub enum BlueprintDescriptor {
  MassBarrier { target: Target, radius: f32 },
}

impl BlueprintDescriptor {
  pub fn static_cost(&self) -> f32 {
    match self {
      Self::MassBarrier { radius, .. } => 10.0 * radius.powi(2),
    }
  }
  pub fn base_target(&self) -> Target {
    match self {
      Self::MassBarrier { target, .. } => target.clone(),
    }
  }
}

#[derive(Clone, Reflect)]
enum BlueprintState {
  Initialized { stored: f32 },
  Built { stored: f32 },
  Active { deficit: f32 },
}

impl Default for BlueprintState {
  fn default() -> Self { Self::Initialized { stored: 0.0 } }
}

#[derive(Bundle)]
pub struct BlueprintBundle {
  pub active_bluep: ActiveBlueprint,
  pub target:       Target,
  pub source_link:  SourceLink,
  pub name:         Name,
}

pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<ActiveBlueprint>()
      .add_systems(Update, check_for_disconnected_bluep);
  }
}

fn check_for_disconnected_bluep(
  mut commands: Commands,
  source_q: Query<&Source>,
  bluep_q: Query<(Entity, &SourceLink), With<ActiveBlueprint>>,
) {
  for (entity, source_link) in bluep_q.iter() {
    if source_q.get(source_link.0).is_err() {
      error!(
        "Blueprint source entity was deleted or didn't exist, removing \
         blueprint entity"
      );
      commands.entity(entity).despawn_recursive();
    }
  }
}
