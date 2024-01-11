use bevy::{
  asset::load_internal_asset,
  pbr::{ExtendedMaterial, MaterialExtension},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderRef},
};

#[allow(clippy::unreadable_literal)]
pub const OUTLINE_SHADER_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(12104443487162275386);

#[allow(clippy::unreadable_literal)]
pub const COLORS_SHADER_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(12104443487162275387);

// struct ToonMaterial {
//   luminance_bands:          u32,
//   luminance_power:          f32,
//   dither_factor:            f32,
//   outline_normal_color:     vec4<f32>,
//   outline_depth_color:      vec4<f32>,
//   outline_normal_threshold: f32,
//   outline_depth_threshold:  f32,
//   outline_scale:            f32,
//   far_plane_bleed:          f32,
// }

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct ToonExtension {
  // We need to ensure that the bindings of the base material and the extension
  // do not conflict, so we start from binding slot 100, leaving slots 0-99
  // for the base material.
  #[uniform(100)]
  pub luminance_bands:          f32,
  #[uniform(100)]
  pub luminance_power:          f32,
  #[uniform(100)]
  pub dither_factor:            f32,
  #[uniform(100)]
  pub outline_normal_color:     Color,
  #[uniform(100)]
  pub outline_depth_color:      Color,
  #[uniform(100)]
  pub outline_normal_threshold: f32,
  #[uniform(100)]
  pub outline_depth_threshold:  f32,
  #[uniform(100)]
  pub outline_scale:            f32,
  #[uniform(100)]
  pub far_plane_bleed:          f32,
}

impl Default for ToonExtension {
  fn default() -> Self {
    Self {
      luminance_bands:          8.0,
      luminance_power:          2.0,
      dither_factor:            5.0,
      outline_normal_color:     Color::rgb(1.2, 1.2, 1.2),
      outline_depth_color:      Color::rgb(0.5, 0.5, 0.5),
      outline_normal_threshold: 0.1,
      outline_depth_threshold:  0.05,
      outline_scale:            1.0,
      far_plane_bleed:          0.1,
    }
  }
}

impl MaterialExtension for ToonExtension {
  fn fragment_shader() -> ShaderRef { "shaders/toon_extension.wgsl".into() }
}

pub type ToonMaterial = ExtendedMaterial<StandardMaterial, ToonExtension>;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
  fn build(&self, app: &mut App) {
    load_internal_asset!(
      app,
      OUTLINE_SHADER_HANDLE,
      "../../../mage_corp/assets/shaders/outline.wgsl",
      Shader::from_wgsl
    );
    load_internal_asset!(
      app,
      COLORS_SHADER_HANDLE,
      "../../../mage_corp/assets/shaders/colors.wgsl",
      Shader::from_wgsl
    );

    app
      .add_plugins(MaterialPlugin::<ToonMaterial>::default())
      .register_asset_reflect::<ToonMaterial>();
  }
}
