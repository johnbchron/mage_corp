use bevy::{
  pbr::{ExtendedMaterial, MaterialExtension},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct ToonExtension {
  // We need to ensure that the bindings of the base material and the extension
  // do not conflict, so we start from binding slot 100, leaving slots 0-99
  // for the base material.
  #[uniform(100)]
  pub dark_two_threshold:  f32,
  #[uniform(100)]
  pub normal_threshold:    f32,
  #[uniform(100)]
  pub highlight_threshold: f32,
  #[uniform(100)]
  pub dark_one_color:      Color,
  #[uniform(100)]
  pub dark_two_color:      Color,
  #[uniform(100)]
  pub normal_color:        Color,
  #[uniform(100)]
  pub highlight_color:     Color,
  #[uniform(100)]
  pub blend_factor:        f32,
  #[uniform(100)]
  pub far_bleed:           f32,
}

impl Default for ToonExtension {
  fn default() -> Self {
    Self {
      dark_two_threshold:  0.2,
      normal_threshold:    1.0,
      highlight_threshold: 3.0,
      dark_one_color:      Color::rgb(0.3, 0.3, 0.3),
      dark_two_color:      Color::rgb(0.8, 0.8, 0.8),
      normal_color:        Color::rgb(1.0, 1.0, 1.0),
      highlight_color:     Color::rgb(1.0, 1.0, 1.0),
      blend_factor:        0.01,
      far_bleed:           0.1,
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
    app
      .add_plugins(MaterialPlugin::<ToonMaterial>::default())
      .register_asset_reflect::<ToonMaterial>();
  }
}
