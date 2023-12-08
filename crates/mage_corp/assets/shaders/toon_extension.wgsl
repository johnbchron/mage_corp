#import bevy_pbr::{
  pbr_fragment::pbr_input_from_standard_material,
  pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
  prepass_io::{VertexOutput, FragmentOutput},
  pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
  forward_io::{VertexOutput, FragmentOutput},
  pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct ToonMaterial {
  quantize_steps: u32,
}

@group(1) @binding(100)
var<uniform> toon_material: ToonMaterial;

@fragment
fn fragment(
  in: VertexOutput,
  @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
  // generate a PbrInput struct from the StandardMaterial bindings
  var pbr_input = pbr_input_from_standard_material(in, is_front);

  // we can optionally modify the input before lighting and alpha_discard is applied
  // pbr_input.material.base_color.b = pbr_input.material.base_color.r;

  // alpha discard
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
  // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
  let out = deferred_output(in, pbr_input);
#else
  var out: FragmentOutput;
  // apply lighting
  out.color = apply_pbr_lighting(pbr_input);

  // we can optionally modify the lit color before post-processing is applied

  let luminance = dot(out.color.rgb, vec3(0.2126, 0.7152, 0.0722));
  let luminance_quantized = floor(luminance * f32(toon_material.quantize_steps)) / f32(toon_material.quantize_steps);
  out.color = vec4(out.color.rgb * luminance_quantized, out.color.a);
  

  // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
  // note this does not include fullscreen postprocessing effects like bloom.
  out.color = main_pass_post_lighting_processing(pbr_input, out.color);

  // we can optionally modify the final result here
  // out.color = out.color * 2.0;
#endif

  return out;
}
