#![feature(path_file_prefix)]

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use bevy::{
  asset::{
    io::{AssetReader, AssetReaderError, AssetSource, PathStream, Reader},
    AssetLoader, AssetPath, AsyncReadExt,
  },
  prelude::*,
  utils::BoxedFuture,
};
use bevy_xpbd_3d::components::Collider;
use planiscope::{
  cache::{CacheProvider, DiskCacheProvider},
  mesher::{FastSurfaceNetsMesher, FullMesh, MesherInputs},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod prelude {
  pub use planiscope::mesher::{MesherDetail, MesherInputs, MesherRegion};

  pub use crate::{
    ColliderAsset, ImplicitInputs, ImplicitMesh, ImplicitsPlugin,
  };
}

fn path_from_inputs(inputs: MesherInputs) -> Result<PathBuf> {
  TryInto::<PathBuf>::try_into(ImplicitInputs(inputs))
}

/// Generates a `bevy::asset::AssetPath` for a mesh generated from the given
/// `MesherInputs`.
pub fn mesh_path(inputs: MesherInputs) -> Result<AssetPath<'static>> {
  Ok(
    AssetPath::from_path(path_from_inputs(inputs)?.as_path())
      .resolve("#mesh")?
      .to_owned(),
  )
}

/// A wrapper around `planiscope::mesher::MesherInputs` that implements
/// `bevy::asset::Asset` and can be de/serialized to a file path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplicitInputs(MesherInputs);

impl TryFrom<PathBuf> for ImplicitInputs {
  type Error = anyhow::Error;

  fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
    // println!("starting with path: {:?}", path);

    let file_prefix = path
      .file_prefix()
      .ok_or_else(|| {
        anyhow!("failed to get file prefix from path: {:?}", path)
      })?
      .to_string_lossy()
      .to_string();
    // println!("base64_encoded: {:?}", file_prefix);

    // decode from base64
    let base64_decoded = general_purpose::URL_SAFE
      .decode(file_prefix)
      .context("failed to decode base64 from file prefix")?;

    // decode from messagepack
    let decoded: Self = rmp_serde::from_slice(&base64_decoded).context(
      "failed to decode messagepack from base64-decoded file prefix",
    )?;

    Ok(decoded)
  }
}

impl TryFrom<ImplicitInputs> for PathBuf {
  type Error = anyhow::Error;

  fn try_from(inputs: ImplicitInputs) -> Result<Self, Self::Error> {
    let messagepack_encoded = rmp_serde::to_vec(&inputs)
      .context(format!("serde failed to serialize: {:?}", &inputs))?;
    // println!("messagepack_encoded: {:?}", messagepack_encoded);
    let base64_encoded = general_purpose::URL_SAFE.encode(&messagepack_encoded);
    // println!("base64_encoded: {:?}", base64_encoded);

    Ok(PathBuf::from(format!(
      "implicit://{}.implicit",
      base64_encoded
    )))
  }
}

struct DummyAsyncReader<T: Serialize + for<'a> Deserialize<'a>>(T);

impl<T: Serialize + for<'a> Deserialize<'a>> futures_io::AsyncRead
  for DummyAsyncReader<T>
{
  fn poll_read(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
    buf: &mut [u8],
  ) -> std::task::Poll<std::io::Result<usize>> {
    let bytes = bincode::serialize(&self.0).unwrap();
    let buf_len = buf.len();
    buf[buf_len..buf_len + bytes.len()].copy_from_slice(&bytes);
    std::task::Poll::Ready(Ok(bytes.len()))
  }
}

/// An `AssetReader` that reads `ImplicitInputs` from a file path.
struct ImplicitInputsAssetReader;

