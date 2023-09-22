//! Node-Space Operations.
//!
//! This module contains functions for performing operations in node-space.

use fidget::{context::Node, Context};

// pub struct Vec3Node {
//   pub x: Node, pub y: Node, pub z: Node,
// }

pub mod csg {
  use fidget::{context::Node, Context};

  /// Performs a CSG union between two nodes.
  pub fn nso_csg_union(a: Node, b: Node, ctx: &mut Context) -> Node {
    ctx.max(a, b).unwrap()
  }

  /// Performs a CSG difference between two nodes.
  pub fn nso_csg_difference(a: Node, b: Node, ctx: &mut Context) -> Node {
    let b = ctx.neg(b).unwrap();
    ctx.max(a, b).unwrap()
  }

  /// Performs a CSG intersection between two nodes.
  pub fn nso_csg_intersection(a: Node, b: Node, ctx: &mut Context) -> Node {
    ctx.min(a, b).unwrap()
  }

  /// Performs a CSG union between two nodes, and preserves the value of the
  /// first node where they intersect.
  pub fn nso_csg_replacement(a: Node, b: Node, ctx: &mut Context) -> Node {
    let neg_a = ctx.neg(a).unwrap();
    let b = ctx.min(b, neg_a).unwrap();
    ctx.min(a, b).unwrap()
  }
}

pub mod regions {
  use fidget::{context::Node, Context};

  /// Translates a node by `pos`.
  pub fn nso_translate(shape: Node, pos: [f32; 3], ctx: &mut Context) -> Node {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let pos_x = ctx.constant(pos[0].into());
    let pos_y = ctx.constant(pos[1].into());
    let pos_z = ctx.constant(pos[2].into());
    let new_x = ctx.sub(x, pos_x).unwrap();
    let new_y = ctx.sub(y, pos_y).unwrap();
    let new_z = ctx.sub(z, pos_z).unwrap();
    ctx.remap_xyz(shape, [new_x, new_y, new_z]).unwrap()
  }

  /// Scales a node by `scale`.
  pub fn nso_scale(shape: Node, scale: [f32; 3], ctx: &mut Context) -> Node {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let scale_x = ctx.constant(scale[0].into());
    let scale_y = ctx.constant(scale[1].into());
    let scale_z = ctx.constant(scale[2].into());
    let new_x = ctx.mul(x, scale_x).unwrap();
    let new_y = ctx.mul(y, scale_y).unwrap();
    let new_z = ctx.mul(z, scale_z).unwrap();
    ctx.remap_xyz(shape, [new_x, new_y, new_z]).unwrap()
  }

  /// Transform volume of size `size` centered at `pos` to a unit cube.
  pub fn nso_normalize_region(
    shape: Node,
    pos: [f32; 3],
    size: [f32; 3],
    ctx: &mut Context,
  ) -> Node {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let pos_x = ctx.constant(pos[0].into());
    let pos_y = ctx.constant(pos[1].into());
    let pos_z = ctx.constant(pos[2].into());
    let size_x = ctx.constant(size[0].into());
    let size_y = ctx.constant(size[1].into());
    let size_z = ctx.constant(size[2].into());
    let new_x = ctx.mul(x, size_x).unwrap();
    let new_y = ctx.mul(y, size_y).unwrap();
    let new_z = ctx.mul(z, size_z).unwrap();
    let moved_x = ctx.add(new_x, pos_x).unwrap();
    let moved_y = ctx.add(new_y, pos_y).unwrap();
    let moved_z = ctx.add(new_z, pos_z).unwrap();
    ctx.remap_xyz(shape, [moved_x, moved_y, moved_z]).unwrap()
  }

  /// Transform unit cube volume to a volume of size `size` centered at `pos`.
  /// Reverses `nso_normalize_region` when using identical `pos` and `size`.
  pub fn nso_denormalize_region(
    shape: Node,
    pos: [f32; 3],
    size: [f32; 3],
    ctx: &mut Context,
  ) -> Node {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let pos_x = ctx.constant(pos[0].into());
    let pos_y = ctx.constant(pos[1].into());
    let pos_z = ctx.constant(pos[2].into());
    let size_x = ctx.constant(size[0].into());
    let size_y = ctx.constant(size[1].into());
    let size_z = ctx.constant(size[2].into());
    let new_x = ctx.div(x, size_x).unwrap();
    let new_y = ctx.div(y, size_y).unwrap();
    let new_z = ctx.div(z, size_z).unwrap();
    let moved_x = ctx.sub(new_x, pos_x).unwrap();
    let moved_y = ctx.sub(new_y, pos_y).unwrap();
    let moved_z = ctx.sub(new_z, pos_z).unwrap();
    ctx.remap_xyz(shape, [moved_x, moved_y, moved_z]).unwrap()
  }
}

