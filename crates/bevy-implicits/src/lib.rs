#![feature(path_file_prefix)]

mod inputs;
mod loader;
mod reader;
mod utils;

use std::path::PathBuf;

use anyhow::Result;
use bevy::{
  asset::{io::AssetSource, AssetPath},
  prelude::*,
};
use planiscope::mesher::MesherInputs;

use self::{inputs::*, loader::*, reader::*};

pub mod prelude {
  pub use planiscope::{
    mesher::{MesherDetail, MesherInputs, MesherRegion},
    shape::Shape,
  };

  pub use crate::{
    asset_path, inputs::ImplicitInputs, ColliderAsset, ImplicitMesh,
    ImplicitsPlugin, SyncImplicits,
  };
}

fn path_from_inputs(inputs: MesherInputs) -> Result<PathBuf> {
  TryInto::<PathBuf>::try_into(ImplicitInputs(inputs))
}

/// Generates a `bevy::asset::AssetPath` for a mesh generated from the given
/// `MesherInputs`.
pub fn asset_path(inputs: MesherInputs) -> Result<AssetPath<'static>> {
  let path = path_from_inputs(inputs)?;
  Ok(AssetPath::from(path).with_source("implicit"))
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

pub struct ImplicitsAssetSourcePlugin;

impl Plugin for ImplicitsAssetSourcePlugin {
  fn build(&self, app: &mut App) {
    app.register_asset_source(
      "implicit",
      AssetSource::build().with_reader(|| Box::new(ImplicitInputsAssetReader)),
    );
  }
}

pub struct ImplicitsPlugin;

impl Plugin for ImplicitsPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_asset::<ImplicitMesh>()
      .init_asset::<ColliderAsset>()
      .register_asset_loader(ImplicitMeshAssetLoader)
      .add_systems(Update, sync_implicits);
  }
}

#[derive(Component)]
pub struct SyncImplicits;

// When an entity has a `ImplicitInputs` and `SyncImplicits` component, this
// system will add the `ImplicitMesh` asset built from the inputs to the entity,
// and update the `Handle<Mesh>` and `Collider` components.
fn sync_implicits(
  mut commands: Commands,
  query: Query<(Entity, &ImplicitInputs), With<SyncImplicits>>,
  asset_server: Res<AssetServer>,
  implicit_meshes: Res<Assets<ImplicitMesh>>,
  colliders: Res<Assets<ColliderAsset>>,
) {
  for (entity, inputs) in query.iter() {
    let asset_path = asset_path(inputs.0.clone()).unwrap();
    let handle: Handle<ImplicitMesh> = asset_server.load(asset_path);

    commands.entity(entity).insert(handle.clone());

    if asset_server.is_loaded_with_dependencies(handle.clone()) {
      let implicit_mesh = implicit_meshes.get(handle).unwrap();

      let collider_handle = implicit_mesh.collider.clone();
      let collider = colliders.get(collider_handle).unwrap();

      commands.entity(entity).insert(implicit_mesh.mesh.clone());
      if let Some(collider) = collider.0.clone() {
        commands.entity(entity).insert(collider);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::convert::TryInto;

  use super::*;

  #[test]
  fn test_implicit_inputs() {
    let inputs = ImplicitInputs(MesherInputs {
      shape:        planiscope::shape::builder::sphere(1.0),
      region:       planiscope::mesher::MesherRegion {
        position: Vec3::ZERO.into(),
        scale:    Vec3::ONE.into(),
        detail:   planiscope::mesher::MesherDetail::Resolution(8.0),
        prune:    true,
      },
      gen_collider: true,
    });
    let path: PathBuf = inputs.clone().try_into().unwrap();
    let inputs2: ImplicitInputs = path.try_into().unwrap();
    assert_eq!(format!("{:?}", inputs), format!("{:?}", inputs2));
  }
}
