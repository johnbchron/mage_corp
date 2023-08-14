use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::render_resource::{AsBindGroup, ShaderRef},
};

use super::ConvertToToonMaterial;

#[derive(AsBindGroup, TypeUuid, Reflect, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e1"]
#[reflect(Default, Debug)]
pub struct ToonMaterial {
  #[uniform(0)]
  pub color:                    Color,
  #[uniform(0)]
  pub ambient_light:            Color,
  #[uniform(0)]
  pub specular_color:           Color,
  #[uniform(0)]
  pub rim_color:                Color,
  #[uniform(0)]
  pub outline_normal_color:            Color,
  #[uniform(0)]
  pub outline_depth_color:            Color,
  #[uniform(0)]
  pub specular_power:           f32,
  #[uniform(0)]
  pub rim_power:                f32,
  #[uniform(0)]
  pub rim_threshold:            f32,
  #[uniform(0)]
  pub outline_scale:            f32,
  #[uniform(0)]
  pub outline_normal_threshold: f32,
  #[uniform(0)]
  pub outline_depth_threshold:  f32,
  #[uniform(0)]
  pub shades:                   f32,
  #[uniform(0)]
  pub shade_cutoff:             f32,
  #[uniform(0)]
  pub dither_strength:          f32,
  #[texture(1)]
  #[sampler(2)]
  pub color_texture:            Option<Handle<Image>>,
  pub alpha_mode:               AlphaMode,
}

impl Material for ToonMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/toon_material.wgsl".into()
  }

  fn alpha_mode(&self) -> AlphaMode {
    self.alpha_mode
    // AlphaMode::Opaque
  }

  fn prepass_fragment_shader() -> ShaderRef {
    StandardMaterial::prepass_fragment_shader()
  }
}

impl Default for ToonMaterial {
  fn default() -> Self {
    Self {
      // cornflower blue
      color:                    Color::rgb(0.392, 0.584, 0.929),
      color_texture:            None,
      ambient_light:            Color::rgb(0.6, 0.6, 0.6),
      specular_color:           Color::rgb(1.0, 1.0, 1.0),
      rim_color:                Color::rgb(1.0, 1.0, 1.0),
      outline_normal_color:     Color::rgb(1.2, 1.2, 1.2),
      outline_depth_color:      Color::rgb(0.2, 0.2, 0.2),
      specular_power:           32.0,
      rim_power:                0.712,
      rim_threshold:            0.1,
      outline_scale:            1.0,
      outline_normal_threshold: 0.1,
      outline_depth_threshold:  0.1,
      shades:                   2.0,
      shade_cutoff:             0.15,
      dither_strength:          0.0,
      alpha_mode:               AlphaMode::Opaque,
    }
  }
}

impl From<Color> for ToonMaterial {
  fn from(color: Color) -> Self {
    Self {
      color,
      alpha_mode: if color.a() < 1.0 {
        AlphaMode::Blend
      } else {
        AlphaMode::Opaque
      },
      ..Default::default()
    }
  }
}

impl From<Handle<Image>> for ToonMaterial {
  fn from(texture: Handle<Image>) -> Self {
    ToonMaterial {
      color_texture: Some(texture),
      ..Default::default()
    }
  }
}

impl From<&StandardMaterial> for ToonMaterial {
  fn from(std_material: &StandardMaterial) -> ToonMaterial {
    let specular: f32 =
      16.0 * 2.0_f32.powf(3.0 * (-std_material.reflectance + 0.5));
    ToonMaterial {
      color: std_material.base_color,
      color_texture: std_material.base_color_texture.clone(),
      specular_power: specular,
      rim_power: std_material.perceptual_roughness.sqrt().clamp(0.5, 1.0),
      alpha_mode: std_material.alpha_mode,
      ..default()
    }
  }
}

impl ToonMaterial {
  pub fn with_settings(
    self,
    settings: &ConvertToToonMaterial,
  ) -> Self {
    let mut new = self;
    if let Some(outline_scale) = settings.outline_scale {
      new.outline_scale = outline_scale;
    }
    if let Some(outline_normal_color) = settings.outline_normal_color {
      new.outline_normal_color = outline_normal_color;
    }
    if let Some(outline_depth_color) = settings.outline_depth_color {
      new.outline_depth_color = outline_depth_color;
    }
    new
  }
}
