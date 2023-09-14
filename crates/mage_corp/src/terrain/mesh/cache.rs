use std::{
  collections::hash_map::DefaultHasher,
  fs::File,
  hash::{Hash, Hasher},
  io::{BufReader, BufWriter},
};

use planiscope::{comp::Composition, mesher::FullMesh};

use crate::terrain::region::TerrainRegion;

pub fn mesh_meta_hash(comp: &Composition, region: &TerrainRegion) -> u64 {
  let mut hasher = DefaultHasher::new();
  comp.hash(&mut hasher);
  region.hash(&mut hasher);
  hasher.finish()
}

pub fn read_mesh_from_file(meta_hash: u64) -> Option<FullMesh> {
  // we'll read the mesh from a file under mesh_cache/[meta_hash].fm
  // if we succeed, return the mesh
  let path = format!("mesh_cache/{:x?}.fm", meta_hash);
  let file = File::open(&path).ok()?;
  let mut reader = BufReader::new(file);
  let mesh: FullMesh = rmp_serde::decode::from_read(&mut reader).ok()?;
  Some(mesh)
}

pub fn write_mesh_to_file(meta_hash: u64, mesh: &FullMesh) -> Option<String> {
  // we'll write the mesh to a file under mesh_cache/[meta_hash].fm
  // if we succeed, return the path
  let _ = std::fs::create_dir_all("mesh_cache");
  let path = format!("mesh_cache/{:x?}.fm", meta_hash);
  let file = File::create(&path).ok()?;
  let mut writer = BufWriter::new(file);
  rmp_serde::encode::write(&mut writer, mesh).ok()?;
  Some(path)
}
