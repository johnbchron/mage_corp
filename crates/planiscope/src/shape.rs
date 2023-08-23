//! Definitions for `Shape`s and `ShapeOp`s.
//!
//! `Shape`s are recursive containers for any number of composed `ShapeOp`s and
//! `ShapeDef`s. Though you can use `Shape`, `ShapeDef`, and `ShapeOp` directly,
//! you should use the builder syntax in the `builder` module.
//!
//! Use the `ShapeLike` trait to implement your own shapes.

use fidget::{context::Node, Context};
use serde::{Deserialize, Serialize};

use crate::{comp::CompilationSettings, nso::*};

/// A trait with methods for compiling Fidget nodes from shape definitions.
pub trait ShapeLike {
  /// Compiles the solid field of a shape.
  fn compile_solid(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node;
  /// Compiles the solid field of a shape, but clamps the result to `1` inside
  /// the shape and `-1` outside.
  fn compile_clamped_solid(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    let shape = self.compile_solid(ctx, settings);
    nso_clamp(shape, ctx)
  }
  /// Compiles the color field of a shape. The resulting value is the 24-bit
  /// representation of the RGB color, divided by 24-bit maximum, and mapped to
  /// the range `[0.1, 1.0]`.
  fn compile_color(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node;
}

/// A shape. Contains either a shape definition or a shape operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Shape {
  /// A shape definition.
  ShapeDef(ShapeDef),
  /// A shape operation.
  ShapeOp(ShapeOp),
}

impl ShapeLike for Shape {
  fn compile_solid(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      Shape::ShapeDef(shape_def) => shape_def.compile_solid(ctx, settings),
      Shape::ShapeOp(shape_op) => shape_op.compile_solid(ctx, settings),
    }
  }
  fn compile_color(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      Shape::ShapeDef(shape_def) => shape_def.compile_color(ctx, settings),
      Shape::ShapeOp(shape_op) => shape_op.compile_color(ctx, settings),
    }
  }
}

/// A shape definition. Shape definitions are pre-defined primitives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeDef {
  SpherePrimitive { radius: f32 },
  RectPrismPrimitive { x: f32, y: f32, z: f32 },
  CubePrimitive { size: f32 },
}

impl ShapeLike for ShapeDef {
  fn compile_solid(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      Self::SpherePrimitive { radius } => {
        if *radius * 2.0 < settings.min_voxel_size {
          return ctx.constant(1.0);
        }
        let r = ctx.constant((*radius).into());
        let x = ctx.x();
        let y = ctx.y();
        let z = ctx.z();
        let x_sq = ctx.square(x).unwrap();
        let y_sq = ctx.square(y).unwrap();
        let z_sq = ctx.square(z).unwrap();
        let r_sq = ctx.square(r).unwrap();
        let sum = ctx.add(x_sq, y_sq).unwrap();
        let sum = ctx.add(sum, z_sq).unwrap();
        ctx.sub(sum, r_sq).unwrap()
      }
      Self::RectPrismPrimitive { x, y, z } => {
        if *x < settings.min_voxel_size
          || *y < settings.min_voxel_size
          || *z < settings.min_voxel_size
        {
          return ctx.constant(1.0);
        }
        let half_x = ctx.constant((*x / 2.0).into());
        let half_y = ctx.constant((*y / 2.0).into());
        let half_z = ctx.constant((*z / 2.0).into());
        let x = ctx.x();
        let y = ctx.y();
        let z = ctx.z();
        let abs_x = ctx.abs(x).unwrap();
        let abs_y = ctx.abs(y).unwrap();
        let abs_z = ctx.abs(z).unwrap();

        let x = ctx.sub(abs_x, half_x).unwrap();
        let y = ctx.sub(abs_y, half_y).unwrap();
        let z = ctx.sub(abs_z, half_z).unwrap();

        let max_xy = ctx.max(x, y).unwrap();
        ctx.max(max_xy, z).unwrap()
      }
      Self::CubePrimitive { size } => Self::RectPrismPrimitive {
        x: *size,
        y: *size,
        z: *size,
      }
      .compile_solid(ctx, settings),
    }
  }
  #[allow(clippy::match_single_binding)]
  fn compile_color(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      _ => {
        let shape = self.compile_clamped_solid(ctx, settings);
        let shape = nso_bleed(shape, 1.1, ctx);

        nso_color(shape, [255, 255, 255], ctx)
      }
    }
  }
}

/// A shape operation. Shape operations are operations between 1 or 2 shapes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeOp {
  /// A unary operation. This takes modifies 1 shape, with the modification
  /// specified in the `UnaryOp` enum.
  UnaryOp(UnaryOp, Box<Shape>),
  /// A binary operation. This takes 2 shapes, and combines them in some way
  /// specified in the `BinaryOp` enum.
  BinaryOp(BinaryOp, Box<Shape>, Box<Shape>),
}

