use super::*;

pub fn expr(expr: String) -> Shape {
  Shape::Expression { expr }
}

pub fn x() -> Shape {
  Shape::XNode
}
pub fn y() -> Shape {
  Shape::YNode
}
pub fn z() -> Shape {
  Shape::ZNode
}

pub fn constant(a: f64) -> Shape {
  Shape::Constant(a)
}

pub fn add(lhs: impl Into<Shape>, rhs: impl Into<Shape>) -> Shape {
  Shape::Add(Box::new(lhs.into()), Box::new(rhs.into()))
}
pub fn sub(lhs: impl Into<Shape>, rhs: impl Into<Shape>) -> Shape {
  Shape::Sub(Box::new(lhs.into()), Box::new(rhs.into()))
}
pub fn mul(lhs: impl Into<Shape>, rhs: impl Into<Shape>) -> Shape {
  Shape::Mul(Box::new(lhs.into()), Box::new(rhs.into()))
}
pub fn div(lhs: impl Into<Shape>, rhs: impl Into<Shape>) -> Shape {
  Shape::Div(Box::new(lhs.into()), Box::new(rhs.into()))
}
pub fn min(lhs: impl Into<Shape>, rhs: impl Into<Shape>) -> Shape {
  Shape::Min(Box::new(lhs.into()), Box::new(rhs.into()))
}
pub fn max(lhs: impl Into<Shape>, rhs: impl Into<Shape>) -> Shape {
  Shape::Max(Box::new(lhs.into()), Box::new(rhs.into()))
}

pub fn neg(a: impl Into<Shape>) -> Shape {
  Shape::Neg(Box::new(a.into()))
}
pub fn exp(a: impl Into<Shape>) -> Shape {
  Shape::Exp(Box::new(a.into()))
}
pub fn sin(a: impl Into<Shape>) -> Shape {
  Shape::Sin(Box::new(a.into()))
}
pub fn cos(a: impl Into<Shape>) -> Shape {
  Shape::Cos(Box::new(a.into()))
}
pub fn recip(a: impl Into<Shape>) -> Shape {
  Shape::Recip(Box::new(a.into()))
}
pub fn abs(a: impl Into<Shape>) -> Shape {
  Shape::Abs(Box::new(a.into()))
}
pub fn sqrt(a: impl Into<Shape>) -> Shape {
  Shape::Sqrt(Box::new(a.into()))
}
pub fn square(a: impl Into<Shape>) -> Shape {
  Shape::Square(Box::new(a.into()))
}

pub fn remap(
  root: impl Into<Shape>,
  x: impl Into<Shape>,
  y: impl Into<Shape>,
  z: impl Into<Shape>,
) -> Shape {
  Shape::Remap {
    root:  Box::new(root.into()),
    new_x: Box::new(x.into()),
    new_y: Box::new(y.into()),
    new_z: Box::new(z.into()),
  }
}

// extra
pub fn smooth_min_cubic(
  lhs: impl Into<Shape>,
  rhs: impl Into<Shape>,
  k: impl Into<Shape>,
) -> Shape {
  Shape::Extra(extra::Extra::SmoothMinCubic(
    Box::new(lhs.into()),
    Box::new(rhs.into()),
    Box::new(k.into()),
  ))
}
