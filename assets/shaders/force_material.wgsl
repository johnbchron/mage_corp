#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_vertex_output MeshVertexOutput

#import mage_corp::prepass

struct ForceMaterial {
  color: vec4<f32>,
  alpha_min: f32,
  alpha_max: f32,
  influence: f32
};

@group(1) @binding(0)
var<uniform> material: ForceMaterial;

@fragment
fn fragment(
  mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {
  let surface_normal = mesh.world_normal;
  
  let camera_view = bevy_pbr::mesh_view_bindings::view;
  let camera_forward = normalize(vec3(camera_view.view[2][0], camera_view.view[2][1], camera_view.view[2][2]));
  
  let intensity = pow(1.0 - abs(dot(surface_normal, camera_forward)), material.influence);
  
  let alpha = mix(material.alpha_min, material.alpha_max, intensity);
  
  return vec4(material.color.xyz, alpha);
}