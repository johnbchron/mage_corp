use super::*;

impl<D: VertexData> Mesh<D> {
  /// Builds a mesh from a list of triangles and vertices.
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

  /// Converts a mesh to a list of triangles and vertices.
  ///
  /// # Invariants
  /// The arity of the mesh must be exactly 3.
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
}
