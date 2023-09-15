use std::{
  collections::hash_map::DefaultHasher,
  fs::File,
  hash::{Hash, Hasher},
  io::{BufReader, BufWriter},
};

use bevy_xpbd_3d::prelude::Collider;
use planiscope::{comp::Composition, mesher::FullMesh};
use serde::{Deserialize, Serialize};

use crate::terrain::region::TerrainRegion;

#[derive(Serialize, Deserialize)]
pub struct CachePack {
  full_mesh:      FullMesh,
  collider_shape: Option<parry3d::shape::SharedShape>,
}

impl From<(FullMesh, Option<Collider>)> for CachePack {
  fn from(value: (FullMesh, Option<Collider>)) -> Self {
    Self {
      full_mesh:      value.0,
      collider_shape: value.1.map(|c| c.get_shape().clone()),
    }
  }
}

impl From<CachePack> for (FullMesh, Option<Collider>) {
  fn from(value: CachePack) -> Self {
    (value.full_mesh, value.collider_shape.map(|s| s.into()))
  }
}

pub fn mesh_meta_hash(comp: &Composition, region: &TerrainRegion) -> u64 {
  let mut hasher = DefaultHasher::new();
  comp.hash(&mut hasher);
  region.hash(&mut hasher);
  hasher.finish()
}

pub async fn read_pack_from_file(meta_hash: u64) -> Option<CachePack> {
  // we'll read the mesh from a file under mesh_cache/[meta_hash].cp
  // if we succeed, return the mesh
  let path = format!("mesh_cache/{:x?}.cp", meta_hash);
  let file = File::open(path).ok()?;
  let mut reader = BufReader::new(file);
  let pack: CachePack = rmp_serde::decode::from_read(&mut reader).ok()?;
  Some(pack)
}

pub async fn write_pack_to_file(
  meta_hash: u64,
  pack: &CachePack,
) -> Option<String> {
  // we'll write the mesh to a file under mesh_cache/[meta_hash].cp
  // if we succeed, return the path
  let _ = std::fs::create_dir_all("mesh_cache");
  let path = format!("mesh_cache/{:x?}.cp", meta_hash);
  let file = File::create(&path).ok()?;
  let mut writer = BufWriter::new(file);
  rmp_serde::encode::write(&mut writer, pack).ok()?;
  Some(path)
}
