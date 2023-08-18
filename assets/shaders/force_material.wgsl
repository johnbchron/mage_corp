#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_view_bindings globals
#import bevy_pbr::mesh_vertex_output MeshVertexOutput

#import mage_corp::prepass

#import bevy_shader_utils::simplex_noise_3d simplex_noise_3d

struct ForceMaterial {
  color: vec4<f32>,
  alpha_min: f32,
  alpha_max: f32,
  influence: f32
};

@group(1) @binding(0)
var<uniform> material: ForceMaterial;
@group(1) @binding(1)
var<storage> contact_points: array<vec4<f32>>;

fn compute_closest_contact_point(world_pos: vec3<f32>) -> f32 {
  var closest_contact_point_distance: f32 = 1.0;
  for (var i = 0u; i < arrayLength(&contact_points); i++) {
    let contact_point_distance = distance(contact_points[i].xyz, world_pos.xyz);
    if (contact_point_distance < closest_contact_point_distance) {
      closest_contact_point_distance = contact_point_distance;
    }
  }
  return closest_contact_point_distance;
}

@fragment
fn fragment(
  mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {
  let surface_normal = mesh.world_normal;
  
  let camera_view = bevy_pbr::mesh_view_bindings::view;
  let camera_forward = normalize(vec3(camera_view.view[2][0], camera_view.view[2][1], camera_view.view[2][2]));
  
  let closest_contact_point_distance = compute_closest_contact_point(mesh.world_position.xyz);
  
  var noise = simplex_noise_3d(mesh.world_position.xyz * 4.0 + globals.time / 4.0) + sin(mesh.world_position.x * 4.0 + mesh.world_position.z * 4.0) * 0.5;
  
  let NdV_intensity = 1.0 - saturate(abs(dot(surface_normal, camera_forward)));
  let cp_intensity = pow(1.0 - saturate(closest_contact_point_distance), material.influence * 2.0);
  var noise_intensity = noise * 0.4;
  
  let intensity = saturate(NdV_intensity + cp_intensity + noise_intensity);
  let color = mix(material.color.xyz, sqrt(material.color.xyz), cp_intensity);
  
  let alpha = mix(material.alpha_min, material.alpha_max, pow(intensity, material.influence));
  
  return vec4(color, alpha);
}