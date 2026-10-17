use std::path::Path;

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::BufReader;

pub(crate) async fn gzip_stream<R, W>(
  input: R,
  output: &mut W,
) -> Result<u64, std::io::Error>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  use async_compression::tokio::bufread::GzipEncoder;
  let mut encoder = GzipEncoder::new(BufReader::new(input));
  tokio::io::copy(&mut encoder, output).await
}

pub(crate) async fn brotli_stream<R, W>(
  input: R,
  output: &mut W,
) -> Result<u64, std::io::Error>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  use async_compression::tokio::bufread::BrotliEncoder;
  let mut encoder = BrotliEncoder::new(BufReader::new(input));
  tokio::io::copy(&mut encoder, output).await
}

pub(crate) async fn zstd_stream<R, W>(
  input: R,
  output: &mut W,
) -> Result<u64, std::io::Error>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  use async_compression::tokio::bufread::ZstdEncoder;
  let mut encoder = ZstdEncoder::new(BufReader::new(input));
  tokio::io::copy(&mut encoder, output).await
}

pub(crate) fn try_extension(input: &Path) -> crate::Result<&str> {
  let Some(ext) = input.extension() else {
    return Ok(Default::default());
  };

  let Some(ext) = ext.to_str() else {
    return Ok(Default::default());
  };

  Ok(ext)
}
