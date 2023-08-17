pub mod material;

use bevy::prelude::*;
pub use material::ForceMaterial;

pub struct ForceMaterialPlugin;

impl Plugin for ForceMaterialPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(MaterialPlugin::<ForceMaterial>::default())
      .register_asset_reflect::<ForceMaterial>();
  }
}
