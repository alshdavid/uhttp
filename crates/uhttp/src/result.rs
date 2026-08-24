#[cfg(feature = "anyhow")]
pub type Result<T> = anyhow::Result<T>;

#[cfg(not(feature = "anyhow"))]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  IO(std::io::Error),
  Generic(String),
  HTTP(http::Error),
}

impl Error {
  #[cfg(not(feature = "anyhow"))]
  pub fn generic(message: impl AsRef<str>) -> Self {
    Self::Generic(message.as_ref().to_string())
  }

  #[cfg(feature = "anyhow")]
  pub fn generic(message: impl AsRef<str>) -> anyhow::Error {
    anyhow::anyhow!("{}", message.as_ref())
  }

  #[cfg(not(feature = "anyhow"))]
  pub fn generic_err(message: impl AsRef<str>) -> Result<()> {
    Err(Self::Generic(message.as_ref().to_string()))
  }

  #[cfg(feature = "anyhow")]
  pub fn generic_err(message: impl AsRef<str>) -> Result<()> {
    Err(anyhow::anyhow!("{}", message.as_ref()))
  }
}

impl std::fmt::Display for Error {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Error::IO(value)
  }
}

impl From<http::Error> for Error {
  fn from(value: http::Error) -> Self {
    Error::HTTP(value)
  }
}
