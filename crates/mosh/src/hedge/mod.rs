//! An implementation of an N-dimensional Half-Edge Mesh.

mod buffers;
mod keys;
mod storage;

use std::hash::Hash;

use hashbrown::{HashMap, HashSet};
use thiserror::Error;
use tracing::info_span;

pub use self::{
  keys::{EdgeKey, FaceKey, OpaqueKey, VertexKey},
  storage::{Storable, Storage},
};

/// A half-edge within a half-edge mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
  pub(crate) id:            EdgeKey,
  pub(crate) origin_vertex: VertexKey,
  pub(crate) target_vertex: VertexKey,
  pub(crate) face:          FaceKey,
  pub(crate) next_edge:     EdgeKey,
  pub(crate) prev_edge:     EdgeKey,
  pub(crate) twin_edge:     Option<EdgeKey>,
}

impl Storable for Edge {
  type Key = EdgeKey;
}

/// A face within a half-edge mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
  pub(crate) id:    FaceKey,
  pub(crate) edges: Vec<EdgeKey>,
}

impl Storable for Face {
  type Key = FaceKey;
}

/// Vertex data that can be stored in a half-edge mesh.
///
/// The vertex data contains all the data for a given vertex, aside from
/// connections to other vertices. Position is required, but other data is
/// optional. The half-edge mesh will deduplicate vertices that have the same
/// data, thus the data must be `Clone`, `PartialEq`, and `Eq`. Additionally,
/// the half-edge mesh uses parallel iteration where possible, so the vertex
/// data must also be `Sync`.
pub trait VertexData: Clone + PartialEq + Eq + Hash + Sync {
  /// Returns the position of the vertex.
  fn pos(&self) -> glam::Vec3A;
}

/// A vertex within a half-edge mesh.
///
/// Vertex data is stored with the vertex for fast access.
#[derive(Clone, Debug, PartialEq)]
pub struct Vertex<D: VertexData> {
  pub(crate) id:   VertexKey,
  pub(crate) data: D,
}

impl<D: VertexData> Storable for Vertex<D> {
  type Key = VertexKey;
}

/// A half-edge mesh.
#[derive(Debug)]
pub struct HedgeMesh<D: VertexData> {
  vertices: Storage<VertexKey, Vertex<D>>,
  edges:    Storage<EdgeKey, Edge>,
  faces:    Storage<FaceKey, Face>,
}

/// Represents the ways that a face can be invalid.
#[derive(Debug, Error)]
pub enum InvalidFaceError {
  /// The face has less than 3 edges.
  #[error("face has less than 3 edges: {0} edges")]
  TooFewEdges(usize),
  /// One or more edges in the face don't exist.
  #[error("one or more edges in the face don't exist")]
  EdgeDoesNotExist(EdgeKey),
  /// An edge has the wrong parent face.
  #[error("an edge has the wrong parent face")]
  EdgeHasWrongParentFace(EdgeKey, FaceKey),
  /// An edge cycles too early.
  #[error("an edge cycles too early")]
  EdgeCycledTooEarly(EdgeKey),
  /// An edge cycles too late.
  #[error("an edge cycles too late")]
  EdgeCycledTooLate(EdgeKey),
  /// The face does not contain the next edge.
  #[error("face does not contain the next edge")]
  FaceDoesNotContainNextEdge(EdgeKey),
}

impl<D: VertexData> HedgeMesh<D> {
  /// Returns an iterator over the faces of the mesh.
  pub fn faces(&self) -> Vec<FaceKey> { self.faces.iter_keys().collect() }

  /// Prunes vertices that are not used by any edges.
  pub fn prune_unused_vertices(&mut self) {
    let _span = info_span!("prune_unused_vertices").entered();

    let used_vertices = self
      .edges
      .iter()
      .map(|edge| edge.origin_vertex)
      .collect::<HashSet<_>>();

    self.vertices.retain(|k, _| used_vertices.contains(k));
  }

