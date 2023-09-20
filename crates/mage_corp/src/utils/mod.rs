pub mod despawn;
pub mod in_progress;
pub mod timer_lifetime;

pub fn f32_lerp(lhs: f32, rhs: f32, s: f32) -> f32 {
  lhs + ((rhs - lhs) * s)
}
