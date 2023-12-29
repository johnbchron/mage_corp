#![feature(iter_map_windows)]
#![warn(missing_docs)]

//! # Mosh
//! ## A **M**esh **O**ptimization **S**uite using **H**alf-edge meshes.

mod bufmesh;
mod hash;
pub mod mizu;
mod simplify;

pub use bufmesh::{BufMesh, FullVertex};
pub use simplify::simplify_mesh;
