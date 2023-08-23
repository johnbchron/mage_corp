#define_import_path mage_corp::prepass

#import bevy_pbr::prepass_utils

fn get_depth(pos: vec2<f32>) -> f32 {
  return bevy_pbr::prepass_utils::prepass_depth(vec4(pos, 0.0, 0.0), 0u);
}

fn get_normal(pos: vec2<f32>) -> vec3<f32> {
  return bevy_pbr::prepass_utils::prepass_normal(vec4(pos, 0.0, 0.0), 0u);
}