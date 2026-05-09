use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use http::Method;
use path_tree::PathTree;
use percent_encoding::percent_decode_str;
use tokio::io::AsyncWriteExt;

use crate::http1::Request;
use crate::http1::Response;

type HandlerFunc = Arc<
  dyn 'static
    + Send
    + Sync
    + Fn(Request, Response) -> Pin<Box<dyn 'static + Send + Future<Output = anyhow::Result<()>>>>,
>;

type RouterHandlerFunc = Box<
  dyn 'static
    + Send
    + Sync
    + Fn(Request, Response) -> Pin<Box<dyn 'static + Send + Future<Output = anyhow::Result<()>>>>,
>;

pub struct Router {
  any_routes: PathTree<HandlerFunc>,
  get_routes: PathTree<HandlerFunc>,
  post_routes: PathTree<HandlerFunc>,
  put_routes: PathTree<HandlerFunc>,
  patch_routes: PathTree<HandlerFunc>,
  delete_routes: PathTree<HandlerFunc>,
  head_routes: PathTree<HandlerFunc>,
  connect_routes: PathTree<HandlerFunc>,
  fallback: Option<HandlerFunc>,
}

impl Router {
  pub fn new() -> Self {
    Self {
      any_routes: PathTree::new(),
      get_routes: PathTree::new(),
      post_routes: PathTree::new(),
      put_routes: PathTree::new(),
      patch_routes: PathTree::new(),
      delete_routes: PathTree::new(),
      head_routes: PathTree::new(),
      connect_routes: PathTree::new(),
      fallback: None,
    }
  }

  pub fn not_found<F, Fut>(
    &mut self,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    self
      .fallback
      .replace(Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn get<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    let _ = self
      .get_routes
      .insert(route, Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn post<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    let _ = self
      .post_routes
      .insert(route, Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn put<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    let _ = self
      .put_routes
      .insert(route, Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn patch<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    let _ = self
      .patch_routes
      .insert(route, Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn delete<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    let _ = self
      .delete_routes
      .insert(route, Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn head<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    let _ = self
      .head_routes
      .insert(route, Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn connect<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = anyhow::Result<()>>,
  {
    let _ = self
      .connect_routes
      .insert(route, Arc::new(move |req, res| Box::pin(handler(req, res))));
  }

  pub fn handler(&self) -> RouterHandlerFunc {
    let any_routes = Arc::new(self.any_routes.clone());
    let get_routes = Arc::new(self.get_routes.clone());
    let post_routes = Arc::new(self.post_routes.clone());
    let put_routes = Arc::new(self.put_routes.clone());
    let patch_routes = Arc::new(self.patch_routes.clone());
    let delete_routes = Arc::new(self.delete_routes.clone());
    let head_routes = Arc::new(self.head_routes.clone());
    let connect_routes = Arc::new(self.connect_routes.clone());
    let fallback = self.fallback.clone();

    Box::new(move |mut req, mut res| {
      let any_routes = any_routes.clone();
      let get_routes = get_routes.clone();
      let post_routes = post_routes.clone();
      let put_routes = put_routes.clone();
      let patch_routes = patch_routes.clone();
      let delete_routes = delete_routes.clone();
      let head_routes = head_routes.clone();
      let connect_routes = connect_routes.clone();
      let fallback = fallback.clone();

      Box::pin(async move {
        let path = req.uri.path().to_string();
        let routes = match req.method() {
          &Method::GET => get_routes,
          &Method::POST => post_routes,
          &Method::PUT => put_routes,
          &Method::PATCH => patch_routes,
          &Method::DELETE => delete_routes,
          &Method::HEAD => head_routes,
          &Method::CONNECT => connect_routes,
          _ => Arc::clone(&any_routes),
        };
        if let Some((handler, params)) = routes.find(&path) {
          let params_map: HashMap<String, String> = params
            .params()
            .iter()
            .map(|(k, v)| {
              (
                k.to_string(),
                percent_decode_str(v).decode_utf8_lossy().to_string(),
              )
            })
            .collect();
          req.params = params_map;
          handler(req, res).await?;
          return Ok(());
        }

        if let Some((handler, params)) = any_routes.find(&path) {
          let params_map: HashMap<String, String> = params
            .params()
            .iter()
            .map(|(k, v)| {
              (
                k.to_string(),
                percent_decode_str(v).decode_utf8_lossy().to_string(),
              )
            })
            .collect();

          req.params = params_map;
          handler(req, res).await?;
          return Ok(());
        }

        if let Some(handler) = fallback {
          handler(req, res).await?;
          return Ok(());
        }

        res.write_all(b"").await?;
        res.write_head(crate::StatusCode::NOT_FOUND).await?;
        Ok(())
      })
    })
  }
}
