//! A shape graph.

use thiserror::Error;

use crate::node::{BinaryOp, Node, NodeId as Id, Solid};

/// An error type for [`Graph`] invariants.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum GraphError {
  /// A node at the given ID does not exist.
  #[error("node not found: {0}")]
  NodeNotFound(Id),
  /// A cycle was detected in the graph.
  #[error("cycle detected")]
  CycleDetected,
}

/// A shape graph.
pub struct Graph {
  nodes:   vec_map::VecMap<Node>,
  last_id: Id,
}

impl Graph {
  /// Create a new empty graph.
  pub fn new() -> Self {
    Self {
      nodes:   vec_map::VecMap::new(),
      last_id: 0,
    }
  }
  /// Creates a new empty graph with the given capacity.
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      nodes:   vec_map::VecMap::with_capacity(capacity),
      last_id: 0,
    }
  }

  /// Insert a node into the graph.
  fn insert(&mut self, node: Node) -> Id {
    let id = self.last_id;
    self.nodes.insert(id, node);
    self.last_id += 1;
    id
  }
  /// Returns a reference to the nodes collection.
  pub(crate) fn nodes(&self) -> &vec_map::VecMap<Node> { &self.nodes }

  /// Insert a solid node into the graph.
  fn insert_solid(&mut self, solid: Solid) -> Id {
    self.insert(Node::Solid(solid))
  }
  /// Inserts a sphere node into the graph.
  pub fn sphere(&mut self, radius: f32) -> Id {
    self.insert_solid(Solid::Sphere { radius })
  }
  /// Inserts a cuboid node into the graph.
  pub fn cuboid(&mut self, half_extents: glam::Vec3) -> Id {
    self.insert_solid(Solid::Cuboid { half_extents })
  }

  /// Insert a binary operation node into the graph.
  fn insert_binary(
    &mut self,
    op: BinaryOp,
    lhs: Id,
    rhs: Id,
  ) -> Result<Id, GraphError> {
    if !self.nodes.contains_key(lhs) {
      return Err(GraphError::NodeNotFound(lhs));
    }
    if !self.nodes.contains_key(rhs) {
      return Err(GraphError::NodeNotFound(rhs));
    }
    Ok(self.insert(Node::Binary { op, lhs, rhs }))
  }
  /// Inserts a union operation node into the graph.
  pub fn union(&mut self, lhs: Id, rhs: Id) -> Result<Id, GraphError> {
    self.insert_binary(BinaryOp::Union, lhs, rhs)
  }
  /// Inserts a difference operation node into the graph.
  pub fn difference(&mut self, lhs: Id, rhs: Id) -> Result<Id, GraphError> {
    self.insert_binary(BinaryOp::Difference, lhs, rhs)
  }
  /// Inserts an intersection operation node into the graph.
  pub fn intersection(&mut self, lhs: Id, rhs: Id) -> Result<Id, GraphError> {
    self.insert_binary(BinaryOp::Intersection, lhs, rhs)
  }

  /// Insert a unary operation node into the graph.
  fn insert_unary(
    &mut self,
    op: crate::node::UnaryOp,
    shape: Id,
  ) -> Result<Id, GraphError> {
    if !self.nodes.contains_key(shape) {
      return Err(GraphError::NodeNotFound(shape));
    }
    Ok(self.insert(Node::Unary { op, shape }))
  }
  /// Inserts a move operation node into the graph.
  pub fn move_(
    &mut self,
    shape: Id,
    offset: glam::Vec3,
  ) -> Result<Id, GraphError> {
    self.insert_unary(crate::node::UnaryOp::Move { offset }, shape)
  }

  /// Returns a list of recursive children of the given node, deduplicated and
  /// sorted. Contains the node itself.
  fn children(&self, id: Id) -> Result<Vec<Id>, GraphError> {
    let node = self.nodes.get(id).ok_or(GraphError::NodeNotFound(id))?;
    let mut nodes = match node {
      Node::Solid(_) => vec![id],
      Node::Binary { lhs, rhs, .. } => {
        let mut lhs_nodes = self.children(*lhs)?;
        let mut rhs_nodes = self.children(*rhs)?;
        lhs_nodes.append(&mut rhs_nodes);
        lhs_nodes.push(id);
        lhs_nodes
      }
      Node::Unary { shape, .. } => self
        .children(*shape)?
        .into_iter()
        .chain(std::iter::once(id))
        .collect(),
    };

    // sort and deduplicate
    nodes.sort_unstable();
    nodes.dedup();
    Ok(nodes)
  }

  /// Prunes the graph to keep only the given nodes and their children.
  pub fn prune(
    &mut self,
    ids: impl IntoIterator<Item = Id>,
  ) -> Result<(), GraphError> {
    let nodes_to_keep: Vec<Id> =
      ids.into_iter().try_fold(Vec::new(), |mut acc, id| {
        let children_result = self.children(id)?;

        acc.extend(children_result);
        Ok(acc)
      })?;

    let kept_nodes = self
      .nodes
      .drain()
      .filter(|(id, _)| nodes_to_keep.contains(id))
      .collect::<Vec<_>>();
    self.nodes.extend(kept_nodes);
    Ok(())
  }

  /// Checks if the graph is valid.
  ///
  /// A graph is valid if:
  /// - All `NodeId`s referenced by other nodes exist.
  /// - There are no cycles in the graph.
  pub fn check(&self) -> Result<(), GraphError> {
    // check that all node ids referenced by other nodes exist
    for node in self.nodes.values() {
      match node {
        Node::Solid(_) => {}
        Node::Binary { lhs, rhs, .. } => {
          if !self.nodes.contains_key(*lhs) {
            return Err(GraphError::NodeNotFound(*lhs));
          }
          if !self.nodes.contains_key(*rhs) {
            return Err(GraphError::NodeNotFound(*rhs));
          }
        }
        Node::Unary { shape, .. } => {
          if !self.nodes.contains_key(*shape) {
            return Err(GraphError::NodeNotFound(*shape));
          }
        }
      }
    }

    // check that there are no cycles in the graph
    let mut visited = vec![false; self.last_id];
    for (id, _) in self.nodes.iter() {
      if visited[id] {
        continue;
      }
      let mut stack = vec![id];
      while let Some(id) = stack.pop() {
        if visited[id] {
          return Err(GraphError::CycleDetected);
        }
        visited[id] = true;
        let node = self.nodes.get(id).unwrap();
        match node {
          Node::Solid(_) => {}
          Node::Binary { lhs, rhs, .. } => {
            stack.push(*lhs);
            stack.push(*rhs);
          }
          Node::Unary { shape, .. } => {
            stack.push(*shape);
          }
        }
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_works() {
    let mut graph = Graph::new();
    let sphere = graph.sphere(1.0);
    let cuboid = graph.cuboid(glam::Vec3::ONE);
    let _union = graph.union(sphere, cuboid).unwrap();
    let _difference = graph.difference(sphere, cuboid).unwrap();
    let _intersection = graph.intersection(sphere, cuboid).unwrap();
  }

  #[test]
  fn children_works() {
    let mut graph = Graph::new();
    let sphere = graph.sphere(1.0);
    let cuboid = graph.cuboid(glam::vec3(1.0, 1.0, 1.0));

    let union = graph.union(sphere, cuboid).unwrap();
    let union_children = graph.children(union).unwrap();
    assert!(union_children.contains(&sphere));
    assert!(union_children.contains(&cuboid));
    assert!(union_children.contains(&union));
    assert_eq!(union_children.len(), 3);

    // the children method should deduplicate nodes
    let difference = graph.difference(sphere, union).unwrap();
    let difference_children = graph.children(difference).unwrap();
    assert!(difference_children.contains(&sphere));
    assert!(difference_children.contains(&cuboid));
    assert!(difference_children.contains(&union));
    assert!(difference_children.contains(&difference));
    assert_eq!(difference_children.len(), 4);
  }

  #[test]
  fn prune_works() {
    let mut graph = Graph::new();
    let sphere = graph.sphere(1.0);
    let cuboid = graph.cuboid(glam::vec3(1.0, 1.0, 1.0));

    let union = graph.union(sphere, cuboid).unwrap();
    let difference = graph.difference(sphere, union).unwrap();
    assert_eq!(graph.nodes.len(), 4);

    graph.prune(vec![union]).unwrap();
    assert_eq!(graph.nodes.len(), 3);
    assert!(graph.nodes.contains_key(union));
    assert!(!graph.nodes.contains_key(difference));
  }

  #[test]
  fn check_detects_bad_ids() {
    let mut graph = Graph::new();
    let sphere = graph.sphere(1.0);
    let cuboid = graph.cuboid(glam::vec3(1.0, 1.0, 1.0));

    let union = graph.union(sphere, cuboid).unwrap();
    let _difference = graph.difference(sphere, union).unwrap();

    // remove the sphere node
    graph.nodes.remove(sphere);

    assert!(graph.check().is_err());
    assert_eq!(graph.check().unwrap_err(), GraphError::NodeNotFound(sphere));
  }

  #[test]
  fn check_detects_cycles() {
    let mut graph = Graph::new();
    let sphere = graph.sphere(1.0);
    let cuboid = graph.cuboid(glam::vec3(1.0, 1.0, 1.0));

    let union = graph.union(sphere, cuboid).unwrap();
    let difference = graph.difference(sphere, union).unwrap();

    // create a cycle
    let union_node = graph.nodes.get_mut(union).unwrap();
    *union_node = Node::Binary {
      op:  BinaryOp::Union,
      lhs: difference,
      rhs: sphere,
    };

    assert!(graph.check().is_err());
    assert_eq!(graph.check().unwrap_err(), GraphError::CycleDetected);
  }
}
