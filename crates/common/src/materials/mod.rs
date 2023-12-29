use bevy::{
  asset::load_internal_asset,
  pbr::{ExtendedMaterial, MaterialExtension},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderRef},
};

#[allow(clippy::unreadable_literal)]
pub const OUTLINE_SHADER_HANDLE: Handle<Shader> =
  Handle::weak_from_u128(12104443487162275386);

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct ToonExtension {
  // We need to ensure that the bindings of the base material and the extension
  // do not conflict, so we start from binding slot 100, leaving slots 0-99
  // for the base material.
  #[uniform(100)]
  pub dark_two_threshold:       f32,
  #[uniform(100)]
  pub normal_threshold:         f32,
  #[uniform(100)]
  pub highlight_threshold:      f32,
  #[uniform(100)]
  pub dark_one_color:           Color,
  #[uniform(100)]
  pub dark_two_color:           Color,
  #[uniform(100)]
  pub normal_color:             Color,
  #[uniform(100)]
  pub highlight_color:          Color,
  #[uniform(100)]
  pub blend_factor:             f32,
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
    let warm_color = Color::rgb(1.0, 0.9, 0.8);
    let cool_color = Color::rgb(0.8, 0.9, 1.0);
    Self {
      dark_two_threshold:       0.2,
      normal_threshold:         1.0,
      highlight_threshold:      3.0,
      dark_one_color:           cool_color * 0.5,
      dark_two_color:           warm_color * 0.75,
      normal_color:             Color::rgb(1.0, 1.0, 1.0),
      highlight_color:          warm_color,
      blend_factor:             0.01,
      outline_normal_color:     warm_color * 1.2,
      outline_depth_color:      cool_color * 0.5,
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

    app
      .add_plugins(MaterialPlugin::<ToonMaterial>::default())
      .register_asset_reflect::<ToonMaterial>();
  }
}
