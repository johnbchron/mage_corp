use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;
use tracing::info_span;

use super::{face::Face, MizuMesh, Vertex, VertexData};

struct SimplificationCandidate {
  faces_to_remove: Vec<u32>,
  new_faces:       Vec<Face>,
}

impl<D: VertexData> MizuMesh<D> {
  fn is_first_form_simplifiable_face(
    &self,
    index: u32,
  ) -> Option<SimplificationCandidate> {
    let neighbors = self
      .neighbors(index)
      .iter()
      .filter_map(|n| *n)
      .collect::<Vec<_>>();
    // if not all neighbors exist, abort
    if neighbors.len() != 3 {
      return None;
    }
    // if any of the neighbors are not coplanar, abort
    if neighbors.iter().any(|n| {
      let neighbor = self.face(n.0);
      !self.face(index).are_coplanar(neighbor)
    }) {
      return None;
    }
    // make sure that each pair index from neighbors is unique
    if neighbors.iter().map(|n| n.1).collect::<HashSet<_>>().len() != 3 {
      return None;
    }

    let self_a = self.vertex(self.face(index).vertices().x);
    let self_b = self.vertex(self.face(index).vertices().y);
    let self_c = self.vertex(self.face(index).vertices().z);

    // get outside edges
    // v_o_1e = vertex_opposite_first_edge
    let v_o_1e = self
      .face(neighbors[0].0)
      .vertex_index_opposite_pair(neighbors[0].1);
    let v_o_2e = self
      .face(neighbors[1].0)
      .vertex_index_opposite_pair(neighbors[1].1);
    let v_o_3e = self
      .face(neighbors[2].0)
      .vertex_index_opposite_pair(neighbors[2].1);

    if Vertex::are_collinear(self.vertex(v_o_1e), self_a, self.vertex(v_o_3e))
      && Vertex::are_collinear(self.vertex(v_o_2e), self_b, self.vertex(v_o_1e))
      && Vertex::are_collinear(self.vertex(v_o_3e), self_c, self.vertex(v_o_2e))
    {
      return Some(SimplificationCandidate {
        faces_to_remove: vec![
          index,
          neighbors[0].0,
          neighbors[1].0,
          neighbors[2].0,
        ],
        new_faces:       vec![Face::new(
          glam::UVec3::new(v_o_1e, v_o_2e, v_o_3e),
          self.compute_normal(&glam::UVec3::new(v_o_1e, v_o_2e, v_o_3e)),
        )],
      });
    }
    None
  }

  fn is_second_form_simplifiable_face(
    &self,
    index: u32,
  ) -> Vec<SimplificationCandidate> {
    self
      .neighbors(index)
      .iter()
      .enumerate()
      .filter_map(|(tri_1_start_pair, neighbor)| {
        if neighbor.is_none() {
          return None;
        }

        let tri_1 = index;
        let (tri_2, tri_2_start_pair) = neighbor.unwrap();
        let tri_2_end_pair = (tri_2_start_pair + 2) % 3;
        let Some((tri_3, tri_3_start_pair)) =
          self.opposites()[tri_2 as usize][tri_2_end_pair as usize]
        else {
          return None;
        };
        let tri_3_end_pair = (tri_3_start_pair + 1) % 3;
        let Some((tri_4, tri_4_start_pair)) =
          self.opposites()[tri_3 as usize][tri_3_end_pair as usize]
        else {
          return None;
        };

        // make sure all the faces are coplanar
        if !self.face(tri_1).are_coplanar(self.face(tri_2)) {
          return None;
        }
        if !self.face(tri_1).are_coplanar(self.face(tri_3)) {
          return None;
        }
        if !self.face(tri_1).are_coplanar(self.face(tri_4)) {
          return None;
        }

        let tri_1_vertices = self.face(tri_1).vertices().to_array();
        let tri_4_vertices = self.face(tri_4).vertices().to_array();

        let vertex_a = tri_1_vertices[((tri_1_start_pair + 2) % 3) as usize];
        let vertex_b = tri_1_vertices[tri_1_start_pair as usize];
        let vertex_c = tri_1_vertices[((tri_1_start_pair + 1) % 3) as usize];
        let vertex_d = tri_4_vertices[((tri_4_start_pair + 1) % 3) as usize];
        let vertex_e = tri_4_vertices[tri_4_start_pair as usize];
        let vertex_f = tri_4_vertices[((tri_4_start_pair + 2) % 3) as usize];

        // make sure A, C, and E are collinear
        if !Vertex::are_collinear(
          self.vertex(vertex_a),
          self.vertex(vertex_c),
          self.vertex(vertex_e),
        ) {
          return None;
        }
        // make sure B, D, and F are collinear
        if !Vertex::are_collinear(
          self.vertex(vertex_b),
          self.vertex(vertex_d),
          self.vertex(vertex_f),
        ) {
          return None;
        }

        // make a new face from A, B, and E, and a new face from E, B, and F
        let new_face_1 = Face::new(
          glam::UVec3::new(vertex_a, vertex_b, vertex_e),
          self.compute_normal(&glam::UVec3::new(vertex_a, vertex_b, vertex_e)),
        );
        let new_face_2 = Face::new(
          glam::UVec3::new(vertex_e, vertex_b, vertex_f),
          self.compute_normal(&glam::UVec3::new(vertex_e, vertex_b, vertex_f)),
        );

        Some(SimplificationCandidate {
          faces_to_remove: vec![tri_1, tri_2, tri_3, tri_4],
          new_faces:       vec![new_face_1, new_face_2],
        })
      })
      .collect()
  }

