pub mod builder;
pub mod compound;

use std::{
  collections::{hash_map::DefaultHasher, HashMap},
  hash::{Hash, Hasher},
  ops::{Add, Div, Mul, Neg, Sub},
};

use bevy_reflect::Reflect;
use decorum::hash::FloatHash;
use educe::Educe;
use fidget::{
  context::{IntoNode, Node},
  rhai::Engine,
  Context,
};
use serde::{Deserialize, Serialize};

pub trait CachedIntoNode: Clone + Hash {
  fn cached_into_node(
    &self,
    ctx: &mut Context,
    cache: &mut HashMap<u64, Node>,
  ) -> Result<Node, fidget::Error>;
  fn eval_root_cached(&self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    let mut cache = HashMap::new();
    self.cached_into_node(ctx, &mut cache)
  }
}

impl CachedIntoNode for Shape {
  fn cached_into_node(
    &self,
    ctx: &mut Context,
    cache: &mut HashMap<u64, Node>,
  ) -> Result<Node, fidget::Error> {
    let mut hasher = DefaultHasher::new();
    self.hash(&mut hasher);
    let hash = hasher.finish();
    if let Some(node) = cache.get(&hash) {
      return Ok(node.clone());
    }

    let node = self.into_node(ctx)?;
    cache.insert(hash, node.clone());
    Ok(node)
  }
}