  /// Deduplicates vertices that have the same data.
  pub fn dedup_equal_vertices(&mut self) {
    let _span = info_span!("dedup_equal_vertices").entered();

    // a map from vertex data to the vertex keys that have that data
    let mut vertex_map: HashMap<D, HashSet<VertexKey>> = HashMap::new();

    for vertex in self.vertices.iter() {
      if vertex_map.contains_key(&vertex.data) {
        vertex_map.get_mut(&vertex.data).unwrap().insert(vertex.id);
      } else {
        vertex_map
          .insert(vertex.data.clone(), [vertex.id].iter().cloned().collect());
      }
    }

    for (_, keys) in vertex_map.iter().filter(|(_, v)| v.len() > 1) {
      let master_vertex_key = keys.iter().max().unwrap();
      for vertex_key in keys.iter().filter(|k| **k != *master_vertex_key) {
        self.replace_vertex(*vertex_key, *master_vertex_key);
      }
    }
  }

  /// Replaces a vertex with another vertex, by key.
  pub fn replace_vertex(
    &mut self,
    to_replace: VertexKey,
    replacement: VertexKey,
  ) {
    let _span = info_span!("replace_vertex").entered();

    for edge in self.edges.iter_mut() {
      if edge.origin_vertex == to_replace {
        edge.origin_vertex = replacement;
      }
      if edge.target_vertex == to_replace {
        edge.target_vertex = replacement;
      }
    }
    self.vertices.remove(to_replace);
  }

  /// Calculates the normal of a face.
  ///
  /// # Notes
  /// The normal is calculated only with the position of the vertices, and does
  /// not take into account any other vertex data.
  pub fn face_normal(&self, face: FaceKey) -> Option<glam::Vec3A> {
    let face = self.faces.get(face).unwrap();
    if face.edges.len() < 3 {
      panic!("face has less than 3 edges");
    }
    if face.edges.iter().any(|edge| *edge == EdgeKey::INVALID) {
      panic!("face has more than 3 edges");
    }

    let mut vertex_iter = face.edges.iter().map(|edge| {
      let edge = self
        .edges
        .get(*edge)
        .expect("failed to find edge in face while calculating normal");
      edge.origin_vertex
    });
    let mut vertex = vertex_iter.next().unwrap();
    let mut next_vertex = vertex_iter.next().unwrap();
    let mut normal = glam::Vec3A::ZERO;
    for next_next_vertex in vertex_iter {
      let edge1 = self.vertices.get(vertex).unwrap().data.pos()
        - self.vertices.get(next_vertex).unwrap().data.pos();
      let edge2 = self.vertices.get(next_next_vertex).unwrap().data.pos()
        - self.vertices.get(next_vertex).unwrap().data.pos();
      normal += edge1.cross(edge2);
      vertex = next_vertex;
      next_vertex = next_next_vertex;
    }
    Some(normal.normalize())
  }

  /// Determines whether a face is coplanar with another face.
  pub fn is_coplanar_with_face(&self, face: FaceKey, other: FaceKey) -> bool {
    let _span = info_span!("is_coplanar_with_face").entered();

    let face_normal = self.face_normal(face).unwrap();
    let other_normal = self.face_normal(other).unwrap();
    face_normal.dot(other_normal).abs() > 0.9999
  }

  /// Returns the neighbors of a face.
  pub fn face_neighbors(&self, face: FaceKey) -> Vec<FaceKey> {
    let _span = info_span!("face_neighbors").entered();

    // let face = self.faces.get(face).unwrap();
    // let mut neighbors = Vec::new();
    // for edge_key in face.edges.iter() {
    //   let edge = self.edges.get(*edge_key).unwrap();
    //   if let Some(twin_edge_key) = edge.twin_edge {
    //     let twin_edge = self.edges.get(twin_edge_key).unwrap();
    //     neighbors.push(twin_edge.face);
    //   }
    // }
    // neighbors
    self
      .faces
      .get(face)
      .unwrap()
      .edges
      .iter()
      .filter_map(|edge_key| {
        let edge = self.edges.get(*edge_key).unwrap();
        edge
          .twin_edge
          .map(|twin_edge_key| self.edges.get(twin_edge_key).unwrap().face)
      })
      .collect()
  }

