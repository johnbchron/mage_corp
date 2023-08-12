#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput
#import bevy_pbr::prepass_utils
#import bevy_pbr::utils               coords_to_viewport_uv
#import bevy_pbr::shadows as shadows

var<private> neighbours: array<vec2<f32>, 9> = array<vec2<f32>, 9>(
  vec2<f32>(-1.0, 1.0),  // 0. top left
  vec2<f32>(0.0, 1.0),   // 1. top center
  vec2<f32>(1.0, 1.0),   // 2. top right
  vec2<f32>(-1.0, 0.0),  // 3. center left
  vec2<f32>(0.0, 0.0),   // 4. center center
  vec2<f32>(1.0, 0.0),   // 5. center right
  vec2<f32>(-1.0, -1.0), // 6. bottom left
  vec2<f32>(0.0, -1.0),  // 7. bottom center
  vec2<f32>(1.0, -1.0),  // 8. bottom right
);

var<private> sobel_x: array<f32, 9> = array<f32, 9>(
  1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0,
);

var<private> sobel_y: array<f32, 9> = array<f32, 9>(
  1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0,
);

var<private> equidistant: array<f32, 9> = array<f32, 9>(
  0.707, 1.0, 0.707, 1.0, 0.0, 1.0, 0.707, 1.0, 0.707,
);

struct ToonMaterial {
  color: vec4<f32>,
  ambient_light: vec4<f32>,
  specular_color: vec4<f32>,
  rim_color: vec4<f32>,
  outline_color: vec4<f32>,
  glossiness: f32,
  rim_power: f32,
  rim_threshold: f32,
  outline_scale: f32,
  outline_normal_threshold: f32,
  outline_depth_threshold: f32,
}

@group(1) @binding(0)
var<uniform> material: ToonMaterial;
@group(1) @binding(1)
var material_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var material_color_texture_sampler: sampler;

fn get_depth(pos: vec2<f32>) -> f32 {
  return bevy_pbr::prepass_utils::prepass_depth(vec4(pos, 0.0, 0.0), 0u);
}

fn get_normal(pos: vec2<f32>) -> vec3<f32> {
  return bevy_pbr::prepass_utils::prepass_normal(vec4(pos, 0.0, 0.0), 0u);
}

fn get_pixel_world_distance() -> f32 {
  let camera_view = bevy_pbr::mesh_view_bindings::view;
  
  let camera_view_width = 2.0 / camera_view.view_proj[0][0];
  let pixel_world_distance = camera_view_width / camera_view.viewport.z;
  
  return camera_view_width;
}

fn detect_edge_depth(frag_coord: vec2<f32>, scale: f32) -> f32 {
  let depth_modulation = 0.001;
  
  var samples = array<f32, 9>();
  for (var i = 0; i < 9; i++) {
    let coords = frag_coord + neighbours[i] * scale;
    // this should be depth in world units
    samples[i] = (1.0 - get_depth(coords)) / depth_modulation;
  }
  
  var total = 0.0;
  for (var i = 0; i < 9; i++) {
    total += -(samples[4] - samples[i]) * equidistant[i];
  }
  total = total / 8.0;
  
  if total < material.outline_depth_threshold {
    return 0.0;
  }
  return total;
}

fn detect_edge_normal(frag_coord: vec2<f32>, scale: f32) -> f32 {
  var samples = array<vec3<f32>, 9>();
  for (var i = 0; i < 9; i++) {
    samples[i] = get_normal(frag_coord + neighbours[i] * scale);
  }

  var total = 0.0;
  for (var i = 0; i < 9; i++) {
    total += (1.0 - dot(samples[4], samples[i])) / 2.0 * equidistant[i];
  }
  total = total / 8.0;
  
  if total < material.outline_normal_threshold {
    return 0.0;
  }
  return total;
}

@fragment
fn fragment(
  mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {
  let surface_normal = get_normal(mesh.position.xy);
  
  let directional_light = bevy_pbr::mesh_view_bindings::lights.directional_lights[0];
  let camera_view = bevy_pbr::mesh_view_bindings::view;
  
  let view_z = dot(vec4<f32>(
    camera_view.inverse_view[0].z,
    camera_view.inverse_view[1].z,
    camera_view.inverse_view[2].z,
    camera_view.inverse_view[3].z
  ), mesh.world_position);
  let shadow = shadows::fetch_directional_shadow(0u, mesh.world_position, mesh.world_normal, view_z);
  
  // main light
  let light_direction = directional_light.direction_to_light;
  let light_intensity = clamp(dot(surface_normal, light_direction), 0.0, 1.0) * shadow;
  let smooth_light_intensity = smoothstep(0.0, 0.025, light_intensity);
  let main_light_color = min(directional_light.color, vec4<f32>(1.0));
  let main_light = main_light_color * smooth_light_intensity;
  
  let camera_forward = normalize(vec3(camera_view.view[2][0], camera_view.view[2][1], camera_view.view[2][2]));
  
  // specular light
  let camera_half_forward = normalize(camera_forward + light_direction);
  let specular_intensity = clamp(dot(surface_normal, camera_half_forward), 0.0, 1.0);
  let specular_light_intensity = pow(specular_intensity * smooth_light_intensity, material.glossiness * material.glossiness);
  let smooth_specular_light_intensity = smoothstep(0.0, 0.025, specular_light_intensity);
  let specular_color = material.specular_color * smooth_specular_light_intensity;
  let specular_light = specular_color * smooth_light_intensity;
  
  // rim light
  let rim_intensity = clamp(1.0 - dot(surface_normal, camera_forward), 0.0, 1.0);
  let rim_light_intensity = rim_intensity * pow(light_intensity, material.rim_threshold);
  let smooth_rim_light_intensity = smoothstep(material.rim_power - 0.01, material.rim_power + 0.01, rim_light_intensity);
  let rim_light = material.rim_color * smooth_rim_light_intensity;
  
  let light = material.ambient_light + main_light + specular_light + rim_light;
  
  let depth_edge = detect_edge_depth(mesh.position.xy, material.outline_scale);
  let normal_edge = detect_edge_normal(mesh.position.xy, material.outline_scale / 2.0);
  
  let edge = min(max(step(material.outline_depth_threshold, depth_edge), step(material.outline_normal_threshold, normal_edge)), 1.0);
  let outline_stencil = 1.0 - ((1.0 - material.outline_color) * edge);

  var texture_color: vec4<f32> = vec4(1.0);
  #ifdef VERTEX_UVS
  texture_color = textureSample(material_color_texture, material_color_texture_sampler, mesh.uv);
  #endif
  
  let unlit_color = texture_color * material.color;
  var color: vec4<f32> = unlit_color * light;
  color = color * outline_stencil;
  // color = mix(color, material.outline_color, edge);
  
  // return vec4(depth_edge, normal_edge, 0.0, 1.0);
  return color;
}