use std::pin::Pin;

use super::Request;
use super::Response;

pub type HandlerFuncResult = anyhow::Result<()>;

pub trait HandlerFunc: 
  'static + 
  Send + 
  Sync + 
  Copy + 
  for <'a> Fn(Request, &'a mut dyn Response) -> Pin<Box<dyn Future<Output = HandlerFuncResult> + Send + 'a>> {}

impl<F> HandlerFunc for F
where
  F: 
    'static + 
    Send + 
    Sync + 
    Copy + 
    for <'a> Fn(Request, &'a mut dyn Response) -> Pin<Box<dyn Future<Output = HandlerFuncResult> + Send + 'a>>,
{}
