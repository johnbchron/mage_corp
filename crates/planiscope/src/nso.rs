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
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let w = ctx.constant(1.0);

    let outputs = nso_matrix_mul([x, y, z, w], mat, ctx)?;

    ctx.remap_xyz(root, outputs[0..3].try_into().unwrap())
  }

  pub fn nso_matrix_mul(
    inputs: [Node; 4],
    mat: &glam::Mat4,
    ctx: &mut Context,
  ) -> Result<[Node; 4], fidget::Error> {
    let mat = mat.inverse();

    let x = inputs[0];
    let y = inputs[1];
    let z = inputs[2];
    let w = inputs[3];

    let x1 = ctx.mul(mat.x_axis.x, x)?;
    let x2 = ctx.mul(mat.x_axis.y, x)?;
    let x3 = ctx.mul(mat.x_axis.z, x)?;
    let x4 = ctx.mul(mat.x_axis.w, x)?;
    let y1 = ctx.mul(mat.y_axis.x, y)?;
    let y2 = ctx.mul(mat.y_axis.y, y)?;
    let y3 = ctx.mul(mat.y_axis.z, y)?;
    let y4 = ctx.mul(mat.y_axis.w, y)?;
    let z1 = ctx.mul(mat.z_axis.x, z)?;
    let z2 = ctx.mul(mat.z_axis.y, z)?;
    let z3 = ctx.mul(mat.z_axis.z, z)?;
    let z4 = ctx.mul(mat.z_axis.w, z)?;
    let w1 = ctx.mul(mat.w_axis.x, w)?;
    let w2 = ctx.mul(mat.w_axis.y, w)?;
    let w3 = ctx.mul(mat.w_axis.z, w)?;
    let w4 = ctx.mul(mat.w_axis.w, w)?;

    let c1_sum = ctx.add(x1, y1)?;
    let c1_sum = ctx.add(c1_sum, z1)?;
    let c1_sum = ctx.add(c1_sum, w1)?;
    let c2_sum = ctx.add(x2, y2)?;
    let c2_sum = ctx.add(c2_sum, z2)?;
    let c2_sum = ctx.add(c2_sum, w2)?;
    let c3_sum = ctx.add(x3, y3)?;
    let c3_sum = ctx.add(c3_sum, z3)?;
    let c3_sum = ctx.add(c3_sum, w3)?;
    let c4_sum = ctx.add(x4, y4)?;
    let c4_sum = ctx.add(c4_sum, z4)?;
    let c4_sum = ctx.add(c4_sum, w4)?;

    Ok([c1_sum, c2_sum, c3_sum, c4_sum])
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

  /// Clamps a node to the range [-1, 1], such that any number > 0 is 1, and any
  /// number < 0 is -1. This should not be used with exact 0.
  pub fn nso_normalized_hardstep(
    shape: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let min_self = ctx.min(shape, 0.0)?;
    let max_self = ctx.max(shape, 0.0)?;
    let min_normal = ctx.div(min_self, shape)?;
    let max_normal = ctx.div(max_self, shape)?;
    ctx.sub(max_normal, min_normal)
  }

  pub fn nso_flat_hardstep(
    shape: Node,
    output: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let a = nso_normalized_hardstep(shape, ctx)?;
    let b = ctx.add(a, 1.0)?;
    let c = ctx.div(b, 2.0)?;
    ctx.mul(c, output)
  }

  pub fn nso_hardstep_choice(
    shape: Node,
    out_neg: Node,
    out_pos: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let min_self = ctx.min(shape, 0.0)?;
    let max_self = ctx.min(shape, 0.0)?;
    let min_normal = ctx.div(min_self, shape)?;
    let max_normal = ctx.div(max_self, shape)?;
    let min = ctx.mul(min_normal, out_neg)?;
    let max = ctx.mul(max_normal, out_pos)?;
    ctx.add(max, min)
  }

  /// Outputs output only when region_min < input < region_max, otherwise 0.0.
  pub fn nso_hardstep_region(
    input: Node,
    region_min: Node,
    region_max: Node,
    output: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let a = ctx.neg(input)?;
    let b = ctx.add(a, region_max)?;
    let c = nso_flat_hardstep(b, output, ctx)?;
    let d = ctx.sub(input, region_min)?;
    nso_flat_hardstep(d, c, ctx)
  }

  /// This is functionally equivalent to `(lhs < 0.0 && rhs < 0.0) ? -1.0 : 1.0`
  pub fn nso_normalized_hardstep_negative_and(
    lhs: Node,
    rhs: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let lhs = nso_normalized_hardstep(lhs, ctx)?;
    let rhs = nso_normalized_hardstep(rhs, ctx)?;
    let sum = ctx.add(lhs, rhs)?;
    let sum_plus_one = ctx.add(sum, 1.0)?;
    ctx.min(sum_plus_one, 1.0)
  }

  pub fn nso_clamp(
    shape: Node,
    min: Node,
    max: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let a = ctx.min(shape, max)?;
    ctx.max(a, min)
  }

  pub fn nso_map(
    shape: Node,
    in_min: Node,
    in_max: Node,
    out_min: Node,
    out_max: Node,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let a = ctx.sub(shape, in_min)?;
    let b = ctx.sub(in_max, in_min)?;
    let c = ctx.div(a, b)?;
    let d = ctx.sub(out_max, out_min)?;
    let e = ctx.mul(c, d)?;
    ctx.add(e, out_min)
  }

  // /// Clamps a node to the range [-1, 1], and drastically steepens the slope
  // of /// the transition between the two extents.
  // pub fn nso_clamp_and_steep(
  //   shape: Node,
  //   ctx: &mut Context,
  // ) -> Result<Node, fidget::Error> { let steep_slope = ctx.constant(1000.0);
  //   let steep_shape = ctx.mul(shape, steep_slope)?; let one =
  //   ctx.constant(1.0); let neg_one = ctx.constant(-1.0); let outside_bounded
  //   = ctx.min(steep_shape, one)?; ctx.max(outside_bounded, neg_one)
  // }

  // /// Clamps and scales a node by the given factor.
  // pub fn nso_bleed(
  //   shape: Node,
  //   factor: f32,
  //   ctx: &mut Context,
  // ) -> Result<Node, fidget::Error> { let shape = nso_clamp_and_steep(shape,
  //   ctx)?; let factor = ctx.constant(factor.into()); let x = ctx.x(); let
  //   new_x = ctx.div(x, factor)?; let y = ctx.y(); let new_y = ctx.div(y,
  //   factor)?; let z = ctx.z(); let new_z = ctx.div(z, factor)?;
  //   ctx.remap_xyz(shape, [new_x, new_y, new_z])
  // }

  // /// Color a node with the given rgb value. It is recommended to use this on
  // a /// node that has had a "bleed" applied to it to reduce the chances of
  // /// vertices being clipped.
  // pub fn nso_color(
  //   shape: Node,
  //   rgb: [u8; 3],
  //   ctx: &mut Context,
  // ) -> Result<Node, fidget::Error> { let bitshifted_color = rgb[0] as u32 *
  //   256 * 256 + rgb[1] as u32 * 256 + rgb[2] as u32; let float_cast_color =
  //   bitshifted_color as f32 / (256_u32).pow(3) as f32; let color_val =
  //   float_cast_color * 0.9 + 0.1; let color_val =
  //   ctx.constant(color_val.into());

  //   // convert from -1 inside and 1 outside to 1 inside and 0 outside
  //   let neg_point_five = ctx.constant(-0.5);
  //   let one = ctx.constant(1.0);
  //   let shape = ctx.sub(shape, one)?;
  //   let shape = ctx.mul(shape, neg_point_five)?;

  //   // clamp to 0-1
  //   let zero = ctx.constant(0.0);
  //   let shape = ctx.max(shape, zero)?;
  //   let one = ctx.constant(1.0);
  //   let shape = ctx.min(shape, one)?;

  //   // multiply by rgb
  //   ctx.mul(shape, color_val)
  // }

  #[cfg(test)]
  mod test {
    use float_cmp::approx_eq;

    use super::*;

    #[test]
    fn nso_hardstep_region_works() {
      let mut ctx = Context::new();
      let x = ctx.x();
      let zero = ctx.constant(0.0);
      let one = ctx.constant(1.0);
      let two = ctx.constant(2.0);
      let x_minus_one = ctx.sub(x, one).unwrap();
      let node =
        nso_hardstep_region(x, zero, two, x_minus_one, &mut ctx).unwrap();
      assert!(approx_eq!(
        f64,
        ctx.eval_xyz(node, -1.0, 0.0, 0.0).unwrap(),
        0.0
      ));
      assert!(approx_eq!(
        f64,
        ctx.eval_xyz(node, 0.1, 0.0, 0.0).unwrap(),
        -0.9
      ));
      assert!(approx_eq!(
        f64,
        ctx.eval_xyz(node, 1.9, 0.0, 0.0).unwrap(),
        0.9
      ));
    }
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

pub mod spline {
  use fidget::{context::Node, Context};

  use crate::nso::other::nso_hardstep_region;

  fn to_t(
    root: Node,
    n_segments: usize,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    let zero = ctx.constant(0.0);
    let one = ctx.constant(1.0);
    let n_segments_node = ctx.constant(n_segments as f64);
    let root = super::other::nso_clamp(root, zero, one, ctx)?;
    ctx.mul(root, n_segments_node)
  }

  /// Remaps the path between [0.0, 0.0, 0.0] and [0.0, 0.0, 1.0] to the path of
  /// a Catmull-Rom spline built from the points specified.
  pub fn nso_catmull_rom_spline(
    root: Node,
    points: &Vec<[f32; 3]>,
    tension: f32,
    ctx: &mut Context,
  ) -> Result<Node, fidget::Error> {
    // set out some constants
    let n_segments = points.len() - 1;

    let zero = ctx.constant(0.0);
    let one = ctx.constant(1.0);
    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let t_x = to_t(x, n_segments, ctx)?;
    let t_y = to_t(y, n_segments, ctx)?;
    let t_z = to_t(z, n_segments, ctx)?;

    // get original point set as vectors
    let mut points = points
      .iter()
      .map(|p| glam::Vec3A::from_array(*p))
      .collect::<Vec<_>>();

    // add first and last ghost points
    let first_point = points[0] - (points[1] - points[0]);
    let last_point = points[points.len() - 1]
      + (points[points.len() - 1] - points[points.len() - 2]);
    points.insert(0, first_point);
    points.push(last_point);
    // reassign to remove mutability
    let points = points;

    let mut running_x_axis = ctx.constant(0.0);
    let mut running_y_axis = ctx.constant(0.0);
    let mut running_z_axis = ctx.constant(0.0);

    for i in 0..n_segments {
      let p0 = points[i];
      let p1 = points[i + 1];
      let p2 = points[i + 2];
      let p3 = points[i + 3];
      // println!(
      //   "for round {i}:\n\tgot p0: {p0:?}\n\tgot p1: {p1:?}\n\tgot p2: \
      //    {p2:?}\n\tgot p3: {p3:?}"
      // );

      let matrix = glam::Mat4::from_cols(
        glam::Vec4::new(0.0, -1.0, 2.0, -1.0),
        glam::Vec4::new(2.0, 0.0, -5.0, 3.0),
        glam::Vec4::new(0.0, 1.0, 4.0, -3.0),
        glam::Vec4::new(0.0, 0.0, -1.0, 1.0),
      )
      // .inverse()
        * 0.5;

      let l0 = p0 * matrix.x_axis.x
        + p1 * matrix.y_axis.x
        + p2 * matrix.z_axis.x
        + p3 * matrix.w_axis.x;
      let l1 = p0 * matrix.x_axis.y
        + p1 * matrix.y_axis.y
        + p2 * matrix.z_axis.y
        + p3 * matrix.w_axis.y;
      let l2 = p0 * matrix.x_axis.z
        + p1 * matrix.y_axis.z
        + p2 * matrix.z_axis.z
        + p3 * matrix.w_axis.z;
      let l3 = p0 * matrix.x_axis.w
        + p1 * matrix.y_axis.w
        + p2 * matrix.z_axis.w
        + p3 * matrix.w_axis.w;

      // move t back to [0, 1]
      let i_node = ctx.constant(i as f64);
      let t_x = ctx.sub(t_x, i_node)?;
      let t_y = ctx.sub(t_y, i_node)?;
      let t_z = ctx.sub(t_z, i_node)?;

      // let t0 = ctx.constant(1.0);
      let t1_x = t_x;
      let t2_x = ctx.mul(t1_x, t_x)?;
      let t3_x = ctx.mul(t2_x, t_x)?;
      let t1_y = t_y;
      let t2_y = ctx.mul(t1_y, t_y)?;
      let t3_y = ctx.mul(t2_y, t_y)?;
      let t1_z = t_z;
      let t2_z = ctx.mul(t1_z, t_z)?;
      let t3_z = ctx.mul(t2_z, t_z)?;

      let x_axis_l0 = ctx.constant(l0.x.into());
      let x_axis_l1 = ctx.constant(l1.x.into());
      let x_axis_l2 = ctx.constant(l2.x.into());
      let x_axis_l3 = ctx.constant(l3.x.into());

      let x_axis_a = ctx.mul(x_axis_l1, t1_x)?;
      let x_axis_b = ctx.mul(x_axis_l2, t2_x)?;
      let x_axis_c = ctx.mul(x_axis_l3, t3_x)?;
      let x_axis_sum = ctx.add(x_axis_l0, x_axis_a)?;
      let x_axis_sum = ctx.add(x_axis_sum, x_axis_b)?;
      let x_axis_sum = ctx.add(x_axis_sum, x_axis_c)?;

      let y_axis_l0 = ctx.constant(l0.y.into());
      let y_axis_l1 = ctx.constant(l1.y.into());
      let y_axis_l2 = ctx.constant(l2.y.into());
      let y_axis_l3 = ctx.constant(l3.y.into());

      let y_axis_a = ctx.mul(y_axis_l1, t1_y)?;
      let y_axis_b = ctx.mul(y_axis_l2, t2_y)?;
      let y_axis_c = ctx.mul(y_axis_l3, t3_y)?;
      let y_axis_sum = ctx.add(y_axis_l0, y_axis_a)?;
      let y_axis_sum = ctx.add(y_axis_sum, y_axis_b)?;
      let y_axis_sum = ctx.add(y_axis_sum, y_axis_c)?;

      let z_axis_l0 = ctx.constant(l0.z.into());
      let z_axis_l1 = ctx.constant(l1.z.into());
      let z_axis_l2 = ctx.constant(l2.z.into());
      let z_axis_l3 = ctx.constant(l3.z.into());

      let z_axis_a = ctx.mul(z_axis_l1, t1_z)?;
      let z_axis_b = ctx.mul(z_axis_l2, t2_z)?;
      let z_axis_c = ctx.mul(z_axis_l3, t3_z)?;
      let z_axis_sum = ctx.add(z_axis_l0, z_axis_a)?;
      let z_axis_sum = ctx.add(z_axis_sum, z_axis_b)?;
      let z_axis_sum = ctx.add(z_axis_sum, z_axis_c)?;

      let x_axis = nso_hardstep_region(t_x, zero, one, x_axis_sum, ctx)?;
      let y_axis = nso_hardstep_region(t_y, zero, one, y_axis_sum, ctx)?;
      let z_axis = nso_hardstep_region(t_z, zero, one, z_axis_sum, ctx)?;

      *(&mut running_x_axis) = ctx.add(running_x_axis, x_axis)?;
      *(&mut running_y_axis) = ctx.add(running_y_axis, y_axis)?;
      *(&mut running_z_axis) = ctx.add(running_z_axis, z_axis)?;
    }

    let x = ctx.x();
    let y = ctx.y();
    let z = ctx.z();
    let new_x = ctx.sub(x, running_x_axis)?;
    // let y = ctx.div(y, 2.0)?;
    let new_y = ctx.sub(y, running_y_axis)?;
    // let new_y = running_y_axis;
    let new_z = ctx.sub(z, running_z_axis)?;

    // for t in 0..21 {
    //   let t = t as f64 / 20.0;
    //   println!(
    //     "x: {:?}, y: {:?}, z: {:?}",
    //     ctx.eval_xyz(new_x, 0.0, t, 0.0)?,
    //     ctx.eval_xyz(new_y, 0.0, t, 0.0)?,
    //     ctx.eval_xyz(new_z, 0.0, t, 0.0)?
    //   );
    // }

    // ctx.remap_xyz(root, [new_x, new_y, new_z])
    ctx.remap_xyz(root, [running_x_axis, running_y_axis, running_z_axis])
  }
}
