mod etag_hash_resolver;
mod etag_last_modified_resolver;
mod etag_resolver;

pub use self::etag_hash_resolver::EtagHashResolver;
pub use self::etag_last_modified_resolver::EtagLastModifiedResolver;
pub use self::etag_resolver::EtagResolver;
