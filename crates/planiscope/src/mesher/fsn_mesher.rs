use fast_surface_nets::{
  ndshape::{RuntimeShape, Shape},
  surface_nets, SurfaceNetsBuffer,
};
use fidget::{eval::Tape, Context};
use mosh::BufMesh;
use tracing::info_span;

use crate::{
  mesher::{fidget_normals, FastSurfaceNetsMesher, Mesher, MesherInputs},
  nso,
  shape::CachedIntoNode,
};

impl Mesher for FastSurfaceNetsMesher {
  type EvalFamily = fidget::vm::Eval;

  fn build_mesh(
    &self,
    inputs: &MesherInputs,
  ) -> Result<BufMesh, fidget::Error> {
    let _span =
      info_span!("plansicope::FastSurfaceNetsMesher::build_mesh").entered();

    // get a node for the composition
    let mut ctx = Context::new();
    let node = inputs.shape.eval_root_cached(&mut ctx)?;

    // we need to normalize the target region into -1..1
    let normalized_node = nso::regions::nso_normalize_region(
      node,
      inputs.region.position.to_array(),
      inputs.region.scale.to_array(),
      &mut ctx,
    )?;

    let tape = ctx.get_tape::<Self::EvalFamily>(normalized_node)?;
    let tape = simplify_tape(tape, [[-1.0, 1.0]; 3])?;

    // how many units the specified number of subdivisions will produce
    let shape_length = inputs.region.voxel_side_length();

    // a shape for the purpose of delinearizing in iteration
    let ndshape_descriptor = RuntimeShape::<u32, 3>::new(shape_length);
    // closure for getting from voxel units to -1..1 f32 units, i.e. node coords
    // let voxel_to_node_coords =
    // move |x| (x as f32) / (shape_length as f32 / 2.0) - 1.0;

    // all of the delinearized points from the shape descriptor, in -1..1
    let points = (0u32..ndshape_descriptor.size())
      .map(|x| ndshape_descriptor.delinearize(x))
      .map(|p| {
        glam::UVec3::from_array(p).as_vec3a()
          / (glam::UVec3::from_array(shape_length).as_vec3a() / 2.0)
          - 1.0
      })
      .collect::<Vec<glam::Vec3A>>();

    let eval_span =
      info_span!("fidget_point_eval", points = points.len()).entered();
    // evaluate the fidget tape on all of the points
    let evaluator = fidget::eval::FloatSliceEval::new(&tape);
    let values = evaluator.eval(
      &points.iter().map(|v| v.x).collect::<Vec<_>>(),
      &points.iter().map(|v| v.y).collect::<Vec<_>>(),
      &points.iter().map(|v| v.z).collect::<Vec<_>>(),
      &[],
    )?;
    drop(eval_span);

    let surface_nets_span = info_span!("surface_nets").entered();
    // create a buffer for holding the surface_nets result
    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(
      &values,
      &ndshape_descriptor,
      [0; 3],
      (glam::UVec3::from_array(shape_length) - 1).to_array(),
      &mut buffer,
    );
    drop(surface_nets_span);

    // convert vertices and triangles into something we can use (what full_mesh
    // is expecting), and scale them back up for the normal calc.
    let positions = buffer
      .positions
      .iter()
      // this is to convert from linearized integer coords back to -1..1
      .map(|a| {
        glam::Vec3A::from_array(*a)
          / (glam::UVec3::from_array(shape_length).as_vec3a() / 2.0)
          - 1.0
      })
      // this is to go from -1..1 to the normal scale
      .collect::<Vec<glam::Vec3A>>();
    // this uses a chunk operation on the slice because the indices aren't in
    // triplets
    let triangles = buffer
      .indices
      .chunks(3)
      .map(|c| glam::UVec3::from_array([c[0], c[1], c[2]]))
      .collect::<Vec<glam::UVec3>>();

    // get the normals
    let normals: Vec<glam::Vec3A> = fidget_normals(&positions, &tape)?;

    let mut mesh = BufMesh {
      positions,
      triangles,
      normals,
    };

    mesh.transform(glam::Vec3A::ZERO, inputs.region.scale);
    let mesh = if inputs.region.simplify {
      mosh::simplify_mesh(mesh)
    } else {
      mesh
    };

    Ok(mesh)
  }
}

fn simplify_tape<F: fidget::eval::Family>(
  tape: Tape<F>,
  region: [[f32; 2]; 3],
) -> Result<Tape<F>, fidget::Error> {
  let interval_eval = tape.new_interval_evaluator();
  let (_, simplify) =
    interval_eval.eval(region[0], region[1], region[2], &[])?;
  match simplify {
    Some(simplify) => simplify.simplify(),
    None => Ok(tape),
  }
}
