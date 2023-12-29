use educe::Educe;
use serde::{Deserialize, Serialize};

use crate::{hash::hash_vec3a, mizu::VertexData};

/// An index-buffer mesh.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufMesh {
  /// The normals attached to the vertices of the mesh.
  pub normals:   Vec<glam::Vec3A>,
  /// The vertex positions of the mesh.
  pub positions: Vec<glam::Vec3A>,
  /// The triangle indices of the mesh.
  pub triangles: Vec<glam::UVec3>,
}

impl BufMesh {
  /// Transforms the mesh to the desired translation and scale.
  ///
  /// `mesh_new()` produces a mesh only between -1 and 1 on all axes.
  pub fn transform(&mut self, translation: glam::Vec3A, scale: glam::Vec3A) {
    self.positions.iter_mut().for_each(|v| {
      *v = v.mul_add(scale, translation);
    });
  }

  /// Removes any triangles which have vertices outside of the -1 to 1 range on
  /// any axis.
  pub fn prune(&mut self) {
    // prune triangles outside of the -1 to 1 range on any axis
    const MESH_BLEED: [f32; 3] = [1.0, 1.0, 1.0];
    let violating_verts = self
      .positions
      .iter()
      // attach an index to each vertex: (usize, Vec3A)
      .enumerate()
      // filter if the absolute value of the vertex is greater than MESH_BLEED
      .filter(|(_, v)| v.abs().cmpgt(MESH_BLEED.into()).any())
      // collect only the indices
      .map(|(i, _)| i)
      .collect::<Vec<usize>>();

    // TODO: optimize. too much iteration.
    self.triangles.retain(|t| {
      violating_verts
        .iter()
        .all(|i| !t.to_array().iter().any(|x| *x == (*i as u32)))
    });
  }
}

/// A vertex with position and normal. Used as the vertex data for the
/// `MizuMesh`.
#[derive(Clone)]
pub struct FullVertex {
  /// The position of the vertex.
  pub position: glam::Vec3A,
  /// The normal of the vertex.
  pub normal:   glam::Vec3A,
}

impl VertexData for FullVertex {
  fn pos(&self) -> glam::Vec3A { self.position }
}
