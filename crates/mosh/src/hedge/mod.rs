//! An implementation of an N-dimensional Half-Edge Mesh.

mod buffers;
pub mod keys;
mod storage;

use std::hash::Hash;

use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;

use self::{
  keys::{EdgeKey, FaceKey, OpaqueKey, VertexKey},
  storage::{Storable, Storage},
};

#[derive(Clone, Debug, PartialEq)]
pub struct HalfEdge {
  pub(crate) id:            EdgeKey,
  pub(crate) origin_vertex: VertexKey,
  pub(crate) target_vertex: VertexKey,
  pub(crate) face:          FaceKey,
  pub(crate) next_edge:     EdgeKey,
  pub(crate) prev_edge:     EdgeKey,
  pub(crate) twin_edge:     Option<EdgeKey>,
}

impl Storable for HalfEdge {
  type Key = EdgeKey;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Face {
  pub(crate) id:    FaceKey,
  pub(crate) edges: Vec<EdgeKey>,
}

impl Storable for Face {
  type Key = FaceKey;
}

pub trait VertexData: Clone + PartialEq + Eq + Hash + Sync {
  fn pos(&self) -> glam::Vec3A;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vertex<D: VertexData> {
  pub(crate) id:   VertexKey,
  pub(crate) data: D,
}

impl<D: VertexData> Storable for Vertex<D> {
  type Key = VertexKey;
}

#[derive(Debug)]
pub struct Mesh<D: VertexData> {
  vertices: Storage<VertexKey, Vertex<D>>,
  edges:    Storage<EdgeKey, HalfEdge>,
  faces:    Storage<FaceKey, Face>,
}

impl<D: VertexData> Mesh<D> {
  pub fn prune_unused_vertices(&mut self) {
    let used_vertices = self
      .edges
      .iter()
      .map(|edge| edge.origin_vertex)
      .collect::<HashSet<_>>();

    self.vertices.retain(|k, _| used_vertices.contains(k));
  }

  pub fn dedup_equal_vertices(&mut self) {
    let mut vertex_map = HashMap::new();
    for vertex_key in self.vertices.iter_keys() {
      let vertex = self.vertices.get(vertex_key).unwrap();
      if let Some(duplicate_vertex_key) = vertex_map.get(&vertex.data) {
        for edge_key in self.edges.iter_keys() {
          let edge = self.edges.get_mut(edge_key).unwrap();
          if edge.origin_vertex == vertex_key {
            edge.origin_vertex = *duplicate_vertex_key;
          }
          if edge.target_vertex == vertex_key {
            edge.target_vertex = *duplicate_vertex_key;
          }
        }
        self.vertices.remove(vertex_key);
      } else {
        vertex_map.insert(vertex.data.clone(), vertex_key);
      }
    }
  }

  pub fn face_normal(&self, face: FaceKey) -> Option<glam::Vec3A> {
    let face = self.faces.get(face)?;
    let mut vertex_iter = face.edges.iter().map(|edge| {
      let edge = self.edges.get(*edge).unwrap();
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

  pub fn remove_face(&mut self, face_key: FaceKey) -> Option<FaceKey> {
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
      self.remove_face(*face_key);
    }

    Some(*master_face_key)
  }

  // /// Returns whether or not the mesh can be traversed from one face to
  // another pub fn faces_are_connected(
  //   &self,
  //   face_key: FaceKey,
  //   neighbor_face_key: FaceKey,
  // ) -> bool {
  //   let mut visited_faces = HashSet::new();
  //   let mut stack = vec![face_key];

  //   while let Some(face_key) = stack.pop() {
  //     if visited_faces.contains(&face_key) {
  //       continue;
  //     }
  //     visited_faces.insert(face_key);

  //     if face_key == neighbor_face_key {
  //       return true;
  //     }

  //     for neighbor_face_key in self.face_neighbors(face_key) {
  //       if visited_faces.contains(&neighbor_face_key) {
  //         continue;
  //       }
  //       stack.push(neighbor_face_key);
  //     }
  //   }
  //   false
  // }

  /// Counts the maxiumum arity of the total mesh.
  pub fn arity(&self) -> usize {
    self
      .faces
      .iter()
      .map(|face| face.edges.len())
      .max()
      .unwrap_or(0)
  }
}
