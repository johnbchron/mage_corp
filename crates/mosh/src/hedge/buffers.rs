use super::*;

impl<D: VertexData> HedgeMesh<D> {
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
          id:   VertexKey::INVALID,
          data: v.clone(),
        })
      })
      .collect::<Vec<_>>();

    triangles.iter().for_each(|(a, b, c)| {
      let face_key = face_storage.add(Face {
        id:    FaceKey::INVALID,
        edges: Vec::new(),
      });

      let ab = edge_storage.add(Edge {
        id:            EdgeKey::INVALID,
        origin_vertex: vertex_keys[*a],
        target_vertex: vertex_keys[*b],
        face:          face_key,
        next_edge:     EdgeKey::INVALID,
        prev_edge:     EdgeKey::INVALID,
        twin_edge:     None,
      });

      let bc = edge_storage.add(Edge {
        id:            EdgeKey::INVALID,
        origin_vertex: vertex_keys[*b],
        target_vertex: vertex_keys[*c],
        face:          face_key,
        next_edge:     EdgeKey::INVALID,
        prev_edge:     EdgeKey::INVALID,
        twin_edge:     None,
      });

      let ca = edge_storage.add(Edge {
        id:            EdgeKey::INVALID,
        origin_vertex: vertex_keys[*c],
        target_vertex: vertex_keys[*a],
        face:          face_key,
        next_edge:     EdgeKey::INVALID,
        prev_edge:     EdgeKey::INVALID,
        twin_edge:     None,
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
        .get_mut(face_key)
        .unwrap()
        .edges
        .extend([ab, bc, ca].iter());
    });

    let mut hedge_mesh = Self {
      vertices: vertex_storage,
      edges:    edge_storage,
      faces:    face_storage,
    };
    hedge_mesh.fix_edge_twin_keys();
    hedge_mesh
  }

  /// Converts a mesh to a list of triangles and vertices.
  ///
  /// # Invariants
  /// The arity of the mesh must be exactly 3.
  pub fn to_buffers(&self) -> (Vec<(usize, usize, usize)>, Vec<D>) {
    if self.faces.is_empty() {
      return (vec![], vec![]);
    }

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
