use rayon::prelude::*;

use super::*;
use crate::node::{Node, Solid};

pub struct CpuEval;

impl Evaluator for CpuEval {
  fn eval(
    &self,
    graph: &Graph,
    root: usize,
    points: &[glam::Vec3],
  ) -> Result<Vec<f32>, GraphError> {
    let mut graph = graph.clone();
    graph.prune(vec![root]).unwrap();

    let solid_indices = graph
      .nodes()
      .iter()
      .filter_map(|(i, node)| {
        if matches!(node, Node::Solid(_)) {
          Some(i)
        } else {
          None
        }
      })
      .collect::<Vec<_>>();

    let solid_values = solid_indices
      .par_iter()
      .map(|i| {
        if let Node::Solid(solid) = graph.nodes()[*i] {
          match solid {
            Solid::Sphere { radius } => radius,
            Solid::Cuboid { half_extents } => {
              half_extents.x.max(half_extents.y).max(half_extents.z)
            }
          }
        } else {
          unreachable!()
        }
      })
      .collect::<Vec<_>>();
  }

  fn eval_with_mat(
    &self,
    graph: &Graph,
    root: usize,
    points: &[glam::Vec3],
  ) -> Result<Vec<ValueWithMat>, GraphError> {
    todo!()
  }
}
