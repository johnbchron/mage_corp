
#define_import_path mage_corp::outline

#import bevy_pbr::prepass_utils::{prepass_depth, prepass_normal}

fn get_depth(pos: vec2<f32>) -> f32 {
  let raw_depth = prepass_depth(vec4(pos, 0.0, 0.0), 0u);
  return bevy_pbr::view_transformations::depth_ndc_to_view_z(raw_depth);
}

fn get_normal(pos: vec2<f32>) -> vec3<f32> {
  return prepass_normal(vec4(pos, 0.0, 0.0), 0u);
}

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

var<private> equidistant: array<f32, 9> = array<f32, 9>(
  0.707, 1.0, 0.707, 1.0, 0.0, 1.0, 0.707, 1.0, 0.707,
);

fn detect_edge_depth(frag_coord: vec2<f32>, scale: f32) -> f32 {
  var samples = array<f32, 9>();
  for (var i = 0; i < 9; i++) {
    let coords = frag_coord + neighbours[i] * scale;
    // this should be depth in world units
    samples[i] = get_depth(coords);
  }
  
  var total = 0.0;
  for (var i = 0; i < 9; i++) {
    let difference = -(samples[4] - samples[i]);
    let percent_difference = difference / samples[4];
    // get the difference in depth between the current pixel and the surrounding
    // pixels, and then weight it by the distance to the current pixel
    total += percent_difference * equidistant[i];
  }
  // average over all 9
  total = total / 8.0;
  
  return min(total, 1.0);
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
    
    // difference in direction between center pixel and current pixel
    // from 0, identical, to 1, opposite
    let pixel_direction_difference = (1.0 - dot(samples[i], camera_forward)) / 2.0;
    let facing_camera = pow(saturate(dot(normalize(samples[4] + samples[i]), camera_forward)), 1.4);
    total += pixel_direction_difference * equidistant[i] * facing_camera;
  }
  total = total / 8.0;
  
  return min(total, 1.0);
}
