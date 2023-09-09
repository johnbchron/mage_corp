use fidget::{
  context::{IntoNode, Node},
  Context,
};
use serde::{Deserialize, Serialize};

use crate::shape::Shape;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Composition {
  shapes: Vec<Shape>,
}

impl Composition {
  pub fn new(shapes: Vec<Shape>) -> Self {
    Self { shapes }
  }
}

impl Default for Composition {
  fn default() -> Self {
    Self::new(vec![Shape::new_rhai("y - 1")])
  }
}

impl From<Vec<Shape>> for Composition {
  fn from(shapes: Vec<Shape>) -> Self {
    Self::new(shapes)
  }
}

impl IntoNode for &Composition {
  fn into_node(self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    // turn each shape into a node, then make a binary tree of `min` operations
    let nodes = self
      .shapes
      .clone()
      .iter()
      .map(|s| s.into_node(ctx))
      .collect::<Result<Vec<Node>, fidget::Error>>()?;
    binary_tree(nodes, ctx)
  }
}

fn binary_tree(
  mut tree: Vec<Node>,
  ctx: &mut Context,
) -> Result<Node, fidget::Error> {
  while tree.len() > 1 {
    let mut new_tree = Vec::new();
    for i in (0..tree.len()).step_by(2) {
      let a = tree[i];
      let b = if i + 1 < tree.len() { tree[i + 1] } else { a };
      new_tree.push(ctx.min(a, b)?);
    }
    tree = new_tree;
  }

  Ok(tree[0])
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn composition_evals_to_min_shape() {
    let comp = Composition::new(vec![
      Shape::new_rhai("x + 1"),
      Shape::new_rhai("x + 2"),
    ]);

    let mut ctx = Context::new();
    // turn the composition into a node
    let node = comp.into_node(&mut ctx);
    assert!(node.is_ok());
    let node = node.unwrap();

    let eval_result = ctx.eval_xyz(node, 1.0, 0.0, 0.0);
    assert!(eval_result.is_ok());
    let eval_result = eval_result.unwrap();

    // make sure the evaluation is the minimum between "x + 1" and "x + 2",
    // which is "x + 1", which is 2
    assert_eq!(eval_result, 2.0);
  }
}
