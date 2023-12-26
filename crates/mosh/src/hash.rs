use std::hash::Hasher;

pub fn hash_vec3a<H: Hasher>(s: &glam::Vec3A, state: &mut H) {
  s.to_array()
    .iter()
    .for_each(|v| decorum::hash::FloatHash::float_hash(v, state));
}