  /// Determines whether two faces are neighbors.
  pub fn is_neighbors(&self, a: FaceKey, b: FaceKey) -> bool {
    let _span = info_span!("is_neighbors").entered();

    let a_face = self.faces.get(a).unwrap();
    let b_face = self.faces.get(b).unwrap();
    for edge_key in a_face.edges.iter() {
      let edge = self.edges.get(*edge_key).unwrap();
      if let Some(twin_edge_key) = edge.twin_edge {
        if b_face.edges.contains(&twin_edge_key) {
          return true;
        }
      }
    }
    false
  }

  /// Determines whether a face is valid.
  ///
  /// # Criteria
  /// A face is valid if:
  /// - It has at least 3 edges.
  /// - When traversing the edges through the `next_edge` key, the set of edges
  ///   traversed is equal to the set of edges in the face.
  /// - The `face` key of each edge is equal to the face's key.
  pub fn is_valid_face(&self, face: FaceKey) -> Result<(), InvalidFaceError> {
    let face = self.faces.get(face).unwrap();
    if face.edges.len() < 3 {
      return Err(InvalidFaceError::TooFewEdges(face.edges.len()));
    }
    let mut visited_edges = HashSet::new();
    let mut next_edge_key = face.edges[0];
    for _ in 0..face.edges.len() {
      let Some(edge) = self.edges.get(next_edge_key).cloned() else {
        return Err(InvalidFaceError::EdgeDoesNotExist(next_edge_key));
      };
      if edge.face != face.id {
        return Err(InvalidFaceError::EdgeHasWrongParentFace(
          next_edge_key,
          face.id,
        ));
      }
      if visited_edges.contains(&next_edge_key)
        && visited_edges.len() != face.edges.len() - 1
      {
        return Err(InvalidFaceError::EdgeCycledTooEarly(next_edge_key));
      }
      if !face.edges.contains(&next_edge_key) {
        return Err(InvalidFaceError::FaceDoesNotContainNextEdge(
          next_edge_key,
        ));
      }

      visited_edges.insert(next_edge_key);
      next_edge_key = edge.next_edge;
    }
    if visited_edges.len() != face.edges.len() {
      return Err(InvalidFaceError::EdgeCycledTooLate(next_edge_key));
    }

    Ok(())
  }

  /// Merges groups of adjacent and coplanar faces.
  pub fn merge_coplanar_faces(&mut self, percent_to_merge: f32) {
    let _span = info_span!("merge_coplanar_face_groups").entered();

    let original_count = self.faces.len();
    let mut merged_count = 0;
    let mut merged_faces = HashSet::new();

    while (merged_count as f32 / original_count as f32) < percent_to_merge {
      let mut merged_any_this_round = false;
      for face in merged_faces.clone().into_iter().chain(self.faces()) {
        let neighbors = self
          .face_neighbors(face)
          .into_iter()
          .filter(|f| self.is_coplanar_with_face(face, *f))
          .find(|f| self.faces_share_contiguous_border(face, *f));
        if let Some(neighbor) = neighbors {
          let new_face = self.merge_face_pair(face, neighbor);
          merged_faces.insert(new_face);
          merged_faces.remove(&neighbor);
          merged_faces.remove(&face);
          merged_count += 1;
          merged_any_this_round = true;
          break;
        }
      }

      if !merged_any_this_round {
        break;
      }
    }
  }

  /// Removes a face and its connnected edges from the mesh.
  pub fn remove_face_and_edges(
    &mut self,
    face_key: FaceKey,
  ) -> Option<FaceKey> {
    let face = self.faces.get(face_key)?;
    for edge_key in face.edges.iter() {
      let Some(edge) = self.edges.get(*edge_key).cloned() else {
        continue;
      };
      self.edges.remove(*edge_key);
      let Some(twin_edge_key) = edge.twin_edge else {
        continue;
      };
      let Some(twin_edge) = self.edges.get_mut(twin_edge_key) else {
        continue;
      };
      twin_edge.twin_edge = None;
    }
    self.faces.remove(face_key);
    Some(face_key)
  }

