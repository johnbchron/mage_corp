//! Provides `Composition`, a collection of shapes.

use fidget::{context::Node, Context};

use crate::{
  nso::nso_translate,
  shape::{Shape, ShapeLike},
};

type Position = [f32; 3];

/// Settings for compiling a `Composition` into node-space.
#[derive(Debug, Clone)]
pub struct CompilationSettings {
  /// The minimum voxel size that can be used for a shape.
  /// This will cause `ShapeOp::Abbreviate` to drop its contents if its
  /// threshold is less than this.
  pub min_voxel_size: f32,
}

/// A collection of shapes.
///
/// Use `Composition::new()` to create an empty `Composition`. Use `add_shape()`
/// to add a shape to the composition. Use `compile_solid()` or
/// `compile_color()` to compile the composition's solid or color contents,
/// respectively, into node-space.
#[derive(Debug, Clone)]
pub struct Composition {
  shapes: Vec<(Shape, Position)>,
}

impl Default for Composition {
  fn default() -> Self {
    Self::new()
  }
}

impl From<Vec<(Shape, Position)>> for Composition {
  fn from(shapes: Vec<(Shape, Position)>) -> Self {
    Composition { shapes }
  }
}

impl Composition {
  /// Create a new, empty `Composition`.
  pub fn new() -> Self {
    Composition { shapes: Vec::new() }
  }

  /// Add a shape to the composition.
  pub fn add_shape(&mut self, shape: Shape, translation: Position) {
    self.shapes.push((shape, translation));
  }

  /// Compile the composition's solid field into node-space. All the shapes are
  /// combined using a `min` operation.
  pub fn compile_solid(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    // compile a translated Node for each Shape
    let shapes = &self
      .shapes
      .iter()
      .map(|(shape, pos)| {
        let a = shape.compile_solid(ctx, settings);
        nso_translate(a, *pos, ctx)
      })
      .collect::<Vec<Node>>();

    binary_shape_tree(shapes.to_vec(), ctx, BinaryShapeTreeCombinator::Min)
  }

  /// Compile the composition's color field into node-space. All the shapes are
  /// combined using a `max` operation.
  pub fn compile_color(
    &self,
    ctx: &mut Context,
    settings: &CompilationSettings,
  ) -> Node {
    // compile a translated Node for each Shape
    let shapes = &self
      .shapes
      .iter()
      .map(|(shape, pos)| {
        let a = shape.compile_color(ctx, settings);
        nso_translate(a, *pos, ctx)
      })
      .collect::<Vec<Node>>();

    binary_shape_tree(shapes.to_vec(), ctx, BinaryShapeTreeCombinator::Max)
  }
}

/// Controls which operation `binary_shape_tree()` uses to combine its contents.
#[allow(dead_code)]
enum BinaryShapeTreeCombinator {
  Min,
  Max,
  Add,
}

fn binary_shape_tree(
  nodes: Vec<Node>,
  ctx: &mut Context,
  combinator: BinaryShapeTreeCombinator,
) -> Node {
  let mut min_tree = nodes;
  while min_tree.len() > 1 {
    let mut new_tree = Vec::new();
    for i in (0..min_tree.len()).step_by(2) {
      let a = &min_tree[i];
      let b = if i + 1 < min_tree.len() {
        &min_tree[i + 1]
      } else {
        a
      };
      let node = match combinator {
        BinaryShapeTreeCombinator::Min => ctx.min(*a, *b).unwrap(),
        BinaryShapeTreeCombinator::Max => ctx.max(*a, *b).unwrap(),
        BinaryShapeTreeCombinator::Add => ctx.add(*a, *b).unwrap(),
      };
      new_tree.push(node);
    }
    min_tree = new_tree;
  }

  min_tree[0]
}
