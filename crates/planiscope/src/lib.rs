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

impl Shape {
  pub fn new_rhai(expr: &str) -> Self {
    Self::FidgetRhai {
      expr: expr.to_string(),
    }
  }
}

impl IntoNode for &Shape {
  fn into_node(self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    match self {
      Shape::FidgetRhai { expr } => {
        let mut engine = Engine::new(Some(ctx.clone()));
        let (node, context) = engine.eval_no_clear(expr)?;
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
  fn uses_a_context() {
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

  #[test]
  fn converts_to_node() {
    let mut ctx = Context::new();

    let x_plus_one = (&Shape::new_rhai("x + 1")).into_node(&mut ctx);
    assert!(x_plus_one.is_ok());
    let x_plus_one = x_plus_one.unwrap();

    let eval_result = ctx.eval_xyz(x_plus_one, 2.0, 0.0, 0.0);
    assert!(eval_result.is_ok());
    let eval_result = eval_result.unwrap();

    assert_eq!(eval_result, 3.0);
  }

  #[test]
  fn does_not_mangle_a_context() {
    let mut ctx = Context::new();

    let x = ctx.x();
    let one = ctx.constant(1.0);
    let x_plus_one = ctx.add(x, one).unwrap();
    let y = ctx.y();
    let x_plus_one_times_y = ctx.mul(x_plus_one, y).unwrap();
    println!("{}", ctx.dot());

    let eval_result = ctx.eval_xyz(x_plus_one_times_y, 2.0, 3.0, 0.0).unwrap();
    assert_eq!(eval_result, 9.0);

    let _shape_node = (&Shape::new_rhai("x + 1")).into_node(&mut ctx).unwrap();
    println!("{}", ctx.dot());

    let eval_result = ctx.eval_xyz(x_plus_one_times_y, 2.0, 3.0, 0.0).unwrap();
    assert_eq!(eval_result, 9.0);
  }
}
