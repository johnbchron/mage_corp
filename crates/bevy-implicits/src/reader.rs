use std::io::{Cursor, Read};

use anyhow::Result;
use bevy::{
  asset::io::{AssetReader, AssetReaderError, PathStream, Reader},
  prelude::*,
  utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};

use crate::inputs::*;

struct CursorAsyncReader(Cursor<Vec<u8>>);

impl CursorAsyncReader {
  fn new<T: Serialize + for<'a> Deserialize<'a>>(inner: T) -> Self {
    Self(Cursor::new(bincode::serialize(&inner).unwrap()))
  }
}

impl futures_io::AsyncRead for CursorAsyncReader {
  fn poll_read(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
    buf: &mut [u8],
  ) -> std::task::Poll<std::io::Result<usize>> {
    let bytes_read = self.get_mut().0.read(buf)?;
    std::task::Poll::Ready(Ok(bytes_read))
  }
}

/// An `AssetReader` that reads `ImplicitInputs` from a file path.
pub(crate) struct ImplicitInputsAssetReader;

impl AssetReader for ImplicitInputsAssetReader {
  fn read<'a>(
    &'a self,
    path: &'a std::path::Path,
  ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
    match ImplicitInputs::try_from(path.to_path_buf()) {
      Ok(inputs) => {
        let reader: Box<dyn futures_io::AsyncRead + Send + Sync + Unpin> =
          Box::new(CursorAsyncReader::new(inputs));
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
