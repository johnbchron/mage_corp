use fidget::{
  context::{IntoNode, Node},
  rhai::Engine,
  Context,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Shape {
  FidgetRhai { expr: String },
}

impl IntoNode for &Shape {
  fn into_node(self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    match self {
      Shape::FidgetRhai { expr } => {
        let mut engine = Engine::new(Some(ctx.clone()));
        let (node, context) = engine.eval(expr)?;
        *ctx = context;
        Ok(node)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use fidget::vm;

  use super::*;

  #[test]
  fn it_works() {
    let shape = Shape::FidgetRhai {
      expr: "x + 1".to_string(),
    };

    let mut ctx = Context::new();
    let node = shape.into_node(&mut ctx);

    // make sure it translate correctly from rhai
    assert!(node.is_ok());
    let node = node.unwrap();

    // this is 5 because the engine declares x, y, and z by default
    // and we add a constant node for 1.0 and an addition node
    assert_eq!(ctx.len(), 5);

    // make sure we can get a tape
    let tape = ctx.get_tape::<vm::Eval>(node);
    assert!(tape.is_ok());
    let tape = tape.unwrap();

    // build an evaluator and make sure the result is correct
    let evaluator = tape.new_point_evaluator();
    let eval_result = evaluator.eval(0.0, 0.0, 0.0, &[]);
    assert!(eval_result.is_ok());
    let eval_result = eval_result.unwrap();

    assert_eq!(eval_result.0, 1.0);
  }
}
