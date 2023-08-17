use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, Reflect, Debug, Clone)]
#[uuid = "c5cb7df5-a1a2-4028-9a22-766824de2ba6"]
#[reflect(Default, Debug)]
pub struct ForceMaterial {
  #[uniform(0)]
  pub color: Color,
}

impl Default for ForceMaterial {
  fn default() -> Self {
    Self {
      color: Color::rgba(0.392, 0.584, 0.929, 0.2),
    }
  }
}

impl From<Color> for ForceMaterial {
  fn from(color: Color) -> Self {
    Self { color }
  }
}

impl Material for ForceMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/force_material.wgsl".into()
  }

  fn alpha_mode(&self) -> AlphaMode {
    AlphaMode::Blend
  }
}
