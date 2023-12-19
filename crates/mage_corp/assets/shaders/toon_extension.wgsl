#import bevy_pbr::{
  pbr_fragment::pbr_input_from_standard_material,
  pbr_functions::alpha_discard,
}

#import bevy_pbr::{
  forward_io::{VertexOutput, FragmentOutput},
  pbr_functions::{PbrLightingOutput, generate_pbr_lighting, main_pass_post_lighting_processing},
  view_transformations,
  mesh_view_bindings::view,
}

struct ToonMaterial {
  dark_two_threshold: f32,
  regular_threshold: f32,
  highlight_threshold: f32,
  dark_one_color: vec4<f32>,
  dark_two_color: vec4<f32>,
  regular_color: vec4<f32>,
  highlight_color: vec4<f32>,
  blend_factor: f32,
  far_bleed: f32,
}

@group(1) @binding(100)
var<uniform> toon_material: ToonMaterial;

// https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
fn luminance(color: vec3<f32>) -> f32 {
  return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

@fragment
fn fragment(
  in: VertexOutput,
  @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
  let far_half_space = view.frustum[5];
  let in_view_space = view_transformations::position_world_to_view(in.world_position.xyz);
  if dot(far_half_space, in.world_position) <= toon_material.far_bleed * in_view_space.z {
    discard;
  }

  // generate a PbrInput struct from the StandardMaterial bindings
  var pbr_input = pbr_input_from_standard_material(in, is_front);
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

  var out: FragmentOutput;
  let pbr_lighting = generate_pbr_lighting(pbr_input);

  let direct_light: vec3<f32> = pbr_lighting.direct_light;
  let indirect_light: vec3<f32> = pbr_lighting.indirect_light;
  let emissive_light: vec3<f32> = pbr_lighting.emissive_light;
  let base_color = pbr_input.material.base_color.rgb;

  let direct_light_luminance = luminance(direct_light + indirect_light);

  let blend = toon_material.blend_factor;
  let dark_two_threshold = toon_material.dark_two_threshold;
  let regular_threshold = toon_material.regular_threshold;
  let highlight_threshold = toon_material.highlight_threshold;

  let dark_one_color = toon_material.dark_one_color.rgb;
  let dark_two_color = toon_material.dark_two_color.rgb;
  let regular_color = toon_material.regular_color.rgb;
  let highlight_color = toon_material.highlight_color.rgb;

  let dark_one_intensity = 1.0;
  let dark_two_intensity = smoothstep(dark_two_threshold, dark_two_threshold + blend, direct_light_luminance);
  let regular_intensity = smoothstep(regular_threshold, regular_threshold + blend, direct_light_luminance);
  let highlight_intensity = smoothstep(highlight_threshold, highlight_threshold + blend, direct_light_luminance);

  let dark_one_light = dark_one_color * dark_one_intensity * base_color;
  let dark_two_light = dark_two_color * dark_two_intensity * base_color;
  let regular_light = regular_color * regular_intensity * base_color;
  let highlight_light = highlight_color * highlight_intensity * base_color;

  let toon_light = max(max(dark_one_light, dark_two_light), regular_light) + highlight_light;

  out.color = vec4(
    toon_light +
    emissive_light,
    pbr_lighting.alpha
  );

  // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
  // note this does not include fullscreen postprocessing effects like bloom.
  out.color = main_pass_post_lighting_processing(pbr_input, out.color);

  return out;
}
