// This module remaps types from dependencies to avoid naming collisions

pub(crate) type ResponseBuilder = hyper::http::response::Builder;
// pub(crate) use hyper::body::Body as HyperBody;
pub(crate) use hyper::Request as HyperRequest;
pub(crate) use hyper::Response as HyperResponse;
pub(crate) use hyper::body::Bytes as HyperBytes;
pub(crate) use hyper::body::Incoming as HyperIncoming;
