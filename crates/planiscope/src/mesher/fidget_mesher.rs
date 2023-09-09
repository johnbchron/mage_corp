use fidget::{context::IntoNode, mesh::Octree, Context};

use crate::{
  mesher::{
    fidget_normals, Composition, FidgetMesher, FullMesh, Mesher, MesherInputs,
    MeshingEvaluatorFamily,
  },
  nso,
};

impl Mesher for FidgetMesher {
  fn build_mesh(
    comp: &Composition,
    inputs: MesherInputs,
  ) -> Result<FullMesh, fidget::Error> {
    // get a node for the composition
    let mut ctx = Context::new();
    let node = comp.into_node(&mut ctx)?;

    // we need to normalize the target region into -1..1
    let normalized_node = nso::nso_normalize_region(
      node,
      [inputs.position.x, inputs.position.y, inputs.position.z],
      [inputs.scale.x, inputs.scale.y, inputs.scale.z],
      &mut ctx,
    );

    let tape = ctx.get_tape::<MeshingEvaluatorFamily>(normalized_node)?;

    let fidget_meshing_settings = fidget::mesh::Settings {
      threads:   1,
      min_depth: inputs.subdivs,
      max_depth: 1,
    };

    let octree =
      Octree::build::<MeshingEvaluatorFamily>(&tape, fidget_meshing_settings);
    let fidget_mesh = octree.walk_dual(fidget_meshing_settings);

    let vertices: Vec<glam::Vec3A> = fidget_mesh
      .vertices
      .iter()
      .map(|v| glam::Vec3A::new(v.x, v.y, v.z))
      .collect();
    let triangles: Vec<glam::UVec3> = fidget_mesh
      .triangles
      .iter()
      .map(|t| glam::UVec3::new(t[0] as u32, t[1] as u32, t[2] as u32))
      .collect();

    let normals: Vec<glam::Vec3A> = fidget_normals(&vertices, &tape)?;

    let mut mesh = FullMesh {
      vertices,
      triangles,
      normals,
    };
    if inputs.prune {
      mesh.prune();
    }
    mesh.transform(glam::Vec3A::ZERO, inputs.scale);

    Ok(mesh)
  }
}
