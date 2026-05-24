use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use http::Method;
use path_tree::PathTree;
use percent_encoding::percent_decode_str;
use tokio::io::AsyncWriteExt;

use super::RouteBuilder;
use super::RouteBuilderNc;
use crate::Request;
use crate::Response;

pub type RouterHandleFunc<Context> = Arc<
  dyn 'static
    + Send
    + Sync
    + Fn(
      Request,
      Response,
      Context,
    ) -> Pin<Box<dyn 'static + Send + Future<Output = crate::Result<()>>>>,
>;

pub type RouterMiddlewareFunc<Context> = Arc<
  dyn 'static
    + Send
    + Sync
    + Fn(
      Request,
      Response,
      Context,
    ) -> Pin<
      Box<
        dyn 'static + Send + Future<Output = crate::Result<Option<(Request, Response, Context)>>>,
      >,
    >,
>;

pub struct Router<T>
where
  T: Clone + Send + Sync + 'static,
{
  middleware: Vec<RouterMiddlewareFunc<T>>,
  any_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  get_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  post_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  put_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  patch_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  delete_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  context: T,
}

impl Router<()> {
  pub fn new_without_context() -> Router<()> {
    Self::new(())
  }
}

impl<T: Clone + Send + Sync + 'static> Router<T> {
  pub fn new(context: T) -> Self {
    Self {
      middleware: Vec::new(),
      any_routes: Rc::new(RefCell::new(PathTree::new())),
      get_routes: Rc::new(RefCell::new(PathTree::new())),
      post_routes: Rc::new(RefCell::new(PathTree::new())),
      put_routes: Rc::new(RefCell::new(PathTree::new())),
      patch_routes: Rc::new(RefCell::new(PathTree::new())),
      delete_routes: Rc::new(RefCell::new(PathTree::new())),
      context,
    }
  }

  pub fn with_all<F, Fut>(
    &mut self,
    middleware: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<Option<(Request, Response, T)>>>,
  {
    self.middleware.push(Arc::new(move |req, res, ctx| {
      Box::pin(middleware(req, res, ctx))
    }));
  }

  pub fn with<F, Fut>(
    &mut self,
    middleware: F,
  ) -> RouteBuilder<T>
  where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<Option<(Request, Response, T)>>>,
  {
    let middleware: RouterMiddlewareFunc<T> =
      Arc::new(move |req, res, ctx| Box::pin(middleware(req, res, ctx)));

    RouteBuilder {
      middleware: vec![middleware],
      any_routes: Rc::clone(&self.any_routes),
      get_routes: Rc::clone(&self.get_routes),
      post_routes: Rc::clone(&self.post_routes),
      put_routes: Rc::clone(&self.put_routes),
      patch_routes: Rc::clone(&self.patch_routes),
      delete_routes: Rc::clone(&self.delete_routes),
    }
  }

  pub fn without_context(&mut self) -> RouteBuilderNc<T> {
    RouteBuilderNc {
      any_routes: Rc::clone(&self.any_routes),
      get_routes: Rc::clone(&self.get_routes),
      post_routes: Rc::clone(&self.post_routes),
      put_routes: Rc::clone(&self.put_routes),
      patch_routes: Rc::clone(&self.patch_routes),
      delete_routes: Rc::clone(&self.delete_routes),
    }
  }

  pub fn get<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.get_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn post<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.post_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn put<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.put_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn patch<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.patch_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn delete<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.delete_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn any<F, Fut>(
    &mut self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.any_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn handler(&self) -> crate::HandleFunc {
    let middleware = Arc::new(self.middleware.clone());
    let any_routes = Arc::new(self.any_routes.borrow().clone());
    let get_routes = Arc::new(self.get_routes.borrow().clone());
    let post_routes = Arc::new(self.post_routes.borrow().clone());
    let put_routes = Arc::new(self.put_routes.borrow().clone());
    let patch_routes = Arc::new(self.patch_routes.borrow().clone());
    let delete_routes = Arc::new(self.delete_routes.borrow().clone());
    let context = self.context.clone();

    Box::new(move |mut req, mut res| {
      let middleware = middleware.clone();
      let any_routes = any_routes.clone();
      let get_routes = get_routes.clone();
      let post_routes = post_routes.clone();
      let put_routes = put_routes.clone();
      let patch_routes = patch_routes.clone();
      let delete_routes = delete_routes.clone();
      let mut context = context.clone();

      Box::pin(async move {
        let path = req.uri.path().to_string();
        let routes = match req.method() {
          &Method::GET => get_routes,
          &Method::POST => post_routes,
          &Method::PUT => put_routes,
          &Method::PATCH => patch_routes,
          &Method::DELETE => delete_routes,
          _ => Arc::clone(&any_routes),
        };

        for middleware in &*middleware {
          let Some((new_req, new_res, new_context)) = middleware(req, res, context).await? else {
            return Ok(());
          };
          req = new_req;
          res = new_res;
          context = new_context;
        }

        if let Some(((middleware, handler), params)) = routes.find(&path) {
          for middleware in middleware {
            let Some((new_req, new_res, new_context)) = middleware(req, res, context).await? else {
              return Ok(());
            };
            req = new_req;
            res = new_res;
            context = new_context;
          }

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
          handler(req, res, context).await?;
          return Ok(());
        }

        if let Some(((middleware, handler), params)) = any_routes.find(&path) {
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

          for middleware in middleware {
            let Some((new_req, new_res, new_context)) = middleware(req, res, context).await? else {
              return Ok(());
            };
            req = new_req;
            res = new_res;
            context = new_context;
          }

          handler(req, res, context).await?;
          return Ok(());
        }

        res.write_all(b"").await?;
        res.write_head(crate::StatusCode::NOT_FOUND).await?;
        Ok(())
      })
    })
  }
}