  fn apply_simplification_candidates(
    &mut self,
    mut candidates: Vec<SimplificationCandidate>,
  ) -> usize {
    let _span = info_span!(
      "mosh::MizuMesh::apply_simplification_candidates",
      count = candidates.len()
    )
    .entered();

    let mut to_remove = HashSet::new();
    candidates.retain(|c| {
      let mut can_remove = true;
      for face in c.faces_to_remove.iter() {
        if to_remove.contains(face) {
          can_remove = false;
          break;
        }
      }
      if can_remove {
        for face in c.faces_to_remove.iter() {
          to_remove.insert(*face);
        }
      }
      can_remove
    });

    let mut to_remove = to_remove.into_iter().collect::<Vec<_>>();
    to_remove.sort_unstable();
    to_remove.reverse();

    for i in to_remove {
      self.faces.remove(i as usize);
    }

    let candidate_count = candidates.len();
    for candidate in candidates {
      self.faces.extend(candidate.new_faces);
    }

    self.opposites.take();
    candidate_count
  }

  fn prune_vertices(&mut self) {
    let _span = info_span!(
      "mosh::MizuMesh::prune_vertices",
      vertices = self.vertices.len(),
      faces = self.faces.len()
    )
    .entered();

    let mut vertex_map = HashMap::new();
    let mut new_vertices = Vec::new();

    for face in self.faces.iter() {
      for vertex in face.vertices().to_array().into_iter() {
        if !vertex_map.contains_key(&vertex) {
          vertex_map.insert(vertex, new_vertices.len() as u32);
          new_vertices.push(self.vertices[vertex as usize].clone());
        }
      }
    }

    for face in self.faces.iter_mut() {
      let vertices = face.vertices_mut();
      vertices.x = vertex_map[&vertices.x];
      vertices.y = vertex_map[&vertices.y];
      vertices.z = vertex_map[&vertices.z];
    }

    self.vertices = new_vertices;
    self.opposites.take();
  }

  /// Simplifies the mesh.
  pub fn simplify(&mut self) {
    let _span = info_span!("mosh::MizuMesh::simplify").entered();

    let _sub_span =
      info_span!("mosh::MizuMesh::simplify::get_candidates").entered();
    let candidates = (0..self.faces.len())
      .par_bridge()
      .flat_map(|i| self.is_second_form_simplifiable_face(i as u32))
      .collect::<Vec<_>>();
    drop(_sub_span);

    self.apply_simplification_candidates(candidates);
    self.prune_vertices();
  }
}
