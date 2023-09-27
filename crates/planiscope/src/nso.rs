//! Node-Space Operations.
//!
//! This module contains functions for performing operations in node-space.

pub mod volumes {
  use fidget::{context::Node, Context};

  pub fn nso_sphere(r: Node, ctx: &mut Context) -> Result<Node, fidget::Error> {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let x_squared = ctx.square(x)?;
    let y_squared = ctx.square(y)?;
    let z_squared = ctx.square(z)?;
    let sum_x_y = ctx.add(x_squared, y_squared)?;
    let sum = ctx.add(sum_x_y, z_squared)?;
    let sqrt = ctx.sqrt(sum)?;
    ctx.sub(sqrt, r)
  }

  pub fn nso_cylinder(
    height: Node,
    radius: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    // this formula was derived with height being height from origin, not
    // total height, so we're halving it to get total height.
    let height = ctx.div(height, 2.0)?;

    let dist_xz = super::vectors::nso_magnitude_2d([x, z], ctx)?;
    let v1 = ctx.sub(radius, dist_xz)?;
    let abs_y = ctx.abs(y)?;
    let v2 = ctx.sub(height, abs_y)?;

    let f = ctx.min(v1, v2)?;
    ctx.neg(f)
  }

  pub fn nso_cylinder_precise(
    height: Node,
    radius: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    // this formula was derived with height being height from origin, not
    // total height, so we're halving it to get total height.
    let height = ctx.div(height, 2.0)?;

    let vx = super::vectors::nso_magnitude_2d([x, z], ctx)?;
    let vy = y;
    let wx = radius;
    let wy = height;
    let vx = ctx.abs(vx)?;
    let vy = ctx.abs(vy)?;
    let dx = ctx.sub(vx, wx)?;
    let dy = ctx.sub(vy, wy)?;

    let f1 = ctx.max(dx, dy)?;
    let f1 = ctx.min(f1, 0.0)?;
    let f2a = ctx.max(dx, 0.0)?;
    let f2b = ctx.max(dy, 0.0)?;
    let f2 = super::vectors::nso_magnitude_2d([f2a, f2b], ctx)?;

    ctx.add(f1, f2)
  }
}

pub mod csg {
  use fidget::{context::Node, Context};

  /// Performs a CSG union between two nodes.
  pub fn nso_csg_union(
    a: Node,
    b: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    ctx.max(a, b)
  }

  /// Performs a CSG difference between two nodes.
  pub fn nso_csg_difference(
    a: Node,
    b: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let b = ctx.neg(b)?;
    ctx.max(a, b)
  }

  /// Performs a CSG intersection between two nodes.
  pub fn nso_csg_intersection(
    a: Node,
    b: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    ctx.min(a, b)
  }

  /// Performs a CSG union between two nodes, and preserves the value of the
  /// first node where they intersect.
  pub fn nso_csg_replacement(
    a: Node,
    b: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let neg_a = ctx.neg(a)?;
    let b = ctx.min(b, neg_a)?;
    ctx.min(a, b)
  }
}

pub mod regions {
  use fidget::{context::Node, Context};

  /// Translates a node by `pos`.
  pub fn nso_translate(
    shape: Node,
    pos: [f32; 3],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let pos_x = ctx.constant(pos[0].into());
    let pos_y = ctx.constant(pos[1].into());
    let pos_z = ctx.constant(pos[2].into());
    let new_x = ctx.sub(x, pos_x)?;
    let new_y = ctx.sub(y, pos_y)?;
    let new_z = ctx.sub(z, pos_z)?;
    ctx.remap_xyz(shape, [new_x, new_y, new_z])
  }

  /// Scales a node by `scale`.
  pub fn nso_scale(
    shape: Node,
    scale: [f32; 3],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let scale_x = ctx.constant(scale[0].into());
    let scale_y = ctx.constant(scale[1].into());
    let scale_z = ctx.constant(scale[2].into());
    let new_x = ctx.mul(x, scale_x)?;
    let new_y = ctx.mul(y, scale_y)?;
    let new_z = ctx.mul(z, scale_z)?;
    ctx.remap_xyz(shape, [new_x, new_y, new_z])
  }

