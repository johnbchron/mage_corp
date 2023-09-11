use fast_surface_nets::{
  ndshape::{RuntimeShape, Shape},
  surface_nets, SurfaceNetsBuffer,
};
use fidget::{context::IntoNode, Context};

use crate::{
  mesher::{
    fidget_normals, Composition, FastSurfaceNetsMesher, FullMesh, Mesher,
    MesherInputs, MeshingEvaluatorFamily,
  },
  nso,
};

impl Mesher for FastSurfaceNetsMesher {
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

    // how many units the specified number of subdivisions will produce
    let shape_length: u32 = 2_u32.pow(inputs.subdivs.into());
    // a shape for the purpose of delinearizing in iteration
    let ndshape_descriptor =
      RuntimeShape::<u32, 3>::new([shape_length, shape_length, shape_length]);
    // closure for getting from subdivision u32 units to -1..1 f32 units
    let normalize_shape_coords =
      move |x| (x as f32) / (shape_length as f32 / 2.0) - 1.0;

    // all of the delinearized points from the shape descriptor, in -1..1
    let points = (0u32..ndshape_descriptor.size())
      .map(|x| ndshape_descriptor.delinearize(x))
      .map(|p| {
        glam::Vec3A::new(
          normalize_shape_coords(p[0]),
          normalize_shape_coords(p[1]),
          normalize_shape_coords(p[2]),
        )
      })
      .collect::<Vec<glam::Vec3A>>();

    // evaluate the fidget tape on all of the points
    let evaluator = fidget::eval::FloatSliceEval::new(&tape);
    let values = evaluator.eval(
      &points.iter().map(|v| v.x).collect::<Vec<_>>(),
      &points.iter().map(|v| v.y).collect::<Vec<_>>(),
      &points.iter().map(|v| v.z).collect::<Vec<_>>(),
      &[],
    )?;

    // create a buffer for holding the surface_nets result
    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(
      &values,
      &ndshape_descriptor,
      [0; 3],
      [shape_length - 1; 3],
      &mut buffer,
    );

    // convert vertices and triangles into something we can use (what full_mesh
    // is expecting)
    let vertices = buffer
      .positions
      .iter()
      .map(|a| glam::Vec3A::from_array(*a) / (shape_length as f32 / 2.0) - 1.0)
      .collect::<Vec<glam::Vec3A>>();
    // this uses a chunk operation on the slice because the indices aren't in
    // triplets
    let triangles = buffer
      .indices
      .chunks(3)
      .map(|c| glam::UVec3::from_array([c[0], c[1], c[2]]))
      .collect::<Vec<glam::UVec3>>();

    // get the normals
    let normals: Vec<glam::Vec3A> = fidget_normals(&vertices, &tape)?;

    let mut mesh = FullMesh {
      vertices,
      triangles,
      normals,
    };
    mesh.transform(glam::Vec3A::ZERO, inputs.scale);

    Ok(mesh)
  }
}
