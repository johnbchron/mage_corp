#import bevy_pbr::{
  pbr_fragment::pbr_input_from_standard_material,
  pbr_functions::alpha_discard,
}

#import bevy_pbr::{
  forward_io::{VertexOutput, FragmentOutput},
  pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
  view_transformations,
  mesh_view_bindings::view,
}

struct ToonMaterial {
  dark_threshold: f32,
  highlight_threshold: f32,
  dark_color: vec4<f32>,
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
    // discard;
  }

  // generate a PbrInput struct from the StandardMaterial bindings
  var pbr_input = pbr_input_from_standard_material(in, is_front);

  // we can optionally modify the input before lighting and alpha_discard is applied
  // pbr_input.material.base_color.b = pbr_input.material.base_color.r;

  // alpha discard
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

  var out: FragmentOutput;
  // apply lighting
  out.color = apply_pbr_lighting(pbr_input);

  // here is our actual toon shading logic
  
  let dark_luminance_threshold = luminance(pbr_input.material.base_color.rgb * toon_material.dark_threshold);
  let highlight_luminance_threshold = luminance(pbr_input.material.base_color.rgb * toon_material.highlight_threshold);

  let dark_color = vec4(pbr_input.material.base_color.rgb, out.color.a) * toon_material.dark_color;
  let normal_color = vec4(pbr_input.material.base_color.rgb, out.color.a);
  let highlight_color = vec4(pbr_input.material.base_color.rgb, out.color.a) * toon_material.highlight_color;

  let luminance = luminance(out.color.rgb);
  // if luminance < dark_luminance_threshold {
  //   out.color = dark_color;
  // } else if luminance > highlight_luminance_threshold {
  //   out.color = highlight_color;
  // } else {
  //   out.color = normal_color;
  // }

  // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
  // note this does not include fullscreen postprocessing effects like bloom.
  out.color = main_pass_post_lighting_processing(pbr_input, out.color);

  // we can optionally modify the final result here
  // out.color = out.color * 2.0;

  return out;
}
