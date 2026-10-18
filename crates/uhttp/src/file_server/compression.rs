use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::BufReader;

pub async fn gzip_stream<R, W>(
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

pub async fn brotli_stream<R, W>(
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

pub async fn zstd_stream<R, W>(
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

pub async fn uncompressed_stream<R, W>(
  mut input: R,
  output: &mut W,
) -> Result<u64, std::io::Error>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  tokio::io::copy(&mut input, output).await
}