impl ShapeLike for ShapeOp {
  fn compile_solid(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      ShapeOp::UnaryOp(unary_op, a) => {
        unary_op.compile_solid(a.as_ref(), ctx, settings)
      }
      ShapeOp::BinaryOp(binary_op, a, b) => {
        binary_op.compile_solid(a, b, ctx, settings)
      }
    }
  }
  fn compile_color(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      ShapeOp::UnaryOp(unary_op, a) => {
        unary_op.compile_color(a.as_ref(), ctx, settings)
      }
      ShapeOp::BinaryOp(binary_op, a, b) => {
        binary_op.compile_color(a, b, ctx, settings)
      }
    }
  }
}

/// A unary operation. This enum defines the possible unary operations and their
/// parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
  /// Translates a shape by a vector.
  Translate { pos: [f32; 3] },
  /// Scales a shape by a vector.
  Scale { scale: [f32; 3] },
  /// Applies a matrix transform to a shape. This is currently not implemented.
  MatrixTransform { matrix: [f32; 16] },
  /// Recolors a shape to a specific RGB color.
  Recolor { rgb: [u8; 3] },
  /// Abbreviates a shape if it is smaller than a certain threshold. This is
  /// used to reduce voxel inaccuracies in the final model, by eliminating
  /// features smaller than the voxel size.
  Abbreviate { threshold: f32 },
}

impl UnaryOp {
  fn compile_solid(
    &self,
    a: &Shape,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      UnaryOp::Translate { pos } => {
        let shape = a.compile_solid(ctx, settings);
        nso_translate(shape, *pos, ctx)
      }
      UnaryOp::Scale { scale } => {
        let shape = a.compile_solid(ctx, settings);
        nso_scale(shape, *scale, ctx)
      }
      UnaryOp::MatrixTransform { matrix: _ } => {
        todo!();
      }
      UnaryOp::Recolor { .. } => a.compile_solid(ctx, settings),
      UnaryOp::Abbreviate { threshold } => {
        if settings.min_voxel_size < *threshold {
          a.compile_solid(ctx, settings)
        } else {
          ctx.constant(1.0)
        }
      }
    }
  }

  fn compile_color(
    &self,
    a: &Shape,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      UnaryOp::Recolor { rgb } => {
        let shape = a.compile_clamped_solid(ctx, settings);
        let shape = nso_bleed(shape, 1.1, ctx);
        nso_color(shape, *rgb, ctx)
      }
      _ => a.compile_color(ctx, settings),
    }
  }
}

/// A binary operation. This enum defines the possible binary operations and
/// their parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
  /// A union operation. This combines 2 shapes into 1.
  Union,
  /// A difference operation. This subtracts the second shape from the first.
  Difference,
  /// An intersection operation. This takes the intersection of 2 shapes.
  Intersection,
  /// A replacement operation. This is a union operation where the properties
  /// (only color currently) of the first shape are used where the shapes are
  /// overlapping.
  Replacement,
}

impl BinaryOp {
  pub fn compile_solid(
    &self,
    a: &Shape,
    b: &Shape,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      BinaryOp::Union => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        nso_union(a, b, ctx)
      }
      BinaryOp::Difference => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        nso_difference(a, b, ctx)
      }
      BinaryOp::Intersection => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        nso_intersection(a, b, ctx)
      }
      BinaryOp::Replacement => {
        let a = a.compile_solid(ctx, settings);
        let b = b.compile_solid(ctx, settings);
        nso_replacement(a, b, ctx)
      }
    }
  }

  pub fn compile_clamped_solid(
    &self,
    a: &Shape,
    b: &Shape,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    let shape = self.compile_solid(a, b, ctx, settings);
    nso_clamp(shape, ctx)
  }

  pub fn compile_color(
    &self,
    a: &Shape,
    b: &Shape,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    match self {
      BinaryOp::Union => {
        // copy from replacement
        BinaryOp::Replacement.compile_color(a, b, ctx, settings)
      }
      BinaryOp::Difference => {
        let shape = self.compile_clamped_solid(a, b, ctx, settings);
        let color = a.compile_color(ctx, settings);
        ctx.mul(color, shape).unwrap()
      }
      BinaryOp::Intersection => {
        let solid = self.compile_solid(a, b, ctx, settings);
        nso_color(solid, [255, 255, 255], ctx)
      }
      BinaryOp::Replacement => {
        let (a_shape, b_shape) = (a, b);
        let a = a_shape.compile_solid(ctx, settings);
        let b = b_shape.compile_solid(ctx, settings);
        let b = nso_difference(b, a, ctx);
        let b = nso_clamp(b, ctx);
        let a_color = a_shape.compile_color(ctx, settings);
        let b_color = b_shape.compile_color(ctx, settings);
        let b_color = ctx.mul(b_color, b).unwrap();
        ctx.add(a_color, b_color).unwrap()
      }
    }
  }
}
