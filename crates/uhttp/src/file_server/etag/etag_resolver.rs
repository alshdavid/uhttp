use std::io;

use super::super::file_server::ETagStrategy;
use super::super::file_system::AsyncFile;
use super::EtagHashResolver;
use super::EtagLastModifiedResolver;

pub enum EtagResolver {
  Hash(EtagHashResolver),
  LastModified(EtagLastModifiedResolver),
}

impl EtagResolver {
  /// `Disabled` has no resolver, so "don't compute an etag" is represented as
  /// `None` rather than a third strategy type.
  pub fn new(strategy: &ETagStrategy) -> Option<Self> {
    match strategy {
      ETagStrategy::Hash => Some(EtagResolver::Hash(EtagHashResolver)),
      ETagStrategy::LastModified => Some(EtagResolver::LastModified(EtagLastModifiedResolver)),
      ETagStrategy::Disabled => None,
    }
  }

  pub async fn resolve(
    &self,
    file: &mut dyn AsyncFile,
    encoding: &str,
  ) -> io::Result<Option<String>> {
    match self {
      EtagResolver::Hash(resolver) => resolver.resolve(file, encoding).await,
      EtagResolver::LastModified(resolver) => resolver.resolve(file, encoding).await,
    }
  }
}
