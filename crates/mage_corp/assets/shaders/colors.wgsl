
#define_import_path mage_corp::colors

//////////////////////////////
// https://github.com/patriciogonzalezvivo/lygia/blob/main/color/space/rgb2oklab.wgsl
fn srgb2oklab(c: vec3f) -> vec3f {
  let l = 0.4122214708f * c.r + 0.5363325363f * c.g + 0.0514459929f * c.b;
	let m = 0.2119034982f * c.r + 0.6806995451f * c.g + 0.1073969566f * c.b;
	let s = 0.0883024619f * c.r + 0.2817188376f * c.g + 0.6299787005f * c.b;

  let l_ = pow(l, 1.0/3.0);
  let m_ = pow(m, 1.0/3.0);
  let s_ = pow(s, 1.0/3.0);

  return vec3(
    0.2104542553f*l_ + 0.7936177850f*m_ - 0.0040720468f*s_,
    1.9779984951f*l_ - 2.4285922050f*m_ + 0.4505937099f*s_,
    0.0259040371f*l_ + 0.7827717662f*m_ - 0.8086757660f*s_,
  );
}
//////////////////////////////

//////////////////////////////
// https://github.com/patriciogonzalezvivo/lygia/blob/main/color/space/oklab2rgb.wgsl
fn oklab2srgb(c: vec3f) -> vec3f {
  let l_ = c.r + 0.3963377774f * c.g + 0.2158037573f * c.b;
  let m_ = c.r - 0.1055613458f * c.g - 0.0638541728f * c.b;
  let s_ = c.r - 0.0894841775f * c.g - 1.2914855480f * c.b;

  let l = l_*l_*l_;
  let m = m_*m_*m_;
  let s = s_*s_*s_;

  return vec3(
  	 4.0767416621f * l - 3.3077115913f * m + 0.2309699292f * s,
  	-1.2684380046f * l + 2.6097574011f * m - 0.3413193965f * s,
  	-0.0041960863f * l - 0.7034186147f * m + 1.7076147010f * s,
  );
}
//////////////////////////////

//////////////////////////////
// https://bottosson.github.io/posts/oklab/
fn oklab2oklch(oklab: vec3f) -> vec3f {
  let l = oklab.x;
  let a = oklab.y;
  let b = oklab.z;
  let c = sqrt(a * a + b * b);
  let h = atan2(b, a);
  return vec3f(l, c, h);
}
fn oklch2oklab(oklch: vec3f) -> vec3f {
  let l = oklch.x;
  let c = oklch.y;
  let h = oklch.z;
  let a = c * cos(h);
  let b = c * sin(h);
  return vec3f(l, a, b);
}
//////////////////////////////

// I just made these up bc composability is cool :)
fn srgb2oklch(rgb: vec3f) -> vec3f {
  return oklab2oklch(srgb2oklab(rgb));
}
fn oklch2srgb(oklch: vec3f) -> vec3f {
  return oklab2srgb(oklch2oklab(oklch));
}
