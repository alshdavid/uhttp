use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use path_tree::PathTree;

use super::PathTreeRoute;
use super::RouterHandleFuncInner;
use super::RouterMiddlewareFuncInner;
use crate::Request;
use crate::Response;

pub struct RouteBuilder<T>
where
  T: Clone + Send + Sync + 'static,
{
  pub(super) middleware: Vec<RouterMiddlewareFuncInner<T>>,
  pub(super) any_routes: Rc<RefCell<PathTree<PathTreeRoute<T>>>>,
  pub(super) get_routes: Rc<RefCell<PathTree<PathTreeRoute<T>>>>,
  pub(super) post_routes: Rc<RefCell<PathTree<PathTreeRoute<T>>>>,
  pub(super) put_routes: Rc<RefCell<PathTree<PathTreeRoute<T>>>>,
  pub(super) patch_routes: Rc<RefCell<PathTree<PathTreeRoute<T>>>>,
  pub(super) delete_routes: Rc<RefCell<PathTree<PathTreeRoute<T>>>>,
}

impl<T: Clone + Send + Sync + 'static> RouteBuilder<T> {
  pub fn with<F, Fut>(
    mut self,
    middleware: F,
  ) -> Self
  where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<Option<(Request, Response, T)>>>,
  {
    self.middleware.push(Arc::new(move |req, res, ctx| {
      Box::pin(middleware(req, res, ctx))
    }));
    self
  }

  pub fn any<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let handler: RouterHandleFuncInner<T> =
      Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx)));

    let _ = self
      .get_routes
      .borrow_mut()
      .insert(route, (Vec::new(), Arc::clone(&handler)));

    let _ = self
      .post_routes
      .borrow_mut()
      .insert(route, (Vec::new(), Arc::clone(&handler)));

    let _ = self
      .put_routes
      .borrow_mut()
      .insert(route, (Vec::new(), Arc::clone(&handler)));

    let _ = self
      .patch_routes
      .borrow_mut()
      .insert(route, (Vec::new(), Arc::clone(&handler)));

    let _ = self
      .delete_routes
      .borrow_mut()
      .insert(route, (Vec::new(), Arc::clone(&handler)));

    let _ = self
      .any_routes
      .borrow_mut()
      .insert(route, (Vec::new(), handler));
  }

  pub fn get<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.get_routes.borrow_mut().insert(
      route,
      (
        self.middleware,
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn post<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.post_routes.borrow_mut().insert(
      route,
      (
        self.middleware,
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn put<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.put_routes.borrow_mut().insert(
      route,
      (
        self.middleware,
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn patch<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.patch_routes.borrow_mut().insert(
      route,
      (
        self.middleware,
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }

  pub fn delete<F, Fut>(
    self,
    route: &str,
    handler: F,
  ) where
    F: 'static + Send + Sync + Fn(Request, Response, T) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let _ = self.delete_routes.borrow_mut().insert(
      route,
      (
        self.middleware,
        Arc::new(move |req, res, ctx| Box::pin(handler(req, res, ctx))),
      ),
    );
  }
}
