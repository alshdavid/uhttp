pub fn without_context<T: 'static + Clone + Send + Sync>(
  handler: crate::HandleFunc
) -> crate::router::RouterHandleFunc<T> {
  Box::new(move |req, res, _ctx| handler(req, res))
}
