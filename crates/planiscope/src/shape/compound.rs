use std::hash::Hasher;

use bevy_reflect::Reflect;
use decorum::hash::FloatHash;
use educe::Educe;
use fidget::{
  context::{IntoNode, Node},
  Context,
};
use serde::{Deserialize, Serialize};

use super::Shape;

#[derive(Educe, Clone, Debug, Serialize, Deserialize, Reflect)]
#[educe(Hash)]
pub enum Compound {
  Sphere {
    #[reflect(ignore)]
    radius: Box<Shape>,
  },
  Cylinder {
    #[reflect(ignore)]
    height: Box<Shape>,
    #[reflect(ignore)]
    radius: Box<Shape>,
  },
  Cuboid {
    #[reflect(ignore)]
    x: Box<Shape>,
    #[reflect(ignore)]
    y: Box<Shape>,
    #[reflect(ignore)]
    z: Box<Shape>,
  },
  SmoothMinCubic {
    #[reflect(ignore)]
    lhs: Box<Shape>,
    #[reflect(ignore)]
    rhs: Box<Shape>,
    #[reflect(ignore)]
    k:   Box<Shape>,
  },
  MatTransform {
    #[reflect(ignore)]
    root: Box<Shape>,
    #[educe(Hash(method = "hash_mat4"))]
    mat:  glam::Mat4,
  },
  Clamp {
    #[reflect(ignore)]
    root: Box<Shape>,
    #[reflect(ignore)]
    min:  Box<Shape>,
    #[reflect(ignore)]
    max:  Box<Shape>,
  },
  Map {
    #[reflect(ignore)]
    root:    Box<Shape>,
    #[reflect(ignore)]
    in_min:  Box<Shape>,
    #[reflect(ignore)]
    in_max:  Box<Shape>,
    #[reflect(ignore)]
    out_min: Box<Shape>,
    #[reflect(ignore)]
    out_max: Box<Shape>,
  },
  CatmullRomSpline {
    #[reflect(ignore)]
    root:    Box<Shape>,
    #[educe(Hash(method = "hash_vec_triplet_f32"))]
    points:  Vec<[f32; 3]>,
    #[educe(Hash(trait = "FloatHash"))]
    tension: f32,
  },
}

impl IntoNode for &Compound {
  fn into_node(self, ctx: &mut Context) -> Result<Node, fidget::Error> {
    match self {
      Compound::Sphere { radius } => {
        let r = radius.into_node(ctx)?;
        crate::nso::volumes::nso_sphere(r, ctx)
      }
      Compound::Cylinder { height, radius } => {
        let height = height.into_node(ctx)?;
        let radius = radius.into_node(ctx)?;
        crate::nso::volumes::nso_cylinder(height, radius, ctx)
      }
      Compound::Cuboid {
        x: length,
        y: width,
        z: height,
      } => {
        let length = length.into_node(ctx)?;
        let width = width.into_node(ctx)?;
        let height = height.into_node(ctx)?;
        crate::nso::volumes::nso_cuboid(length, width, height, ctx)
      }
      Compound::SmoothMinCubic { lhs, rhs, k } => {
        let lhs = lhs.into_node(ctx)?;
        let rhs = rhs.into_node(ctx)?;
        let k = k.into_node(ctx)?;
        crate::nso::smooth::nso_smooth_min_cubic(lhs, rhs, k, ctx)
      }
      Compound::MatTransform { root, mat } => {
        let root = root.into_node(ctx)?;
        crate::nso::regions::nso_matrix_transform(root, mat, ctx)
      }
      Compound::Clamp { root, min, max } => {
        let root = root.into_node(ctx)?;
        let min = min.into_node(ctx)?;
        let max = max.into_node(ctx)?;
        crate::nso::other::nso_clamp(root, min, max, ctx)
      }
      Compound::Map {
        root,
        in_min,
        in_max,
        out_min,
        out_max,
      } => {
        let root = root.into_node(ctx)?;
        let in_min = in_min.into_node(ctx)?;
        let in_max = in_max.into_node(ctx)?;
        let out_min = out_min.into_node(ctx)?;
        let out_max = out_max.into_node(ctx)?;
        crate::nso::other::nso_map(root, in_min, in_max, out_min, out_max, ctx)
      }
      Compound::CatmullRomSpline {
        root: _,
        points: _,
        tension: _,
      } => {
        // let root = root.into_node(ctx)?;
        // crate::nso::spline::nso_catmull_rom_spline(root, points, *tension,
        // ctx)
        todo!()
      }
    }
  }
}

fn hash_mat4<H: Hasher>(s: &glam::Mat4, state: &mut H) {
  s.to_cols_array()
    .iter()
    .for_each(|v| decorum::hash::FloatHash::float_hash(v, state));
}
fn hash_vec_triplet_f32<H: Hasher>(s: &[[f32; 3]], state: &mut H) {
  s.iter().for_each(|a| {
    a.iter()
      .for_each(|v| decorum::hash::FloatHash::float_hash(v, state))
  })
}
