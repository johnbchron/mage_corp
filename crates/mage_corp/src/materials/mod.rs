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
  pub dark_threshold:      f32,
  #[uniform(100)]
  pub highlight_threshold: f32,
  #[uniform(100)]
  pub dark_color:          Color,
  #[uniform(100)]
  pub highlight_color:     Color,
  #[uniform(100)]
  pub blend_factor:        f32,
}

impl Default for ToonExtension {
  fn default() -> Self {
    Self {
      dark_threshold:      0.5,
      highlight_threshold: 6.0,
      dark_color:          Color::rgb(0.25, 0.25, 0.25),
      highlight_color:     Color::rgb(1.5, 1.5, 1.5),
      blend_factor:        0.01,
    }
  }
}

impl MaterialExtension for ToonExtension {
  fn fragment_shader() -> ShaderRef {
    "shaders/toon_extension.wgsl".into()
  }

  fn deferred_fragment_shader() -> ShaderRef {
    "shaders/toon_extension.wgsl".into()
  }
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
