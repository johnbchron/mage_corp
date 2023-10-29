pub mod visuals;

use bevy::prelude::*;

#[derive(Component, Clone, Default, Reflect)]
pub struct ActiveBlueprint {
  descriptor: BlueprintDescriptor,
  stage:      BlueprintStage,
}

impl ActiveBlueprint {
  pub fn new(_type: BlueprintDescriptor) -> Self {
    Self {
      descriptor: _type,
      stage:      BlueprintStage::Initialized { stored: 0.0 },
    }
  }
  pub fn saturated(&self) -> bool {
    match self.stage {
      BlueprintStage::Active { deficit } => deficit <= 0.0,
      BlueprintStage::Built { stored } => {
        stored >= self.descriptor.initial_cost()
      }
      _ => false,
    }
  }
}

#[derive(Clone, Default, Reflect)]
pub enum BlueprintDescriptor {
  #[default]
  MassBarrier,
}

impl BlueprintDescriptor {
  pub fn initial_cost(&self) -> f32 {
    match self {
      Self::MassBarrier => 10.0,
    }
  }
}

#[derive(Clone, Reflect)]
pub enum BlueprintStage {
  Initialized { stored: f32 },
  Built { stored: f32 },
  Active { deficit: f32 },
}

impl Default for BlueprintStage {
  fn default() -> Self {
    Self::Initialized { stored: 0.0 }
  }
}

pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<ActiveBlueprint>()
      .add_plugins(visuals::BlueprintVisualsPlugin);
  }
}