pub mod vectors {
  use fidget::{context::Node, Context};

  /// Gets the distance of a 2d coordinate from the origin.
  pub fn nso_magnitude_2d(x: Node, y: Node, ctx: &mut Context) -> Node {
    let x2 = ctx.mul(x, x).unwrap();
    let y2 = ctx.mul(y, y).unwrap();
    let sum = ctx.add(x2, y2).unwrap();
    ctx.sqrt(sum).unwrap()
  }

  /// Gets the distance of a 3d coordinate from the origin.
  pub fn nso_magnitude_3d(
    x: Node,
    y: Node,
    z: Node,
    ctx: &mut Context,
  ) -> Node {
    let x2 = ctx.mul(x, x).unwrap();
    let y2 = ctx.mul(y, y).unwrap();
    let z2 = ctx.mul(z, z).unwrap();
    let sum = ctx.add(x2, y2).unwrap();
    let sum = ctx.add(sum, z2).unwrap();
    ctx.sqrt(sum).unwrap()
  }
}

pub mod other {
  use fidget::{context::Node, Context};

  /// Clamps a node to the range [-1, 1], and drastically steepens the slope of
  /// the transition between the two extents.
  pub fn nso_clamp_and_steep(shape: Node, ctx: &mut Context) -> Node {
    let steep_slope = ctx.constant(1000.0);
    let steep_shape = ctx.mul(shape, steep_slope).unwrap();
    let one = ctx.constant(1.0);
    let neg_one = ctx.constant(-1.0);
    let outside_bounded = ctx.min(steep_shape, one).unwrap();
    ctx.max(outside_bounded, neg_one).unwrap()
  }

  /// Clamps and scales a node by the given factor.
  pub fn nso_bleed(shape: Node, factor: f32, ctx: &mut Context) -> Node {
    let shape = nso_clamp_and_steep(shape, ctx);
    let factor = ctx.constant(factor.into());
    let x = ctx.x();
    let new_x = ctx.div(x, factor).unwrap();
    let y = ctx.y();
    let new_y = ctx.div(y, factor).unwrap();
    let z = ctx.z();
    let new_z = ctx.div(z, factor).unwrap();
    ctx.remap_xyz(shape, [new_x, new_y, new_z]).unwrap()
  }

  /// Color a node with the given rgb value. It is recommended to use this on a
  /// node that has had a "bleed" applied to it to reduce the chances of
  /// vertices being clipped.
  pub fn nso_color(shape: Node, rgb: [u8; 3], ctx: &mut Context) -> Node {
    let bitshifted_color =
      rgb[0] as u32 * 256 * 256 + rgb[1] as u32 * 256 + rgb[2] as u32;
    let float_cast_color = bitshifted_color as f32 / (256_u32).pow(3) as f32;
    let color_val = float_cast_color * 0.9 + 0.1;
    let color_val = ctx.constant(color_val.into());

    // convert from -1 inside and 1 outside to 1 inside and 0 outside
    let neg_point_five = ctx.constant(-0.5);
    let one = ctx.constant(1.0);
    let shape = ctx.sub(shape, one).unwrap();
    let shape = ctx.mul(shape, neg_point_five).unwrap();

    // clamp to 0-1
    let zero = ctx.constant(0.0);
    let shape = ctx.max(shape, zero).unwrap();
    let one = ctx.constant(1.0);
    let shape = ctx.min(shape, one).unwrap();

    // multiply by rgb
    ctx.mul(shape, color_val).unwrap()
  }
}

pub mod smooth {
  use fidget::{context::Node, Context};

  pub fn nso_smooth_min_cubic(
    lhs: Node,
    rhs: Node,
    k: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let zero = ctx.constant(0.0);
    let one_over_six = ctx.constant(1.0 / 6.0);

    let v = ctx.sub(lhs, rhs)?;
    let v = ctx.abs(v)?;
    let v = ctx.sub(k, v)?;
    let v = ctx.max(v, zero)?;
    let h = ctx.div(v, k)?;

    let v = ctx.square(h)?;
    let v = ctx.mul(v, h)?;
    let v = ctx.mul(v, k)?;
    let v = ctx.mul(v, one_over_six)?;
    let v2 = ctx.min(lhs, rhs)?;

    ctx.sub(v2, v)
  }
}
