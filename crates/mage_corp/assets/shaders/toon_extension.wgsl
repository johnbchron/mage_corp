
#import bevy_pbr::{
  forward_io::{VertexOutput, FragmentOutput},
  pbr_fragment::pbr_input_from_standard_material,
  pbr_functions::{
    alpha_discard,
    PbrLightingOutput,
    generate_pbr_lighting,
    apply_pbr_lighting,
    main_pass_post_lighting_processing
  },
  view_transformations,
  mesh_view_bindings::view,
}
#import bevy_core_pipeline::tonemapping::screen_space_dither;

#import mage_corp::colors;

struct ToonMaterial {
  luminance_bands:          f32,
  luminance_power:          f32,
  dither_factor:            f32,
  outline_normal_color:     vec4<f32>,
  outline_depth_color:      vec4<f32>,
  outline_normal_threshold: f32,
  outline_depth_threshold:  f32,
  outline_scale:            f32,
  far_plane_bleed:          f32,
}

@group(1) @binding(100)
var<uniform> toon_material: ToonMaterial;

// https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
fn luminance(color: vec3<f32>) -> f32 {
  return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

fn apply_outlines(input_color: vec3<f32>, frag_pos: vec2<f32>) -> vec3<f32> {
  let normal_outline = mage_corp::outline::detect_edge_normal(frag_pos, toon_material.outline_scale);
  let depth_outline = mage_corp::outline::detect_edge_depth(frag_pos, toon_material.outline_scale);

  var outline_normal_intensity: f32 = step(toon_material.outline_normal_threshold, normal_outline);
  let outline_depth_intensity = step(toon_material.outline_depth_threshold, depth_outline);
  if outline_depth_intensity == 1.0 {
    outline_normal_intensity = 0.0;
  }

  let outline_normal_color = mix(vec3(1.0), toon_material.outline_normal_color.rgb, outline_normal_intensity);
  let outline_depth_color = mix(vec3(1.0), toon_material.outline_depth_color.rgb, outline_depth_intensity);
  let final_color = input_color * outline_normal_color * outline_depth_color;
  return final_color;
}

@fragment
fn fragment(
  in: VertexOutput,
  @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
  let far_half_space = view.frustum[5];
  let in_view_space = view_transformations::position_world_to_view(in.world_position.xyz);
  if dot(far_half_space, in.world_position) <= toon_material.far_plane_bleed * in_view_space.z {
    discard;
  }

  // generate a PbrInput struct from the StandardMaterial bindings
  var pbr_input = pbr_input_from_standard_material(in, is_front);
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

  // calculate vanilla pbr lighting
  var out: FragmentOutput;
  let pbr_output_color = apply_pbr_lighting(pbr_input);
  alpha_discard(pbr_input.material, pbr_output_color);

  // convert into oklch
  let oklch = colors::srgb2oklch(pbr_output_color.rgb);
  var luminance: f32 = oklch.x;
  var chroma: f32 = oklch.y;
  var hue: f32 = oklch.z;

  // apply world space dither to luminance
  let dither_factor = toon_material.dither_factor;
  let dither = length(screen_space_dither(in.position.xy)) * dither_factor;
  luminance += dither;

  // quantize luminance
  let bands = toon_material.luminance_bands;
  let power = toon_material.luminance_power;
  luminance = pow(floor(pow(luminance, 1.0 / power) * bands) / bands, power);

  // convert back to srgb and apply outlines
  let out_rgb = colors::oklch2srgb(vec3<f32>(luminance, chroma, hue));
  out.color = vec4(
    apply_outlines(out_rgb, in.position.xy),
    pbr_output_color.a
  );

  // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
  // note this does not include fullscreen postprocessing effects like bloom.
  out.color = main_pass_post_lighting_processing(pbr_input, out.color);
  return out;
}
