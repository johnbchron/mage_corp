pub mod cpu;

use crate::graph::{Graph, GraphError};

#[derive(Debug, Clone, Copy)]
pub struct Value {
  pub value: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct ValueWithMat {
  pub value:    f32,
  pub material: u32,
}

pub trait Evaluator {
  /// Evaluate the graph at the given points.
  fn eval(
    &self,
    graph: &Graph,
    root: usize,
    points: &[glam::Vec3],
  ) -> Result<Vec<f32>, GraphError>;
  /// Evaluate the graph at the given points with the material.
  fn eval_with_mat(
    &self,
    graph: &Graph,
    root: usize,
    points: &[glam::Vec3],
  ) -> Result<Vec<ValueWithMat>, GraphError>;
}
