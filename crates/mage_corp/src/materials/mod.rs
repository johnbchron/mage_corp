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
  pub quantize_steps: u32,
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
    app.add_plugins(MaterialPlugin::<ToonMaterial>::default());
  }
}