  /// Returns the set of edges on one face that are bordering another.
  pub fn bordering_edges(&self, a: FaceKey, b: FaceKey) -> HashSet<EdgeKey> {
    let _span = info_span!("bordering_edges").entered();

    let mut bordering_edges = HashSet::new();
    let a_edges = self.faces.get(a).unwrap().edges.clone();
    let b_edges = self.faces.get(b).unwrap().edges.clone();
    for edge_key in a_edges.iter() {
      let edge = self.edges.get(*edge_key).unwrap();
      if let Some(twin_edge_key) = edge.twin_edge {
        if b_edges.contains(&twin_edge_key) {
          bordering_edges.insert(*edge_key);
        }
      }
    }
    bordering_edges
  }

  fn faces_share_contiguous_border(&self, a: FaceKey, b: FaceKey) -> bool {
    let _span = info_span!("faces_share_contiguous_border").entered();

    let a_bordering_edges = self.bordering_edges(a, b);
    let mut disconnected_edges = 0;
    for edge_key in a_bordering_edges.iter() {
      let edge = self.edges.get(*edge_key).unwrap();
      if !a_bordering_edges.contains(&edge.next_edge) {
        disconnected_edges += 1;
      }
      if !a_bordering_edges.contains(&edge.prev_edge) {
        disconnected_edges += 1;
      }
    }
    disconnected_edges == 2
  }

