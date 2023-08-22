use bevy::reflect::TypeUuid;
pub mod force;
pub mod toon;

pub const PREPASS_SHADER_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12104443487162275386);

use bevy::{asset::load_internal_asset, prelude::*};

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(bevy_shader_utils::ShaderUtilsPlugin);

    load_internal_asset!(
      app,
      PREPASS_SHADER_HANDLE,
      "../../assets/shaders/prepass.wgsl",
      Shader::from_wgsl
    );

    app
      .add_plugins(force::ForceMaterialPlugin)
      .add_plugins(toon::ToonMaterialPlugin);
  }
}
