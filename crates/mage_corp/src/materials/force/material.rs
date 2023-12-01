use bevy::{
  pbr::{MaterialPipeline, MaterialPipelineKey},
  prelude::*,
  reflect::TypeUuid,
  render::{
    mesh::MeshVertexBufferLayout,
    render_resource::{
      AsBindGroup, RenderPipelineDescriptor, ShaderRef,
      SpecializedMeshPipelineError,
    },
  },
};

#[derive(AsBindGroup, TypeUuid, Asset, Reflect, Debug, Clone)]
#[uuid = "c5cb7df5-a1a2-4028-9a22-766824de2ba6"]
#[reflect(Default, Debug)]
pub struct ForceMaterial {
  #[uniform(0)]
  pub color:          Color,
  #[uniform(0)]
  pub alpha_min:      f32,
  #[uniform(0)]
  pub alpha_max:      f32,
  #[uniform(0)]
  pub influence:      f32,
  #[storage(1, read_only)]
  pub contact_points: Vec<[f32; 4]>,
  pub alpha_mode:     AlphaMode,
}

impl Default for ForceMaterial {
  fn default() -> Self {
    Self {
      color:          Color::rgb(0.392, 0.584, 0.929),
      alpha_min:      0.05,
      alpha_max:      0.5,
      influence:      5.0,
      contact_points: vec![],
      alpha_mode:     AlphaMode::Blend,
    }
  }
}

impl From<Color> for ForceMaterial {
  fn from(color: Color) -> Self {
    Self { color, ..default() }
  }
}

impl Material for ForceMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/force_material.wgsl".into()
  }

  fn alpha_mode(&self) -> AlphaMode {
    self.alpha_mode
  }

  fn specialize(
    _pipeline: &MaterialPipeline<Self>,
    descriptor: &mut RenderPipelineDescriptor,
    _layout: &MeshVertexBufferLayout,
    _key: MaterialPipelineKey<Self>,
  ) -> Result<(), SpecializedMeshPipelineError> {
    descriptor.primitive.cull_mode = None;
    Ok(())
  }
}
