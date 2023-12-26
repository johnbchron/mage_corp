use std::{
  collections::hash_map::DefaultHasher,
  fs::File,
  hash::{Hash, Hasher},
  io::{BufReader, BufWriter},
  path::PathBuf,
};

use parry3d::shape::SharedShape;
use serde::{Deserialize, Serialize};

use super::{CacheProvider, DiskCacheProvider};
use crate::{
  collider::{generate_collider, ColliderSettings},
  mesher::{Mesher, MesherInputs},
};

fn hash_single<H: Hash>(value: &H) -> u64 {
  let mut hasher = DefaultHasher::new();
  value.hash(&mut hasher);
  hasher.finish()
}

fn serialize_to_file<V: Serialize>(path: &str, value: &V) -> Option<String> {
  std::fs::create_dir_all(PathBuf::from(path).parent()?).ok()?;
  let file = File::create(path).ok()?;
  let mut writer = BufWriter::new(file);
  rmp_serde::encode::write(&mut writer, value).ok()?;
  Some(path.to_string())
}

fn deserialize_from_file<V: for<'de> Deserialize<'de>>(
  path: &str,
) -> Option<V> {
  let file = File::open(path).ok()?;
  let mut reader = BufReader::new(file);
  rmp_serde::decode::from_read(&mut reader).ok()
}

impl<M: Mesher> CacheProvider for DiskCacheProvider<M> {
  fn get_mesh(
    &self,
    inputs: &MesherInputs,
  ) -> Result<crate::mesher::BufMesh, fidget::Error> {
    // get the hash and resulting path
    let inputs_hash = hash_single(inputs);
    let path = format!("{}{}", self.mesh_path, inputs_hash);

    // try to open the file
    if let Some(file) = deserialize_from_file(&path) {
      return Ok(file);
    }

    // if we're here, either the cache didn't exist or was corrupted.
    // in either case, time to generate the mesh.
    let meshed = self.mesher.build_mesh(inputs);

    // cache the file if the meshing worked
    if let Ok(mesh) = &meshed {
      serialize_to_file(&path, mesh);
    }

    meshed
  }

  fn get_collider(
    &self,
    inputs: &MesherInputs,
  ) -> Option<parry3d::shape::SharedShape> {
    if !inputs.gen_collider {
      return None;
    }

    // get the hash and resulting path
    let inputs_hash = hash_single(inputs);
    let path = format!("{}{}", self.mesh_path, inputs_hash);

    // try to open the file
    if let Some(collider) = deserialize_from_file(&path) {
      return Some(collider);
    }

    // we can't get it from cache, so we need to generate it. to generate it we
    // need the actual mesh it's from, so let's get that, hopefully from cache.
    self
      .get_mesh(inputs)
      .ok()
      .and_then(|mesh| generate_collider(mesh, &ColliderSettings::default()))
      .inspect(|c| {
        serialize_to_file(&path, c);
      })
  }

  fn get_mesh_and_collider(
    &self,
    inputs: &MesherInputs,
  ) -> (
    Result<crate::mesher::BufMesh, fidget::Error>,
    Option<parry3d::shape::SharedShape>,
  ) {
    let mesh = self.get_mesh(inputs);

    let inputs_hash = hash_single(inputs);
    let path = format!("{}{}", self.collider_path, inputs_hash);

    if !inputs.gen_collider {
      return (mesh, None);
    }

    let collider = match deserialize_from_file::<SharedShape>(&path) {
      Some(s) => Some(s),
      None => mesh
        .as_ref()
        .ok()
        .and_then(|m| {
          generate_collider(m.clone(), &ColliderSettings::default())
        })
        .inspect(|s| {
          serialize_to_file(&path, s);
        }),
    };

    (mesh, collider)
  }
}
