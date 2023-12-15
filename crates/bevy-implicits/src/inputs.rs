use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use bevy::{prelude::Component, reflect::Reflect};
use planiscope::mesher::MesherInputs;
use serde::{Deserialize, Serialize};

/// A wrapper around `planiscope::mesher::MesherInputs` that implements
/// `bevy::asset::Asset` and can be de/serialized to a file path.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ImplicitInputs(pub MesherInputs);

impl TryFrom<PathBuf> for ImplicitInputs {
  type Error = anyhow::Error;

  fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
    let file_prefix = path
      .file_prefix()
      .ok_or_else(|| {
        anyhow!("failed to get file prefix from path: {:?}", path)
      })?
      .to_string_lossy()
      .to_string();

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
    let base64_encoded = general_purpose::URL_SAFE.encode(messagepack_encoded);
    // println!("base64_encoded: {:?}", base64_encoded);

    Ok(PathBuf::from(format!("{}.implicit", base64_encoded)))
  }
}
