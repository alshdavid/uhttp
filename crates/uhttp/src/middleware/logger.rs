pub async fn logger_default<T>(
  req: crate::Request,
  res: crate::Response,
  ctx: T,
) -> crate::Result<Option<(crate::Request, crate::Response, T)>> {
  let method = req.method().to_string();
  let path = match req.uri().path_and_query() {
    Some(path) => path.as_str(),
    None => "",
  };

  println!("[{}] {}", method, path);
  Ok(Some((req, res, ctx)))
}

pub struct LoggerOptions {
  // todo
}

pub fn logger<T: 'static + Clone + Send + Sync>(
  _options: LoggerOptions
) -> crate::router::RouterMiddlewareFunc<T> {
  Box::new(|req, res, ctx| {
    Box::pin(async move {
      let method = req.method().to_string();
      let path = match req.uri().path_and_query() {
        Some(path) => path.as_str(),
        None => "",
      };

      println!("[{}] {}", method, path);
      Ok(Some((req, res, ctx)))
    })
  })
}
