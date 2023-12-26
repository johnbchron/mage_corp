use crate::{
  bufmesh::{BufMesh, FullVertex},
  hedge::Mesh,
};

/// Simplifies a mesh by merging coplanar faces.
pub fn simplify_mesh(mesh: BufMesh) -> BufMesh {
  let triangles = mesh
    .triangles
    .iter()
    .map(|t| (t.x as usize, t.y as usize, t.z as usize))
    .collect::<Vec<_>>();
  let vertices = (0..mesh.positions.len())
    .map(|i| FullVertex {
      position: mesh.positions[i],
      normal:   mesh.normals[i],
    })
    .collect::<Vec<_>>();

  let mut hedge = Mesh::from_buffers(triangles.as_slice(), vertices.as_slice());

  // simplification goes here
  let coplanar_face_groups = hedge.find_coplanar_face_groups();
  for group in coplanar_face_groups {
    let _ =
      hedge.merge_faces(group.iter().copied().collect::<Vec<_>>().as_slice());
  }
  hedge.dedup_equal_vertices();
  hedge.prune_unused_vertices();

  let hedge_buffers = hedge.to_buffers();
  let triangles = hedge_buffers
    .0
    .into_iter()
    .map(|t| glam::UVec3::new(t.0 as u32, t.1 as u32, t.2 as u32))
    .collect::<Vec<_>>();
  let vertices = hedge_buffers
    .1
    .iter()
    .map(|v| v.position)
    .collect::<Vec<_>>();
  let normals = hedge_buffers
    .1
    .into_iter()
    .map(|v| v.normal)
    .collect::<Vec<_>>();

  BufMesh {
    triangles,
    positions: vertices,
    normals,
  }
}

// /// Merges a coplanar group of faces into a single face.
// fn merge_coplanar_group(
//   mesh_graph: &mut MeshGraph<FullVertex>,
//   coplanar_group: &Vec<FaceKey>,
// ) -> Result<FaceKey, GraphError> {
//   let mut master_face_key = coplanar_group[0];
//   let mut merged_faces = vec![master_face_key];

//   loop {
//     let master_face = mesh_graph.face_mut(master_face_key).unwrap();
//     let neighbors = master_face
//       .neighboring_faces()
//       .filter(|f| {
//         coplanar_group.contains(&f.key()) && !merged_faces.contains(&f.key())
//       })
//       .collect::<Vec<_>>();
//     if neighbors.len() == 0 {
//       break;
//     }

//     let neighbor_key = neighbors[0].key();
//     let new_master_face =
//       master_face.merge(plexus::prelude::Selector::ByKey(neighbor_key))?;
//     master_face_key = new_master_face.key();
//     merged_faces.push(neighbor_key);
//   }

//   mesh_graph.face_mut(master_face_key).unwrap().triangulate();

//   Ok(master_face_key)
// }

// fn fullmesh_from_mesh_graph(
//   mesh_graph: &MeshGraph<FullVertex>,
// ) -> Result<FullMesh, GraphError> {
//   let mesh_buffer = mesh_graph
//     .to_mesh_buffer_by_vertex::<U3, u32, FullVertex>()
//     .unwrap();
//   let (triangles, vertices): (Vec<u32>, Vec<FullVertex>) =
//     mesh_buffer.into_raw_buffers();

//   let triangles = triangles
//     .into_iter()
//     .map_windows(|i: &[u32; 3]| glam::UVec3::from_slice(i))
//     .collect::<Vec<_>>();
//   let (vertices, normals): (Vec<_>, Vec<_>) = vertices
//     .into_iter()
//     .map(|v: FullVertex| (v.position, v.normal))
//     .unzip();
//   Ok(FullMesh {
//     vertices,
//     normals,
//     triangles,
//   })
// }
