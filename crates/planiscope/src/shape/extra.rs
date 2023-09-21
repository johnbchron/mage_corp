use bevy_reflect::Reflect;
use fidget::{
  context::{IntoNode, Node},
  Context,
};
use serde::{Deserialize, Serialize};

use super::Shape;

#[derive(Hash, Clone, Debug, Serialize, Deserialize, Reflect)]
pub enum Extra {
  Smooth(#[reflect(ignore)] Box<Shape>, #[reflect(ignore)] Box<Shape>),
}

impl IntoNode for &Extra {
  fn into_node(self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    match self {
      Extra::Smooth(lhs, k) => {
        let zero = ctx.constant(0.0);
        let point_five = ctx.constant(0.5);
        let one = ctx.constant(1.0);

        todo!()
      }
    }
  }
}