  /// Merges two adjacent faces.
  ///
  /// # Invariants
  /// - The faces must be adjacent.
  /// - The faces must be coplanar.
  /// - Both faces must be valid according to `is_valid_face`.
  /// - The faces must share a contiguous border, i.e. if not triangular, they
  ///   can't have more than one bordering section, because merging them would
  ///   produce a hole.
  pub fn merge_face_pair(&mut self, a: FaceKey, b: FaceKey) -> FaceKey {
    let _span = info_span!("merge_face_pair").entered();

    assert!(a != b, "cannot merge a face with itself");
    assert!(
      self.faces.contains(a) && self.faces.contains(b),
      "one or more faces do not exist"
    );
    assert!(
      self.is_valid_face(a).is_ok(),
      "face a is invalid: {:?}",
      self.is_valid_face(a)
    );
    assert!(
      self.is_valid_face(b).is_ok(),
      "face b is invalid: {:?}",
      self.is_valid_face(b)
    );
    assert!(self.is_neighbors(a, b), "faces are not adjacent");
    assert!(self.is_coplanar_with_face(a, b), "faces are not coplanar");
    assert!(
      self.faces_share_contiguous_border(a, b),
      "bordering edges are not contiguous"
    );

    let a_face = self.faces.get(a).unwrap();
    let b_face = self.faces.get(b).unwrap();

    // check to see if the triangles are the same, but just reversed. if so,
    // we'll just remove one of the faces and return the other.
    let reversed_triangles = a_face
      .edges
      .iter()
      .filter_map(|edge_key| self.edges.get(*edge_key).unwrap().twin_edge)
      .all(|e| b_face.edges.contains(&e));
    if reversed_triangles {
      self.remove_face_and_edges(b);
      return a;
    }

    let a_bordering_edges = self.bordering_edges(a, b);
    let b_bordering_edges = self.bordering_edges(b, a);

    // check to see if, due to other merging, either triangle is degenerate.
    // if so, we'll just remove the degenerate triangle and return the other.

    if a_bordering_edges.len() == a_face.edges.len() {
      self.remove_face_and_edges(a);
      return b;
    } else if b_bordering_edges.len() == b_face.edges.len() {
      self.remove_face_and_edges(b);
      return a;
    }

    // in this case, a `*_pre_border_edge` is an edge that preceeds the border
    let a_pre_border_edge = a_bordering_edges
      .iter()
      .find_map(|edge_key| {
        let edge = self.edges.get(*edge_key).unwrap();
        if !a_bordering_edges.contains(&edge.prev_edge) {
          return Some(edge.prev_edge);
        }
        None
      })
      .unwrap();
    let a_post_border_edge = a_bordering_edges
      .iter()
      .find_map(|edge_key| {
        let edge = self.edges.get(*edge_key).unwrap();
        if !a_bordering_edges.contains(&edge.next_edge) {
          return Some(edge.next_edge);
        }
        None
      })
      .unwrap();
    let b_pre_border_edge = b_bordering_edges
      .iter()
      .find_map(|edge_key| {
        let edge = self.edges.get(*edge_key).unwrap();
        if !b_bordering_edges.contains(&edge.prev_edge) {
          return Some(edge.prev_edge);
        }
        None
      })
      .unwrap();
    let b_post_border_edge = b_bordering_edges
      .iter()
      .find_map(|edge_key| {
        let edge = self.edges.get(*edge_key).unwrap();
        if !b_bordering_edges.contains(&edge.next_edge) {
          return Some(edge.next_edge);
        }
        None
      })
      .unwrap();

    // here we're getting the edges from the post-border edge to the pre-border
    // edge by following the next edge
    let mut a_other_edges = Vec::new();
    let mut edge_key = a_post_border_edge;
    loop {
      let edge = self.edges.get(edge_key).unwrap();
      if edge.next_edge == a_pre_border_edge {
        break;
      }
      edge_key = edge.next_edge;
      a_other_edges.push(edge_key);
    }

    let mut b_other_edges = Vec::new();
    let mut edge_key = b_post_border_edge;
    loop {
      let edge = self.edges.get(edge_key).unwrap();
      if edge.next_edge == b_pre_border_edge {
        break;
      }
      edge_key = edge.next_edge;
      b_other_edges.push(edge_key);
    }

    // now we have the entire edge order. it will be:
    // a_post_border_edge -> a_other_edges -> a_pre_border_edge ->
    // b_post_border_edge -> b_other_edges -> b_pre_border_edge we need to
    // remove the bordering edges from the mesh, and then add a new face with
    // the new edge order

    // in the case that either triangle shares all but one edge, we need to
    // handle that case specially.
    let edge_order = if a_pre_border_edge == a_post_border_edge {
      [a_pre_border_edge, b_post_border_edge]
        .iter()
        .chain(b_other_edges.iter())
        .chain([b_pre_border_edge].iter())
        .cloned()
        .collect::<Vec<_>>()
    } else if b_pre_border_edge == b_post_border_edge {
      [a_post_border_edge]
        .iter()
        .chain(a_other_edges.iter())
        .chain([a_pre_border_edge, b_pre_border_edge].iter())
        .cloned()
        .collect::<Vec<_>>()
    } else {
      [a_post_border_edge]
        .iter()
        .chain(a_other_edges.iter())
        .chain([a_pre_border_edge, b_post_border_edge].iter())
        .chain(b_other_edges.iter())
        .chain([b_pre_border_edge].iter())
        .cloned()
        .collect::<Vec<_>>()
    };

    let new_face_key = self.faces.add(Face {
      id:    FaceKey::INVALID,
      edges: edge_order.clone(),
    });
    let new_face = self.faces.get_mut(new_face_key).unwrap();
    new_face.id = new_face_key;

    // remove the old faces
    let a_face = self.faces.remove(a).unwrap();
    let b_face = self.faces.remove(b).unwrap();

    // fix the edges
    for (i, edge_key) in edge_order.iter().enumerate() {
      let edge = self.edges.get_mut(*edge_key).unwrap();
      edge.face = new_face_key;
      edge.next_edge = edge_order[(i + 1) % edge_order.len()];
      edge.prev_edge =
        edge_order[(i + edge_order.len() - 1) % edge_order.len()];
    }

    let is_valid_face = self.is_valid_face(new_face_key);
    if is_valid_face.is_err() {
      println!("a: {:?}", a_face);
      println!("b: {:?}", b_face);
      println!("new face key: {:?}", new_face_key);
      println!("edge order: {:?}", edge_order);
      for edge_key in edge_order.iter() {
        println!(
          "edge: {:?}, face: {:?}, next: {:?}, prev: {:?}, twin: {:?}",
          edge_key,
          self.edges.get(*edge_key).unwrap().face,
          self.edges.get(*edge_key).unwrap().next_edge,
          self.edges.get(*edge_key).unwrap().prev_edge,
          self.edges.get(*edge_key).unwrap().twin_edge,
        );
      }
      panic!("new face is invalid: {}", is_valid_face.unwrap_err());
    }

    new_face_key
  }