impl AssetReader for ImplicitInputsAssetReader {
  fn read<'a>(
    &'a self,
    path: &'a std::path::Path,
  ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
    match ImplicitInputs::try_from(path.to_path_buf()) {
      Ok(inputs) => {
        let reader: Box<dyn futures_io::AsyncRead + Send + Sync + Unpin> =
          Box::new(DummyAsyncReader(inputs));
        Box::pin(async move { Ok(reader) })
      }
      Err(err) => Box::pin(async move {
        error!("failed to decode planiscope payload: {:?}", err);
        Err(AssetReaderError::NotFound(path.to_path_buf()))
      }),
    }
  }

  fn read_meta<'a>(
    &'a self,
    path: &'a std::path::Path,
  ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
    Box::pin(async move { Err(AssetReaderError::NotFound(path.to_path_buf())) })
  }

  fn read_directory<'a>(
    &'a self,
    _path: &'a std::path::Path,
  ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
    unimplemented!(
      "`read_directory` makes no sense for generated assets. You might be \
       generating your asset paths incorrectly."
    )
  }

  fn is_directory<'a>(
    &'a self,
    _path: &'a std::path::Path,
  ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
    unimplemented!(
      "`is_directory` makes no sense for generated assets. You might be \
       generating your asset paths incorrectly."
    )
  }
}

/// A wrapper around `bevy_xpbd_3d::components::Collider` that implements
/// `bevy::asset::Asset`.
#[derive(Debug, Clone, Asset, TypePath)]
pub struct ColliderAsset(pub Option<bevy_xpbd_3d::components::Collider>);

/// The asset generated by `ImplicitMeshAssetLoader`. It contains the meshing
/// inputs, the generated mesh, and the collider.
#[derive(Debug, Clone, Asset, TypePath)]
pub struct ImplicitMesh {
  pub inputs:   MesherInputs,
  pub mesh:     Handle<Mesh>,
  pub collider: Handle<ColliderAsset>,
}

/// An `AssetLoader` that loads `ImplicitMesh` from a file path and generates
/// the mesh if necessary.
struct ImplicitMeshAssetLoader;

#[derive(Error, Debug)]
enum ImplicitMeshError {
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
      let _ = reader.read_to_end(&mut bytes);
      let inputs: ImplicitInputs = bincode::deserialize(&bytes).unwrap();

      let (mesh, collider) =
        DiskCacheProvider::<FastSurfaceNetsMesher>::default()
          .get_mesh_and_collider(&planiscope::mesher::MesherInputs {
            shape:  inputs.0.shape.clone(),
            region: inputs.0.region.clone(),
          });
      let mesh = mesh.map_err(|e| ImplicitMeshError::MeshError(e))?;
      let mesh = bevy_mesh_from_pls_mesh(mesh);
      let collider = collider.map(|s| Collider::from(s));

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

/// The central plugin for this crate.
/// It registers the asset types, loaders, and sources.
pub struct ImplicitsPlugin;

impl Plugin for ImplicitsPlugin {
  fn build(&self, app: &mut App) {
    let asset_source =
      AssetSource::build().with_reader(|| Box::new(ImplicitInputsAssetReader));

    app
      .init_asset::<ImplicitMesh>()
      .init_asset::<ColliderAsset>()
      .register_asset_loader(ImplicitMeshAssetLoader)
      .register_asset_source("implicit", asset_source);
  }
}

/// Converts a `planiscope::mesher::FullMesh` to a `bevy::render::mesh::Mesh`.
pub fn bevy_mesh_from_pls_mesh(mesh: FullMesh) -> Mesh {
  let mut bevy_mesh =
    Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);

  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_POSITION,
    mesh
      .vertices
      .into_iter()
      .map(|v| v.to_array())
      .collect::<Vec<_>>(),
  );
  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_NORMAL,
    mesh
      .normals
      .into_iter()
      .map(|v| v.to_array())
      .collect::<Vec<_>>(),
  );

  bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
    mesh
      .triangles
      .into_iter()
      .flat_map(|v| v.to_array())
      .collect(),
  )));
  bevy_mesh
}

#[cfg(test)]
mod tests {
  use std::convert::TryInto;

  use super::*;

  #[test]
  fn test_implicit_inputs() {
    let inputs = ImplicitInputs(MesherInputs {
      shape:  planiscope::shape::builder::sphere(1.0),
      region: planiscope::mesher::MesherRegion {
        position: Vec3::ZERO.into(),
        scale:    Vec3::ONE.into(),
        detail:   planiscope::mesher::MesherDetail::Resolution(8.0),
        prune:    true,
      },
    });
    let path: PathBuf = inputs.clone().try_into().unwrap();
    let inputs2: ImplicitInputs = path.try_into().unwrap();
    assert_eq!(format!("{:?}", inputs), format!("{:?}", inputs2));
  }
}
