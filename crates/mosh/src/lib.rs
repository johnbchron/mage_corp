#![feature(iter_map_windows)]
#![warn(missing_docs)]

//! # Mosh
//! ## A **M**esh **O**ptimization **S**uite using **H**alf-edge meshes.

mod hash;
pub mod hedge;
mod mesh;
mod simplify;

pub use mesh::{FullMesh, FullVertex};
pub use simplify::simplify_mesh;
