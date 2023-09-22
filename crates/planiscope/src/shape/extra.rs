use bevy_reflect::Reflect;
use fidget::{
  context::{IntoNode, Node},
  Context,
};
use serde::{Deserialize, Serialize};

use super::Shape;

#[derive(Hash, Clone, Debug, Serialize, Deserialize, Reflect)]
pub enum Extra {
  SmoothMinCubic(
    #[reflect(ignore)] Box<Shape>,
    #[reflect(ignore)] Box<Shape>,
    #[reflect(ignore)] Box<Shape>,
  ),
}

impl IntoNode for &Extra {
  fn into_node(self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    match self {
      Extra::SmoothMinCubic(lhs, rhs, k) => {
        let lhs = lhs.into_node(ctx)?;
        let rhs = rhs.into_node(ctx)?;
        let k = k.into_node(ctx)?;
        crate::nso::smooth::nso_smooth_min_cubic(lhs, rhs, k, ctx)
      }
    }
  }
}
