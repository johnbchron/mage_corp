#![feature(iter_map_windows)]

mod hash;
mod hedge;
mod mesh;
mod simplify;

pub use mesh::{FullMesh, FullVertex};
pub use simplify::simplify_mesh;
