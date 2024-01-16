//! Shape nodes.

/// An ID for a node in a graph.
pub type NodeId = usize;

#[derive(Debug, Clone)]
pub(crate) enum Solid {
  Sphere { radius: f32 },
  Cuboid { half_extents: glam::Vec3 },
}

#[derive(Debug, Clone)]
pub(crate) enum Node {
  Solid(Solid),
  Binary {
    op:  BinaryOp,
    lhs: NodeId,
    rhs: NodeId,
  },
  Unary {
    op:    UnaryOp,
    shape: NodeId,
  },
}

#[derive(Debug, Clone)]
pub(crate) enum BinaryOp {
  Union,
  Difference,
  Intersection,
}

#[derive(Debug, Clone)]
pub(crate) enum UnaryOp {
  Move { offset: glam::Vec3 },
}
