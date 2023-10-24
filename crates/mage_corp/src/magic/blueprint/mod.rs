pub mod visuals;

use bevy::prelude::*;

#[derive(Component, Debug, Default, Reflect)]
pub struct Blueprint {
  _type: BlueprintType,
  stage: BlueprintStage,
}

impl Blueprint {
  pub fn new(_type: BlueprintType) -> Self {
    Self {
      _type,
      stage: BlueprintStage::Initialized,
    }
  }
}

#[derive(Debug, Default, Reflect)]
pub enum BlueprintType {
  #[default]
  MassBarrier,
}

#[derive(Debug, Default, Reflect)]
pub enum BlueprintStage {
  #[default]
  Initialized,
  Built,
}

pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<Blueprint>()
      .add_plugins(visuals::BlueprintVisualsPlugin);
  }
}
