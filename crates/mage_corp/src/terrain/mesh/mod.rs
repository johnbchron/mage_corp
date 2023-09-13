use std::io::BufReader;
use std::fs::File;
use std::io::BufWriter;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use planiscope::{
  comp::Composition,
  mesher::{FastSurfaceNetsMesher, FullMesh, Mesher, MesherInputs},
};

use super::region::TerrainRegion;

#[derive(Debug, TypeUuid, Reflect)]
#[uuid = "3dc0b7c0-e829-4634-b490-2f5f53873a1d"]
pub struct TerrainMesh {
  /// Contains the bevy mesh for this terrain mesh.
  pub mesh:      Handle<Mesh>,
  /// Describes the region that the composition was evaluated over.
  pub region:    TerrainRegion,
  /// The hash of the composition.
  pub comp_hash: u64,
}

pub fn generate(comp: &Composition, region: &TerrainRegion) -> Mesh {
  let mesher_inputs = MesherInputs {
    position: region.position.into(),
    scale:    region.scale.into(),
    subdivs:  region.subdivs,
    prune:    true,
  };

  let meta_hash = mesh_meta_hash(comp, region);
  
  match read_mesh_from_file(meta_hash) {
    Some(mesh) => {
      info!("read mesh from file");
      bevy_mesh_from_pls_mesh(mesh)
    },
    None => {
      let full_mesh = FastSurfaceNetsMesher::build_mesh(comp, mesher_inputs).unwrap();
      if let Some(path) = write_mesh_to_file(meta_hash, &full_mesh) {
        info!("wrote mesh to {}", path);
      }
      let mesh: Mesh = bevy_mesh_from_pls_mesh(full_mesh);
      if mesh.count_vertices() != 0 {
        info!(
          "generated terrain mesh for position {:?} and scale {:?} with {:?} \
           vertices",
          region.position,
          region.scale,
          mesh.count_vertices()
        );
      }
      mesh
    }
  }
}

fn mesh_meta_hash(comp: &Composition, region: &TerrainRegion) -> u64 {
  let mut hasher = DefaultHasher::new();
  comp.hash(&mut hasher);
  region.hash(&mut hasher);
  hasher.finish()
}

fn read_mesh_from_file(meta_hash: u64) -> Option<FullMesh> {
  // we'll read the mesh from a file under mesh_cache/[meta_hash].fm
  // if we succeed, return the mesh
  let path = format!("mesh_cache/{}.fm", meta_hash);
  let file = File::open(&path).ok()?;
  let mut reader = BufReader::new(file);
  let mesh: FullMesh = rmp_serde::decode::from_read(&mut reader).ok()?;
  Some(mesh)
}

fn write_mesh_to_file(meta_hash: u64, mesh: &FullMesh) -> Option<String> {
  // we'll write the mesh to a file under mesh_cache/[meta_hash].fm
  // if we succeed, return the path
  let _ = std::fs::create_dir_all("mesh_cache");
  let path = format!("mesh_cache/{}.fm", meta_hash);
  let file = File::create(&path).ok()?;
  let mut writer = BufWriter::new(file);
  rmp_serde::encode::write(&mut writer, mesh).ok()?;
  Some(path)
}

fn bevy_mesh_from_pls_mesh(mesh: FullMesh) -> Mesh {
  let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);
  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_POSITION,
    mesh
      .vertices
      .clone()
      .into_iter()
      .map(Into::<[f32; 3]>::into)
      .collect::<Vec<_>>(),
  );
  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_NORMAL,
    mesh
      .normals
      .into_iter()
      .map(Into::<[f32; 3]>::into)
      .collect::<Vec<_>>(),
  );

  bevy_mesh.set_indices(Some(Indices::U32(
    mesh
      .triangles
      .into_iter()
      .flat_map(|v| [v.x, v.y, v.z])
      .collect(),
  )));
  bevy_mesh
}