  /// Transform volume of size `size` centered at `pos` to a unit cube.
  pub fn nso_normalize_region(
    shape: Node,
    pos: [f32; 3],
    size: [f32; 3],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let pos_x = ctx.constant(pos[0].into());
    let pos_y = ctx.constant(pos[1].into());
    let pos_z = ctx.constant(pos[2].into());
    let size_x = ctx.constant(size[0].into());
    let size_y = ctx.constant(size[1].into());
    let size_z = ctx.constant(size[2].into());
    let new_x = ctx.mul(x, size_x)?;
    let new_y = ctx.mul(y, size_y)?;
    let new_z = ctx.mul(z, size_z)?;
    let moved_x = ctx.add(new_x, pos_x)?;
    let moved_y = ctx.add(new_y, pos_y)?;
    let moved_z = ctx.add(new_z, pos_z)?;
    let new_root = ctx.remap_xyz(shape, [moved_x, moved_y, moved_z])?;
    ctx.div(new_root, size[0])
  }

  /// Transform unit cube volume to a volume of size `size` centered at `pos`.
  /// Reverses `nso_normalize_region` when using identical `pos` and `size`.
  pub fn nso_denormalize_region(
    shape: Node,
    pos: [f32; 3],
    size: [f32; 3],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let pos_x = ctx.constant(pos[0].into());
    let pos_y = ctx.constant(pos[1].into());
    let pos_z = ctx.constant(pos[2].into());
    let size_x = ctx.constant(size[0].into());
    let size_y = ctx.constant(size[1].into());
    let size_z = ctx.constant(size[2].into());
    let new_x = ctx.div(x, size_x)?;
    let new_y = ctx.div(y, size_y)?;
    let new_z = ctx.div(z, size_z)?;
    let moved_x = ctx.sub(new_x, pos_x)?;
    let moved_y = ctx.sub(new_y, pos_y)?;
    let moved_z = ctx.sub(new_z, pos_z)?;
    ctx.remap_xyz(shape, [moved_x, moved_y, moved_z])
  }

  pub fn nso_matrix_transform(
    root: Node,
    mat: &glam::Mat4,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let mat = mat.inverse();

    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();

    let x1 = ctx.mul(mat.x_axis.x, x)?;
    let x2 = ctx.mul(mat.x_axis.y, x)?;
    let x3 = ctx.mul(mat.x_axis.z, x)?;
    // let x4 = ctx.mul(mat.x_axis.w, x)?;
    let y1 = ctx.mul(mat.y_axis.x, y)?;
    let y2 = ctx.mul(mat.y_axis.y, y)?;
    let y3 = ctx.mul(mat.y_axis.z, y)?;
    // let y4 = ctx.mul(mat.y_axis.w, y)?;
    let z1 = ctx.mul(mat.z_axis.x, z)?;
    let z2 = ctx.mul(mat.z_axis.y, z)?;
    let z3 = ctx.mul(mat.z_axis.z, z)?;
    // let z4 = ctx.mul(mat.z_axis.w, z)?;
    let w1 = ctx.mul(mat.w_axis.x, 1.0)?;
    let w2 = ctx.mul(mat.w_axis.y, 1.0)?;
    let w3 = ctx.mul(mat.w_axis.z, 1.0)?;
    // let w4 = ctx.mul(mat.w_axis.w, 1.0)?;

    let c1_sum = ctx.add(x1, y1)?;
    let c1_sum = ctx.add(c1_sum, z1)?;
    let c1_sum = ctx.add(c1_sum, w1)?;
    let c2_sum = ctx.add(x2, y2)?;
    let c2_sum = ctx.add(c2_sum, z2)?;
    let c2_sum = ctx.add(c2_sum, w2)?;
    let c3_sum = ctx.add(x3, y3)?;
    let c3_sum = ctx.add(c3_sum, z3)?;
    let c3_sum = ctx.add(c3_sum, w3)?;
    // let c4_sum = ctx.add(x4, y4)?;
    // let c4_sum = ctx.add(c4_sum, z4)?;
    // let c4_sum = ctx.add(c4_sum, w4)?;

    ctx.remap_xyz(root, [c1_sum, c2_sum, c3_sum])
  }
}

