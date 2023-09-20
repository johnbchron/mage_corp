pub mod despawn;
pub mod in_progress;
pub mod timer_lifetime;

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

pub fn f32_lerp(lhs: f32, rhs: f32, s: f32) -> f32 {
  lhs + ((rhs - lhs) * s)
}

pub fn hash_single<H: Hash>(value: &H) -> u64 {
  let mut hasher = DefaultHasher::new();
  value.hash(&mut hasher);
  hasher.finish()
}