  /// Merges a group of faces.
  ///
  /// # Invariants
  /// - The faces must form a contiguous group.
  /// - The faces must be coplanar.
  /// - All faces must be valid according to `is_valid_face`.
  /// - Adjacent must share a contiguous border, i.e. if not triangular, they
  ///   can't have more than one bordering section, because merging them would
  ///   produce a hole.
  pub fn merge_face_group(&mut self, face_group: HashSet<FaceKey>) -> FaceKey {
    let _span =
      info_span!("merge_face_group", count = face_group.len()).entered();

    let mut all_faces = face_group.iter().cloned().collect::<HashSet<_>>();
    while let Some((face, neighbor)) =
      all_faces.clone().into_iter().find_map(|f| {
        self
          .face_neighbors(f)
          .into_iter()
          .find(|n| {
            all_faces.contains(n) && self.faces_share_contiguous_border(f, *n)
          })
          .map(|n| (f, n))
      })
    {
      let new_face = self.merge_face_pair(face, neighbor);
      all_faces.remove(&face);
      all_faces.remove(&neighbor);
      all_faces.insert(new_face);
    }
    assert_eq!(all_faces.len(), 1);

    // println!("merged face group with {} faces", face_group.len());
    *all_faces.iter().next().unwrap()
  }

  /// Regenerates invalid keys.
  ///
  /// # Invariants
  /// Requires that faces have valid and consistently edge keys.
  pub fn regenerate_invalid_keys(&mut self) {
    let _span = info_span!("regenerate_invalid_keys").entered();

    // start with `self.id` keys
    let vertices_with_invalid_self_keys = self
      .vertices
      .inner()
      .iter()
      .filter_map(|(k, v)| {
        if v.id == VertexKey::INVALID {
          Some(k)
        } else {
          None
        }
      })
      .copied()
      .collect::<Vec<_>>();
    let edges_with_invalid_self_keys = self
      .edges
      .inner()
      .iter()
      .filter_map(|(k, v)| {
        if v.id == EdgeKey::INVALID {
          Some(k)
        } else {
          None
        }
      })
      .copied()
      .collect::<Vec<_>>();
    let faces_with_invalid_self_keys = self
      .faces
      .inner()
      .iter()
      .filter_map(|(k, v)| {
        if v.id == FaceKey::INVALID {
          Some(k)
        } else {
          None
        }
      })
      .copied()
      .collect::<Vec<_>>();

    for vertex_key in vertices_with_invalid_self_keys {
      self.vertices.get_mut(vertex_key).unwrap().id = vertex_key;
    }
    for edge_key in edges_with_invalid_self_keys {
      self.edges.get_mut(edge_key).unwrap().id = edge_key;
    }
    for face_key in faces_with_invalid_self_keys {
      self.faces.get_mut(face_key).unwrap().id = face_key;
    }

    // fix edges with invalid face keys
    let edges_with_invalid_face_keys = self
      .edges
      .inner()
      .iter()
      .filter_map(|(k, v)| {
        if v.face == FaceKey::INVALID {
          Some(k)
        } else {
          None
        }
      })
      .copied()
      .collect::<Vec<_>>();
    if !edges_with_invalid_face_keys.is_empty() {
      let mut edge_to_face_map = HashMap::new();
      for face in self.faces.iter() {
        for edge_key in face.edges.iter() {
          edge_to_face_map.insert(*edge_key, face.id);
        }
      }
      for edge_key in edges_with_invalid_face_keys {
        let edge = self.edges.get_mut(edge_key).unwrap();
        // edge.face = *edge_to_face_map.get(&edge_key).unwrap();
        if let Some(face_key) = edge_to_face_map.get(&edge_key) {
          edge.face = *face_key;
        }
      }
    }

    self.fix_edge_parent_keys();
    self.fix_edge_twin_keys();
    self.fix_edge_next_prev_keys();
  }

