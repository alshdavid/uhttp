use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use path_tree::PathTree;

use super::RouterHandleFunc;
use super::RouterMiddlewareFunc;
use crate::Request;
use crate::Response;

pub struct RouteBuilderNc<T>
where
  T: Clone + Send + Sync + 'static,
{
  pub(super) any_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  pub(super) get_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  pub(super) post_routes:
    Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  pub(super) put_routes: Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  pub(super) patch_routes:
    Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
  pub(super) delete_routes:
    Rc<RefCell<PathTree<(Vec<RouterMiddlewareFunc<T>>, RouterHandleFunc<T>)>>>,
}

impl<T: Clone + Send + Sync + 'static> RouteBuilderNc<T> {
  pub fn any<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.any_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, _ctx| Box::pin(handler(req, res))),
      ),
    );
  }

  pub fn get<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.get_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, _ctx| Box::pin(handler(req, res))),
      ),
    );
  }

  pub fn post<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.post_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, _ctx| Box::pin(handler(req, res))),
      ),
    );
  }

  pub fn put<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.put_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, _ctx| Box::pin(handler(req, res))),
      ),
    );
  }

  pub fn patch<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.patch_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, _ctx| Box::pin(handler(req, res))),
      ),
    );
  }

  pub fn delete<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.delete_routes.borrow_mut().insert(
      route,
      (
        Vec::new(),
        Arc::new(move |req, res, _ctx| Box::pin(handler(req, res))),
      ),
    );
  }
}
