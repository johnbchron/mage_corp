//! An implementation of an N-dimensional Half-Edge Mesh.

mod buffers;
mod keys;
mod storage;

use std::hash::Hash;

use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;

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

impl<D: VertexData> HedgeMesh<D> {
  /// Prunes vertices that are not used by any edges.
  pub fn prune_unused_vertices(&mut self) {
    let used_vertices = self
      .edges
      .iter()
      .map(|edge| edge.origin_vertex)
      .collect::<HashSet<_>>();

    self.vertices.retain(|k, _| used_vertices.contains(k));
  }

  /// Deduplicates vertices that have the same data.
  pub fn dedup_equal_vertices(&mut self) {
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

  /// Returns the neighbors of a face.
  pub fn face_neighbors(&self, face: FaceKey) -> Vec<FaceKey> {
    let face = self.faces.get(face).unwrap();
    let mut neighbors = Vec::new();
    for edge_key in face.edges.iter() {
      let edge = self.edges.get(*edge_key).unwrap();
      if let Some(twin_edge_key) = edge.twin_edge {
        let twin_edge = self.edges.get(twin_edge_key).unwrap();
        neighbors.push(twin_edge.face);
      }
    }
    neighbors
  }

  /// Identifies groups of adjacent and coplanar faces.
  pub fn find_coplanar_face_groups(&self) -> Vec<HashSet<FaceKey>> {
    let mut coplanar_face_groups = Vec::new();
    let mut visited_faces = HashSet::new();

    let normals = self
      .faces
      .par_iter_keys()
      .map(|face_key| (face_key, self.face_normal(*face_key).unwrap()))
      .collect::<HashMap<_, _>>();

    let mut face_keys = self.faces.iter_keys().collect::<Vec<_>>();
    face_keys.sort();

    for face_key in face_keys {
      if visited_faces.contains(&face_key) {
        continue;
      }
      let mut coplanar_face_group = HashSet::new();
      let mut stack = vec![face_key];

      while let Some(face_key) = stack.pop() {
        if coplanar_face_group.contains(&face_key) {
          continue;
        }
        coplanar_face_group.insert(face_key);
        visited_faces.insert(face_key);
        let face_normal = normals.get(&face_key).unwrap();

        for neighbor_face_key in self.face_neighbors(face_key) {
          if visited_faces.contains(&neighbor_face_key) {
            continue;
          }
          let neighbor_face_normal = normals.get(&neighbor_face_key).unwrap();
          if (*face_normal - *neighbor_face_normal).length() < 0.0001 {
            stack.push(neighbor_face_key);
          }
        }
      }
      if coplanar_face_group.len() > 1 {
        coplanar_face_groups.push(coplanar_face_group);
      }
    }
    coplanar_face_groups
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

  /// Merges a set of faces into a single face.
  ///
  /// # Notes
  /// This operation increases the arity of the mesh, and may make it
  /// impossible to convert back to buffers.
  pub fn merge_faces(&mut self, faces: &[FaceKey]) -> Option<FaceKey> {
    let mut face_keys = faces.to_vec();
    face_keys.sort();
    let faces = face_keys
      .iter()
      .skip(1)
      .map(|f| self.faces.get(*f).unwrap().clone())
      .collect::<Vec<_>>();

    let master_face_key = face_keys.first()?;
    let master_face = self.faces.get_mut(*master_face_key)?;

    for face in faces {
      let mut face_edges = face.edges.clone();
      face_edges.reverse();
      master_face.edges.extend(face_edges);
    }
    for face_key in face_keys.iter().skip(1) {
      self.faces.remove(*face_key);
    }

    Some(*master_face_key)
  }

  fn add_edge(&mut self, origin: VertexKey, target: VertexKey) -> EdgeKey {
    let edge = Edge {
      id:            EdgeKey::INVALID,
      origin_vertex: origin,
      target_vertex: target,
      face:          FaceKey::INVALID,
      next_edge:     EdgeKey::INVALID,
      prev_edge:     EdgeKey::INVALID,
      twin_edge:     None,
    };
    self.edges.add(edge)
  }

  fn add_or_get_edge(
    &mut self,
    origin: VertexKey,
    target: VertexKey,
  ) -> EdgeKey {
    for (key, edge) in self.edges.inner().iter() {
      if edge.origin_vertex == origin && edge.target_vertex == target {
        return *key;
      }
    }
    self.add_edge(origin, target)
  }

  fn add_face(&mut self, a: VertexKey, b: VertexKey, c: VertexKey) -> FaceKey {
    let a_to_b = self.add_or_get_edge(a, b);
    let b_to_c = self.add_or_get_edge(b, c);
    let c_to_a = self.add_or_get_edge(c, a);

    let face = Face {
      id:    FaceKey::INVALID,
      edges: vec![a_to_b, b_to_c, c_to_a],
    };
    let face_key = self.faces.add(face);

    self.edges.get_mut(a_to_b).unwrap().face = face_key;
    self.edges.get_mut(b_to_c).unwrap().face = face_key;
    self.edges.get_mut(c_to_a).unwrap().face = face_key;

    face_key
  }

  /// Regenerates invalid keys.
  ///
  /// # Invariants
  /// Requires that faces have valid and consistently edge keys.
  fn regenerate_invalid_keys(&mut self) {
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
      .cloned()
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
      .cloned()
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
      .cloned()
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
      .cloned()
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

    self.fix_edge_twin_keys();
    self.fix_edge_next_prev_keys();
  }

  fn fix_edge_twin_keys(&mut self) {
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

  /// Triangulates a face.
  ///
  /// # Invariants
  /// The face must not have holes. The arity of the face must be `>= 3`.
  pub fn triangulate(&mut self, face_key: FaceKey) {
    let face = self.faces.get(face_key).unwrap();
    let face_normal = self.face_normal(face_key).unwrap();
    // get edges by next edge
    let mut face_edges = Vec::new();
    let mut next_edge_key = face.edges[0];
    for _ in 0..face.edges.len() {
      let edge = self.edges.get(next_edge_key).unwrap();
      face_edges.push(edge.clone());
      next_edge_key = edge.next_edge;
    }

    // get the positions of the vertices
    let (position_index_to_vertex_key, vertex_positions): (
      Vec<VertexKey>,
      Vec<glam::Vec3A>,
    ) = face_edges
      .iter()
      .map(|edge| {
        (
          edge.origin_vertex,
          self.vertices.get(edge.origin_vertex).unwrap().data.pos(),
        )
      })
      .unzip();

    // project the positions onto the face's plane
    let projected_positions = vertex_positions
      .iter()
      .map(|v| v.project_onto_normalized(face_normal).truncate())
      .collect::<Vec<_>>();
    // flatten the projected positions into a single array
    let flattened_positions = projected_positions
      .iter()
      .flat_map(|v| vec![v.x, v.y])
      .collect::<Vec<_>>();
    // indices of the position triplets that form the triangles
    let indices = earcutr::earcut(&flattened_positions, &[], 2).unwrap();

    // convert the indices into vertex keys
    let triangles: Vec<(VertexKey, VertexKey, VertexKey)> = indices
      .iter()
      .map_windows(|chunk: &[&usize; 3]| {
        (
          position_index_to_vertex_key[*chunk[0]],
          position_index_to_vertex_key[*chunk[1]],
          position_index_to_vertex_key[*chunk[2]],
        )
      })
      .collect::<Vec<_>>();

    // remove the face
    let old_face = self.faces.remove(face_key).unwrap();
    // invalidate the face keys of the old face's edges
    for edge_key in old_face.edges.iter() {
      let edge = self.edges.get_mut(*edge_key).unwrap();
      edge.face = FaceKey::INVALID;
    }

    // add the new faces
    for (a, b, c) in triangles {
      self.add_face(a, b, c);
    }

    // fix everything
    self.regenerate_invalid_keys();
  }

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
