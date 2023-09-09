pub mod fidget_mesher;
pub mod fsn_mesher;

use fidget::eval::Tape;
use serde::{Deserialize, Serialize};

use crate::comp::Composition;

type MeshingEvaluatorFamily = fidget::vm::Eval;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullMesh {
  pub vertices:  Vec<glam::Vec3A>,
  pub triangles: Vec<glam::UVec3>,
  pub normals:   Vec<glam::Vec3A>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MesherInputs {
  pub position: glam::Vec3A,
  pub scale:    glam::Vec3A,
  pub subdivs:  u8,
  pub prune:    bool,
}

pub struct FidgetMesher;
pub struct FastSurfaceNetsMesher;

pub trait Mesher {
  fn build_mesh(
    comp: &Composition,
    inputs: MesherInputs,
  ) -> Result<FullMesh, fidget::Error>;
}

impl FullMesh {
  /// Transforms the mesh to the desired translation and scale.
  ///
  /// `mesh_new()` produces a mesh only between -1 and 1 on all axes.
  pub fn transform(&mut self, translation: glam::Vec3A, scale: glam::Vec3A) {
    self.vertices.iter_mut().for_each(|v| {
      *v = v.mul_add(scale, translation);
    });
  }

  /// Removes any triangles which have vertices outside of the -1 to 1 range on
  /// any axis.
  pub fn prune(&mut self) {
    // prune triangles outside of the -1 to 1 range on any axis
    const MESH_BLEED: [f32; 3] = [1.0, 1.0, 1.0];
    let violating_verts = self
      .vertices
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

pub fn fidget_normals(
  vertices: &[glam::Vec3A],
  tape: &Tape<MeshingEvaluatorFamily>,
) -> Result<Vec<glam::Vec3A>, fidget::Error> {
  Ok(
    tape
      .new_grad_slice_evaluator()
      .eval(
        &vertices.iter().map(|v| v.x).collect::<Vec<_>>(),
        &vertices.iter().map(|v| v.y).collect::<Vec<_>>(),
        &vertices.iter().map(|v| v.z).collect::<Vec<_>>(),
        &[],
      )?
      .into_iter()
      .map(|g| glam::Vec3A::new(g.dx, g.dy, g.dz))
      .collect(),
  )
}