  fn fix_edge_twin_keys(&mut self) {
    let _span = info_span!("fix_edge_twin_keys").entered();

    let mut vertex_pair_to_edge = HashMap::new();
    for edge in self.edges.iter() {
      let target_vertex = edge.target_vertex;
      let origin_vertex = edge.origin_vertex;
      vertex_pair_to_edge.insert((origin_vertex, target_vertex), edge.id);
    }
    for edge in self.edges.iter_mut() {
      let twin_edge_key =
        vertex_pair_to_edge.get(&(edge.target_vertex, edge.origin_vertex));
      edge.twin_edge = twin_edge_key.cloned();
    }
  }

  /// Fixes the `next_edge` and `prev_edge` keys for each edge.
  ///
  /// # Invariants
  /// The `face` keys of each edge must be correct.
  fn fix_edge_next_prev_keys(&mut self) {
    let _span = info_span!("fix_edge_next_prev_keys").entered();

    let mut vertex_to_edge_by_face: HashMap<
      FaceKey,
      HashMap<VertexKey, EdgeKey>,
    > = HashMap::new();
    for face in self.faces.iter() {
      let mut vertex_to_edge = HashMap::new();
      for edge_key in face.edges.iter() {
        let edge = self.edges.get(*edge_key).unwrap();
        vertex_to_edge.insert(edge.origin_vertex, edge.id);
      }
      vertex_to_edge_by_face.insert(face.id, vertex_to_edge);
    }

    for face in self.faces.iter() {
      let vertex_to_edge = vertex_to_edge_by_face.get(&face.id).unwrap();
      for edge_key in face.edges.iter() {
        let current_edge_key = *edge_key;
        let next_edge_key = vertex_to_edge
          .get(&self.edges.get(*edge_key).unwrap().target_vertex)
          .unwrap();

        let current_edge = self.edges.get_mut(current_edge_key).unwrap();
        current_edge.next_edge = *next_edge_key;

        let next_edge = self.edges.get_mut(*next_edge_key).unwrap();
        next_edge.prev_edge = current_edge_key;
      }
    }
  }

  /// Fixes edge parent keys.
  ///
  /// # Invariants
  /// The `face` keys of each edge must be correct.
  fn fix_edge_parent_keys(&mut self) {
    let _span = info_span!("fix_edge_parent_keys").entered();

    let mut edge_to_face_map = HashMap::new();
    for face in self.faces.iter() {
      for edge_key in face.edges.iter() {
        edge_to_face_map.insert(*edge_key, face.id);
      }
    }
    for edge_key in self.edges.iter_keys() {
      let edge = self.edges.get_mut(edge_key).unwrap();
      // edge.face = *edge_to_face_map.get(&edge_key).unwrap();
      if let Some(face_key) = edge_to_face_map.get(&edge_key) {
        edge.face = *face_key;
      }
    }
  }

  // /// Fixes the order of edges in each face so they follow the order of
  // /// `next_edge`.
  // ///
  // /// # Invariants
  // /// - The face must be considered valid by `is_valid_face`.
  // fn reorder_edges_in_face(&mut self, face_key: FaceKey) {
  //   let face = self.faces.get(face_key).unwrap();
  //   let mut edge_key = face.edges[0];
  //   let mut edge_keys = Vec::new();
  //   for _ in 0..face.edges.len() {
  //     let edge = self.edges.get(edge_key).unwrap();
  //     edge_keys.push(edge.id);
  //     edge_key = edge.next_edge;
  //   }
  //   self.faces.get_mut(face_key).unwrap().edges = edge_keys;
  // }

  /// Counts the maxiumum arity of the total mesh.
  pub fn arity(&self) -> usize {
    self
      .faces
      .iter()
      .map(|face| face.edges.len())
      .max()
      .unwrap()
  }
}
