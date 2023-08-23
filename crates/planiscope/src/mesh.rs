//! Provides a wrapper for tesselating and converting to Bevy meshes.

use bevy_render::mesh::Mesh as BevyMesh;
use fidget::{
  eval::{Family, Tape},
  mesh::{Mesh as FidgetMesh, Octree, Settings},
};

/// A wrapper around a mesh and its attributes.
///
/// A `FullMesh` can be converted into a `bevy::render::mesh::Mesh` using
/// `From`.
#[derive(Clone)]
pub struct FullMesh {
  pub vertices:  Vec<glam::Vec3A>,
  pub triangles: Vec<glam::UVec3>,
  pub normals:   Option<Vec<glam::Vec3A>>,
  pub colors:    Option<Vec<glam::Vec4>>,
}

impl FullMesh {
  /// Tesselates a solid and color tape into a mesh.
  ///
  /// # Arguments
  /// * `solid_tape` - the solid tape
  /// * `color_tape` - the color tape
  /// * `smooth_normals` - whether to calculate normals from evaluating the
  ///   gradient
  /// from the solid tape
  /// * `max_depth` - the maximum depth of the octree, i.e. the maximum number
  /// of times to divide the starting -1 to 1 volume. Increasing this will
  /// result in detailed portions of the tape having more vertices.
  /// * `min_depth` - the minimum depth of the octree, i.e. the minimum number
  /// of times to divide the starting -1 to 1 volume. Increasing this will
  /// result in larger portions of the resulting mesh being subdivided to this
  /// threshold of divisions.
  pub fn tesselate<T: Family>(
    solid_tape: &Tape<T>,
    color_tape: Option<&Tape<T>>,
    smooth_normals: bool,
    max_depth: u8,
    min_depth: u8,
  ) -> Self {
    let settings = Settings {
      threads:   6,
      // no this is not a typo. I think that these are named opposite of what
      // they should be. the smallest voxel, represented by `min_depth` is at
      // the maximum depth. the largest voxel, represented by `max_depth` is at
      // the minimum depth.
      //
      // regardless, there be dragons here.
      min_depth: max_depth,
      max_depth: min_depth,
    };

    println!("building octree");
    let octree = Octree::build::<T>(solid_tape, settings);
    let fidget_mesh = octree.walk_dual(settings);
    println!("octree built");

    // this converts from nalgebra vectors to glam vectors
    println!("transforming vertices");
    let vertices = fidget_mesh
      .vertices
      .iter()
      .map(|v| glam::Vec3A::new(v.x, v.y, v.z))
      .collect();
    println!("vertices transformed");

    // same conversion here
    println!("transforming triangles");
    let triangles = fidget_mesh
      .triangles
      .iter()
      .map(|t| glam::UVec3::new(t[0] as u32, t[1] as u32, t[2] as u32))
      .collect();
    println!("triangles transformed");

    let normals = if smooth_normals {
      println!("calculating normals from surface");
      let normals = implicit_normals(&fidget_mesh, solid_tape);
      println!("normals calculated");
      Some(normals)
    } else {
      None
    };

    let colors = if let Some(color_tape) = color_tape {
      println!("calculating colors from surface");
      let colors = implicit_colors(&fidget_mesh, color_tape);
      println!("colors calculated");
      Some(colors)
    } else {
      None
    };

    FullMesh {
      vertices,
      triangles,
      normals,
      colors,
    }
  }

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

impl From<FullMesh> for BevyMesh {
  fn from(mesh: FullMesh) -> Self {
    let mut bevy_mesh =
      BevyMesh::new(bevy_render::mesh::PrimitiveTopology::TriangleList);
    bevy_mesh.insert_attribute(
      BevyMesh::ATTRIBUTE_POSITION,
      mesh
        .vertices
        .clone()
        .into_iter()
        .map(Into::<[f32; 3]>::into)
        .collect::<Vec<_>>(),
    );
    if let Some(normals) = mesh.normals {
      bevy_mesh.insert_attribute(
        BevyMesh::ATTRIBUTE_NORMAL,
        normals
          .into_iter()
          .map(Into::<[f32; 3]>::into)
          .collect::<Vec<_>>(),
      );
    } else {
      bevy_mesh.duplicate_vertices();
      bevy_mesh.compute_flat_normals();
    }
    if let Some(colors) = mesh.colors {
      bevy_mesh.insert_attribute(
        BevyMesh::ATTRIBUTE_COLOR,
        colors
          .iter()
          .map(|c| [c.x, c.y, c.z, c.w])
          .collect::<Vec<_>>(),
      );
    }
    bevy_mesh.set_indices(Some(bevy_render::mesh::Indices::U32(
      mesh
        .triangles
        .into_iter()
        .flat_map(|v| [v.x, v.y, v.z])
        .collect(),
    )));
    bevy_mesh
  }
}

// TODO: don't use panic
fn implicit_normals<T: Family>(
  mesh: &FidgetMesh,
  tape: &Tape<T>,
) -> Vec<glam::Vec3A> {
  let eval = tape.new_grad_slice_evaluator();
  // let mut normals: Vec<glam::Vec3A> = vec![];
  let grad = eval.eval(
    &mesh.vertices.iter().map(|v| v.x).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.y).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.z).collect::<Vec<_>>(),
    &[],
  );
  match grad {
    Err(_) => panic!("normal evaluation failed"),
    Ok(grad) => grad
      .into_iter()
      .map(|g| glam::Vec3A::new(g.dx, g.dy, g.dz))
      .collect(),
  }
}

#[allow(dead_code)]
fn flat_normals(
  triangles: Vec<glam::UVec3>,
  vertices: Vec<glam::Vec3A>,
) -> Vec<glam::Vec3A> {
  let mut normals: Vec<glam::Vec3A> = vec![];
  for t in triangles.iter() {
    let v0 = vertices[t[0] as usize];
    let v1 = vertices[t[1] as usize];
    let v2 = vertices[t[2] as usize];
    let normal = (v1 - v0).cross(v2 - v0).normalize();
    normals.push(glam::Vec3A::new(normal.x, normal.y, normal.z));
    normals.push(glam::Vec3A::new(normal.x, normal.y, normal.z));
    normals.push(glam::Vec3A::new(normal.x, normal.y, normal.z));
  }
  normals
}

// TODO: refactor this to actually use bulk evaluators
fn implicit_colors<T: Family>(
  mesh: &FidgetMesh,
  tape: &Tape<T>,
) -> Vec<glam::Vec4> {
  let eval = tape.new_float_slice_evaluator();

  let grad = eval.eval(
    &mesh.vertices.iter().map(|v| v.x).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.y).collect::<Vec<_>>(),
    &mesh.vertices.iter().map(|v| v.z).collect::<Vec<_>>(),
    &[],
  );

  match grad {
    Err(_) => panic!("color evaluation failed"),
    Ok(grad) => grad.into_iter().map(transform_implicit_color).collect(),
  }
}

fn transform_implicit_color(val: f32) -> glam::Vec4 {
  // we offset the hue by a bit when it gets set to avoid sampling red when
  // sampling noise
  if val < 0.1 {
    return glam::Vec4::new(1.0, 1.0, 1.0, 1.0);
  }

  // put it back in the normal range
  let val = (val - 0.1) / 0.9;

  let val = val * (256_u32.pow(3)) as f32;
  // bit shift to get the original values
  let red = ((val as u32) >> 16) as f32;
  let green = (((val as u32) << 16) >> 24) as f32;
  let blue = (((val as u32) << 24) >> 24) as f32;

  glam::Vec4::new(red / 255.0, green / 255.0, blue / 255.0, 1.0)
}
