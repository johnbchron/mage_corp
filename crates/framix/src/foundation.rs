//! A foundation module.

use super::*;
use crate::{primitive::ConcreteBlock, rendered::RenderedPrimitive};

/// A brick wall module.
#[derive(Reflect, Default)]
pub struct FoundationFragment;

impl FragmentConfig for FoundationFragment {
  fn render(&self) -> RenderedFragment {
    let smudge = 1.02;
    let block = ConcreteBlock {
      dims: Vec3::new(1.0, 0.5, 1.0) * smudge,
    };
    let primitive = RenderedPrimitive::new(
      Box::new(block),
      Transform::from_xyz(0.0, 0.25, 0.0),
    );
    RenderedFragment::new(vec![primitive])
  }
}