#[derive(Educe, Clone, Debug, Serialize, Deserialize, Reflect)]
#[educe(Hash)]
pub enum Shape {
  Expression {
    expr: String,
  },
  XNode,
  YNode,
  ZNode,
  Constant(#[educe(Hash(trait = "FloatHash"))] f64),
  Add(#[reflect(ignore)] Box<Shape>, #[reflect(ignore)] Box<Shape>),
  Sub(#[reflect(ignore)] Box<Shape>, #[reflect(ignore)] Box<Shape>),
  Mul(#[reflect(ignore)] Box<Shape>, #[reflect(ignore)] Box<Shape>),
  Div(#[reflect(ignore)] Box<Shape>, #[reflect(ignore)] Box<Shape>),
  Min(#[reflect(ignore)] Box<Shape>, #[reflect(ignore)] Box<Shape>),
  Max(#[reflect(ignore)] Box<Shape>, #[reflect(ignore)] Box<Shape>),
  Neg(#[reflect(ignore)] Box<Shape>),
  Exp(#[reflect(ignore)] Box<Shape>),
  Sin(#[reflect(ignore)] Box<Shape>),
  Cos(#[reflect(ignore)] Box<Shape>),
  Recip(#[reflect(ignore)] Box<Shape>),
  Abs(#[reflect(ignore)] Box<Shape>),
  Sqrt(#[reflect(ignore)] Box<Shape>),
  Square(#[reflect(ignore)] Box<Shape>),
  Remap {
    #[reflect(ignore)]
    root:  Box<Shape>,
    #[reflect(ignore)]
    new_x: Box<Shape>,
    #[reflect(ignore)]
    new_y: Box<Shape>,
    #[reflect(ignore)]
    new_z: Box<Shape>,
  },
  Extra(compound::Compound),
}

impl Default for Shape {
  fn default() -> Self { Self::Constant(1.0_f64) }
}

impl From<f64> for Shape {
  fn from(value: f64) -> Self { Shape::Constant(value) }
}

impl From<f32> for Shape {
  fn from(value: f32) -> Self { Shape::Constant(value.into()) }
}

impl Add<Shape> for Shape {
  type Output = Shape;

  fn add(self, rhs: Shape) -> Self::Output {
    Shape::Add(Box::new(self), Box::new(rhs))
  }
}
impl Sub<Shape> for Shape {
  type Output = Shape;

  fn sub(self, rhs: Shape) -> Self::Output {
    Shape::Sub(Box::new(self), Box::new(rhs))
  }
}
impl Mul<Shape> for Shape {
  type Output = Shape;

  fn mul(self, rhs: Shape) -> Self::Output {
    Shape::Mul(Box::new(self), Box::new(rhs))
  }
}
impl Div<Shape> for Shape {
  type Output = Shape;

  fn div(self, rhs: Shape) -> Self::Output {
    Shape::Div(Box::new(self), Box::new(rhs))
  }
}
impl Neg for Shape {
  type Output = Shape;

  fn neg(self) -> Self::Output { Shape::Neg(Box::new(self)) }
}

impl Shape {
  pub fn new_expr(expr: &str) -> Self {
    Self::Expression {
      expr: expr.to_string(),
    }
  }
}

impl IntoNode for &Shape {
  fn into_node(self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    match self {
      Shape::Expression { expr } => {
        let mut engine = Engine::new(Some(ctx.clone()));
        let (node, context) = engine.eval_no_clear(expr)?;
        *ctx = context;
        Ok(node)
      }
      Shape::XNode => Ok(ctx.x()),
      Shape::YNode => Ok(ctx.y()),
      Shape::ZNode => Ok(ctx.z()),
      Shape::Constant(c) => Ok(ctx.constant(*c)),
      Shape::Add(lhs, rhs) => ctx.add(lhs.as_ref(), rhs.as_ref()),
      Shape::Sub(lhs, rhs) => ctx.sub(lhs.as_ref(), rhs.as_ref()),
      Shape::Mul(lhs, rhs) => ctx.mul(lhs.as_ref(), rhs.as_ref()),
      Shape::Div(lhs, rhs) => ctx.div(lhs.as_ref(), rhs.as_ref()),
      Shape::Min(lhs, rhs) => ctx.min(lhs.as_ref(), rhs.as_ref()),
      Shape::Max(lhs, rhs) => ctx.max(lhs.as_ref(), rhs.as_ref()),
      Shape::Neg(lhs) => ctx.neg(lhs.as_ref()),
      Shape::Exp(lhs) => ctx.exp(lhs.as_ref()),
      Shape::Sin(lhs) => ctx.sin(lhs.as_ref()),
      Shape::Cos(lhs) => ctx.cos(lhs.as_ref()),
      Shape::Recip(lhs) => ctx.recip(lhs.as_ref()),
      Shape::Abs(lhs) => ctx.abs(lhs.as_ref()),
      Shape::Sqrt(lhs) => ctx.sqrt(lhs.as_ref()),
      Shape::Square(lhs) => ctx.square(lhs.as_ref()),
      Shape::Remap {
        root,
        new_x,
        new_y,
        new_z,
      } => {
        let root_node = root.as_ref().into_node(ctx)?;
        let new_x_node = new_x.as_ref().into_node(ctx)?;
        let new_y_node = new_y.as_ref().into_node(ctx)?;
        let new_z_node = new_z.as_ref().into_node(ctx)?;
        ctx.remap_xyz(root_node, [new_x_node, new_y_node, new_z_node])
      }
      Shape::Extra(extra) => extra.into_node(ctx),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rhai_shape_converts_to_node() {
    let mut ctx = Context::new();

    let x_plus_one = (&Shape::new_expr("x + 1")).into_node(&mut ctx);
    assert!(x_plus_one.is_ok());
    let x_plus_one = x_plus_one.unwrap();

    let eval_result = ctx.eval_xyz(x_plus_one, 2.0, 0.0, 0.0);
    assert!(eval_result.is_ok());
    let eval_result = eval_result.unwrap();

    assert_eq!(eval_result, 3.0);
  }

  #[test]
  fn rhai_shape_eval_does_not_mangle_a_context() {
    let mut ctx = Context::new();

    let x = ctx.x();
    let one = ctx.constant(1.0);
    let x_plus_one = ctx.add(x, one).unwrap();
    let y = ctx.y();
    let x_plus_one_times_y = ctx.mul(x_plus_one, y).unwrap();
    println!("{}", ctx.dot());

    let eval_result = ctx.eval_xyz(x_plus_one_times_y, 2.0, 3.0, 0.0).unwrap();
    assert_eq!(eval_result, 9.0);

    let _shape_node = (&Shape::new_expr("x + 1")).into_node(&mut ctx).unwrap();
    println!("{}", ctx.dot());

    let eval_result = ctx.eval_xyz(x_plus_one_times_y, 2.0, 3.0, 0.0).unwrap();
    assert_eq!(eval_result, 9.0);
  }
}
