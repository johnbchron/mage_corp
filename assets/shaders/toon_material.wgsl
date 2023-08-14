#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput
#import bevy_pbr::prepass_utils
#import bevy_pbr::utils               coords_to_viewport_uv
#import bevy_pbr::shadows as shadows
#import bevy_core_pipeline::tonemapping screen_space_dither

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

// var<private> sobel_x: array<f32, 9> = array<f32, 9>(
//   1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0,
// );

// var<private> sobel_y: array<f32, 9> = array<f32, 9>(
//   1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0,
// );

var<private> equidistant: array<f32, 9> = array<f32, 9>(
  0.707, 1.0, 0.707, 1.0, 0.0, 1.0, 0.707, 1.0, 0.707,
);

struct ToonMaterial {
  color: vec4<f32>,
  ambient_light: vec4<f32>,
  specular_color: vec4<f32>,
  rim_color: vec4<f32>,
  outline_normal_color: vec4<f32>,
  outline_depth_color: vec4<f32>,
  glossiness: f32,
  rim_power: f32,
  rim_threshold: f32,
  outline_scale: f32,
  outline_normal_threshold: f32,
  outline_depth_threshold: f32,
  shades: f32,
  shade_cutoff: f32,
  dither_strength: f32
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

// fn get_pixel_world_distance() -> f32 {
//   let camera_view = bevy_pbr::mesh_view_bindings::view;
//   
//   let camera_view_width = 2.0 / camera_view.view_proj[0][0];
//   let pixel_world_distance = camera_view_width / camera_view.viewport.z;
//   
//   return camera_view_width;
// }

fn detect_edge_depth(frag_coord: vec2<f32>, scale: f32) -> f32 {
  // this is to make up for the far field of the camera
  let depth_modulation = 0.001;
  
  var samples = array<f32, 9>();
  for (var i = 0; i < 9; i++) {
    let coords = frag_coord + neighbours[i] * scale;
    // this should be depth in world units
    samples[i] = (1.0 - get_depth(coords)) / depth_modulation;
  }
  
  var total = 0.0;
  for (var i = 0; i < 9; i++) {
    // get the difference in depth between the current pixel and the surrounding
    // pixels, and then weight it by the distance to the current pixel
    total += -(samples[4] - samples[i]) * equidistant[i];
  }
  // average over all 9
  total = total / 8.0;
  
  if total < material.outline_depth_threshold {
    return 0.0;
  }
  return total;
}

fn detect_edge_normal(frag_coord: vec2<f32>, scale: f32) -> f32 {
  // we weight based on how much the normal is facing the camera, so we need
  // the camera's view direction
  let camera_view = bevy_pbr::mesh_view_bindings::view;
  let camera_forward = normalize(vec3(camera_view.view[2][0], camera_view.view[2][1], camera_view.view[2][2]));
  
  var samples = array<vec3<f32>, 9>();
  for (var i = 0; i < 9; i++) {
    samples[i] = get_normal(frag_coord + neighbours[i] * scale);
  }

  var total = 0.0;
  for (var i = 0; i < 9; i++) {
    // we get first the difference between the current pixel and the sample,
    // and then weight it by the distance to the current pixel, and then
    // weight it by how much the current pixel is facing the camera
    total += (1.0 - dot(samples[4], samples[i])) / 2.0 * equidistant[i] * saturate(dot(samples[4], camera_forward) + 0.2);
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
  let light_intensity = saturate(dot(surface_normal, light_direction)) * shadow;
  let dither = (length(screen_space_dither(mesh.position.xy)) * 255.0 - 0.5) * 2.0 / 255.0;
  let dithered_light_intensity = light_intensity + dither * material.dither_strength;
  let smooth_light_intensity = floor(smoothstep(0.0, material.shade_cutoff, dithered_light_intensity) * material.shades) / material.shades;
  // let main_light_color = saturate(directional_light.color);
  let main_light_color = directional_light.color;
  let main_light = main_light_color * smooth_light_intensity;
  
  let camera_forward = normalize(vec3(camera_view.view[2][0], camera_view.view[2][1], camera_view.view[2][2]));
  
  // specular light
  let camera_half_forward = normalize(camera_forward + light_direction);
  let specular_intensity = saturate(dot(surface_normal, camera_half_forward));
  let specular_light_intensity = pow(specular_intensity * smooth_light_intensity, material.glossiness * material.glossiness);
  let smooth_specular_light_intensity = smoothstep(0.0, 0.025, specular_light_intensity);
  let specular_color = material.specular_color * smooth_specular_light_intensity;
  let specular_light = specular_color * smooth_light_intensity;
  
  // rim light
  let rim_intensity = saturate(1.0 - dot(surface_normal, camera_forward));
  let rim_light_intensity = rim_intensity * pow(light_intensity, material.rim_threshold);
  let smooth_rim_light_intensity = smoothstep(material.rim_power - 0.01, material.rim_power + 0.01, rim_light_intensity);
  let rim_light = material.rim_color * smooth_rim_light_intensity;
  
  let light = material.ambient_light + main_light + specular_light + rim_light;

  var texture_color: vec4<f32> = vec4(1.0);
  #ifdef VERTEX_UVS
  texture_color = textureSample(material_color_texture, material_color_texture_sampler, mesh.uv);
  #endif
  
  let unlit_color = texture_color * material.color;
  var color: vec4<f32> = unlit_color * light;
  
  if (material.outline_scale > 0.0) {
    let normal_edge = detect_edge_normal(mesh.position.xy, material.outline_scale / 2.0);
    let depth_edge = detect_edge_depth(mesh.position.xy, material.outline_scale);
    
    var edge: vec2<f32> = vec2(normal_edge, depth_edge);
    if (length(edge) > 0.0) {
      edge = normalize(edge);
    }
    let edge_color = edge.x * material.outline_normal_color + edge.y * material.outline_depth_color;
    
    let outline_stencil = mix(vec4(1.0), edge_color, length(edge));
    color = color * outline_stencil;
    // return vec4(edge, 0.0, 1.0);
  }
  
  return color;
}