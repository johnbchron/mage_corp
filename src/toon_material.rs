use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, Reflect, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e1"]
pub struct ToonMaterial {
  #[uniform(0)]
  pub color:          Color,
  #[uniform(1)]
  pub ambient_light:  Color,
  #[uniform(2)]
  pub specular_color: Color,
  #[uniform(3)]
  pub rim_color:      Color,
  #[uniform(4)]
  pub outline_color:  Color,
  #[uniform(5)]
  pub specular_power: f32,
  #[uniform(6)]
  pub rim_power:      f32,
  #[uniform(7)]
  pub rim_threshold:  f32,
  #[uniform(8)]
  pub outline_scale:  f32,
}

impl Material for ToonMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/toon_material.wgsl".into()
  }

  fn alpha_mode(&self) -> AlphaMode {
    AlphaMode::Opaque
  }
}

impl Default for ToonMaterial {
  fn default() -> Self {
    Self {
      // cornflower blue
      color:          Color::rgb(0.392, 0.584, 0.929),
      ambient_light:  Color::rgb(0.4, 0.4, 0.4),
      specular_color: Color::rgb(1.0, 1.0, 1.0),
      rim_color:      Color::rgb(1.0, 1.0, 1.0),
      outline_color:  Color::rgb(0.0, 0.0, 0.0),
      specular_power: 32.0,
      rim_power:      0.712,
      rim_threshold:  0.1,
      outline_scale:  0.5,
    }
  }
}

impl From<Color> for ToonMaterial {
  fn from(color: Color) -> Self {
    Self {
      color,
      ..Default::default()
    }
  }
}
