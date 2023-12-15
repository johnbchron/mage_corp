use anyhow::Result;
use bevy::{
  asset::{io::Reader, AssetLoader, AsyncReadExt},
  prelude::*,
  utils::BoxedFuture,
};
use bevy_xpbd_3d::components::Collider;
use planiscope::{
  cache::{CacheProvider, DiskCacheProvider},
  mesher::FastSurfaceNetsMesher,
};
use thiserror::Error;

use crate::{
  inputs::*, utils::bevy_mesh_from_pls_mesh, ColliderAsset, ImplicitMesh,
};

/// An `AssetLoader` that loads `ImplicitMesh` from a file path and generates
/// the mesh if necessary.
pub(crate) struct ImplicitMeshAssetLoader;

#[derive(Error, Debug)]
pub(crate) enum ImplicitMeshError {
  #[error("Failed to generate mesh: {0}")]
  MeshError(fidget::Error),
}

impl AssetLoader for ImplicitMeshAssetLoader {
  type Asset = ImplicitMesh;
  type Settings = ();
  type Error = ImplicitMeshError;

  fn load<'a>(
    &'a self,
    reader: &'a mut Reader,
    _settings: &'a Self::Settings,
    load_context: &'a mut bevy::asset::LoadContext,
  ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
    Box::pin(async move {
      let mut bytes = Vec::new();
      reader.read_to_end(&mut bytes).await.unwrap();
      let inputs: ImplicitInputs = bincode::deserialize(&bytes).unwrap();

      let (mesh, collider) =
        DiskCacheProvider::<FastSurfaceNetsMesher>::default()
          .get_mesh_and_collider(&inputs.0);
      let mesh = mesh.map_err(ImplicitMeshError::MeshError)?;
      let mesh = bevy_mesh_from_pls_mesh(mesh);
      let collider = collider.map(Collider::from);

      let mesh_handle =
        load_context.add_labeled_asset("mesh".to_string(), mesh);
      let collider_handle = load_context
        .add_labeled_asset("collider".to_string(), ColliderAsset(collider));

      Ok(ImplicitMesh {
        inputs:   inputs.0,
        mesh:     mesh_handle,
        collider: collider_handle,
      })
    })
  }

  fn extensions(&self) -> &[&str] {
    &["implicit"]
  }
}
