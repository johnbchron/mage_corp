//! An implementation of an N-dimensional Half-Edge Mesh.

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
  pub fn from_buffers(
    triangles: &[(usize, usize, usize)],
    vertices: &[D],
  ) -> Self {
    let triangles = triangles.to_vec();

    // assert that all triangles are counter-clockwise
    // assert!(
    //   triangles
    //     .iter()
    //     .map(|(a, b, c)| {
    //       let ab = vertices[*b].pos() - vertices[*a].pos();
    //       let ac = vertices[*c].pos() - vertices[*a].pos();
    //       ab.cross(ac).normalize()
    //     })
    //     .all(|n| n.z > 0.0),
    //   "All triangles must be counter-clockwise"
    // );

    let mut vertex_storage = Storage::new();
    let mut edge_storage = Storage::new();
    let mut face_storage = Storage::new();

    // all the vertices as keys in the original order
    let vertex_keys = vertices
      .iter()
      .map(|v| {
        vertex_storage.add(Vertex {
          id:   VertexKey::new(0),
          data: v.clone(),
        })
      })
      .collect::<Vec<_>>();

    triangles.iter().for_each(|(a, b, c)| {
      let face = face_storage.add(Face {
        id:    FaceKey::new(0),
        edges: Vec::new(),
      });

      let ab = edge_storage.add(HalfEdge {
        id: EdgeKey::new(0),
        origin_vertex: vertex_keys[*a],
        target_vertex: vertex_keys[*b],
        face,
        next_edge: EdgeKey::new(0),
        prev_edge: EdgeKey::new(0),
        twin_edge: None,
      });

      let bc = edge_storage.add(HalfEdge {
        id: EdgeKey::new(0),
        origin_vertex: vertex_keys[*b],
        target_vertex: vertex_keys[*c],
        face,
        next_edge: EdgeKey::new(0),
        prev_edge: EdgeKey::new(0),
        twin_edge: None,
      });

      let ca = edge_storage.add(HalfEdge {
        id: EdgeKey::new(0),
        origin_vertex: vertex_keys[*c],
        target_vertex: vertex_keys[*a],
        face,
        next_edge: EdgeKey::new(0),
        prev_edge: EdgeKey::new(0),
        twin_edge: None,
      });

      let edge_mut_ab = edge_storage.get_mut(ab).unwrap();
      edge_mut_ab.id = ab;
      edge_mut_ab.next_edge = bc;
      edge_mut_ab.prev_edge = ca;
      let edge_mut_bc = edge_storage.get_mut(bc).unwrap();
      edge_mut_bc.id = bc;
      edge_mut_bc.next_edge = ca;
      edge_mut_bc.prev_edge = ab;
      let edge_mut_ca = edge_storage.get_mut(ca).unwrap();
      edge_mut_ca.id = ca;
      edge_mut_ca.next_edge = ab;
      edge_mut_ca.prev_edge = bc;

      face_storage
        .get_mut(face)
        .unwrap()
        .edges
        .extend([ab, bc, ca].iter());
    });

    // fill in the twin edges
    let mut vertex_pair_to_edge = HashMap::new();
    for edge in edge_storage.iter() {
      let target_vertex = edge.target_vertex;
      let origin_vertex = edge.origin_vertex;
      vertex_pair_to_edge.insert((origin_vertex, target_vertex), edge.id);
    }
    for edge in edge_storage.iter_mut() {
      let twin_edge_key =
        vertex_pair_to_edge.get(&(edge.target_vertex, edge.origin_vertex));
      edge.twin_edge = twin_edge_key.cloned();
    }

    Self {
      vertices: vertex_storage,
      edges:    edge_storage,
      faces:    face_storage,
    }
  }

  pub fn prune_unused_vertices(&mut self) {
    let mut used_vertices = HashSet::new();
    for edge in self.edges.iter() {
      used_vertices.insert(edge.origin_vertex);
      used_vertices.insert(edge.target_vertex);
    }

    for vertex_key in self.vertices.iter_keys() {
      if !used_vertices.contains(&vertex_key) {
        self.vertices.remove(vertex_key);
      }
    }
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

  pub fn to_buffers(&self) -> (Vec<(usize, usize, usize)>, Vec<D>) {
    assert_eq!(
      self.arity(),
      3,
      "Mesh must be triangular; arity is {}",
      self.arity()
    );

    let mut triangles = vec![];
    let mut vertices = vec![];

    // A map from vertex keys to indices in the new vertex array.
    // The order of the vertices in the new vertex array is sorted from the
    // inner integer value of the vertex keys.
    let mut vertex_map = HashMap::new();
    let mut vertex_keys = self.vertices.iter_keys().collect::<Vec<_>>();
    vertex_keys.sort();
    for (i, vertex_key) in vertex_keys.iter().enumerate() {
      vertex_map.insert(*vertex_key, i);
      vertices.push(self.vertices.get(*vertex_key).unwrap().data.clone());
    }

    // build the triangle array
    for face in self.faces.iter() {
      let face_vertices = face
        .edges
        .iter()
        .map(|edge_key| {
          let edge = self.edges.get(*edge_key).unwrap();
          vertex_map[&edge.origin_vertex]
        })
        .collect::<Vec<_>>();
      triangles.push((face_vertices[0], face_vertices[1], face_vertices[2]));
    }

    (triangles, vertices)
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