pub mod vectors {
  use fidget::{context::Node, Context};

  /// Gets the distance of a 2d coordinate from the origin.
  pub fn nso_magnitude_2d(
    v: [Node; 2],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x2 = ctx.square(v[0])?;
    let y2 = ctx.square(v[1])?;
    let sum = ctx.add(x2, y2)?;
    ctx.sqrt(sum)
  }

  /// Gets the distance of a 3d coordinate from the origin.
  pub fn nso_magnitude_3d(
    v: [Node; 3],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let x2 = ctx.square(v[0])?;
    let y2 = ctx.square(v[1])?;
    let z2 = ctx.square(v[2])?;
    let sum = ctx.add(x2, y2)?;
    let sum = ctx.add(sum, z2)?;
    ctx.sqrt(sum)
  }

  /// Returns the dot product of the given 2d vectors.
  pub fn nso_dot_product_2d(
    a: [Node; 2],
    b: [Node; 2],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let v0 = ctx.mul(a[0], b[0])?;
    let v1 = ctx.mul(a[1], b[1])?;
    ctx.add(v0, v1)
  }

  /// Returns the dot product of the given 2d vectors.
  pub fn nso_dot_product_3d(
    a: [Node; 3],
    b: [Node; 3],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let v0 = ctx.mul(a[0], b[0])?;
    let v1 = ctx.mul(a[1], b[1])?;
    let v2 = ctx.mul(a[2], a[2])?;
    let sum = ctx.add(v0, v1)?;
    ctx.add(sum, v2)
  }
}

pub mod other {
  use fidget::{context::Node, Context};

  /// Clamps a node to the range [-1, 1], and drastically steepens the slope of
  /// the transition between the two extents.
  pub fn nso_clamp_and_steep(
    shape: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let steep_slope = ctx.constant(1000.0);
    let steep_shape = ctx.mul(shape, steep_slope)?;
    let one = ctx.constant(1.0);
    let neg_one = ctx.constant(-1.0);
    let outside_bounded = ctx.min(steep_shape, one)?;
    ctx.max(outside_bounded, neg_one)
  }

  /// Clamps and scales a node by the given factor.
  pub fn nso_bleed(
    shape: Node,
    factor: f32,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let shape = nso_clamp_and_steep(shape, ctx)?;
    let factor = ctx.constant(factor.into());
    let x = ctx.x();
    let new_x = ctx.div(x, factor)?;
    let y = ctx.y();
    let new_y = ctx.div(y, factor)?;
    let z = ctx.z();
    let new_z = ctx.div(z, factor)?;
    ctx.remap_xyz(shape, [new_x, new_y, new_z])
  }

  /// Color a node with the given rgb value. It is recommended to use this on a
  /// node that has had a "bleed" applied to it to reduce the chances of
  /// vertices being clipped.
  pub fn nso_color(
    shape: Node,
    rgb: [u8; 3],
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let bitshifted_color =
      rgb[0] as u32 * 256 * 256 + rgb[1] as u32 * 256 + rgb[2] as u32;
    let float_cast_color = bitshifted_color as f32 / (256_u32).pow(3) as f32;
    let color_val = float_cast_color * 0.9 + 0.1;
    let color_val = ctx.constant(color_val.into());

    // convert from -1 inside and 1 outside to 1 inside and 0 outside
    let neg_point_five = ctx.constant(-0.5);
    let one = ctx.constant(1.0);
    let shape = ctx.sub(shape, one)?;
    let shape = ctx.mul(shape, neg_point_five)?;

    // clamp to 0-1
    let zero = ctx.constant(0.0);
    let shape = ctx.max(shape, zero)?;
    let one = ctx.constant(1.0);
    let shape = ctx.min(shape, one)?;

    // multiply by rgb
    ctx.mul(shape, color_val)
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
