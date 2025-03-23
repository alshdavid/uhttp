// This module remaps types from dependencies to avoid naming collisions

pub type ResponseBuilder = hyper::http::response::Builder;
pub use hyper::Response as HyperResponse;
pub use hyper::body::Bytes as HyperBytes;
