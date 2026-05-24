use std::sync::Arc;

/*
  Test with:
  curl http://localhost:8080
  curl http://localhost:8080/foo
  curl http://localhost:8080/bar
  curl http://localhost:8080/fizz/something
*/
use uhttp::*;

#[derive(Debug, Clone)]
struct Context {
  test: Option<String>,
  service: Arc<String>,
}

async fn middleware_logger<T>(
  req: uhttp::Request,
  res: uhttp::Response,
  ctx: T,
) -> uhttp::Result<Option<(uhttp::Request, uhttp::Response, T)>> {
  println!("[{}] {}", req.method(), req.uri().path());
  Ok(Some((req, res, ctx)))
}

async fn middleware_value_set(
  req: uhttp::Request,
  res: uhttp::Response,
  mut ctx: Context,
) -> uhttp::Result<Option<(uhttp::Request, uhttp::Response, Context)>> {
  ctx.test = Some("something".to_string());
  Ok(Some((req, res, ctx)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let context = Context {
    test: None,
    service: Arc::new("OK".to_string()),
  };

  let mut app = uhttp::router::Router::new(context);

  app.with_all(middleware_logger);
  // app.with_all(middleware_value_set);

  app
    .with(middleware_value_set)
    .get("/foo", |_req, mut res, ctx| async move {
      dbg!(&ctx);
      println!("hiii");
      res.write(b"foo\n").await?;
      Ok(())
    });

  app.post("/bar", |_req, mut res, ctx| async move {
    res.write(b"bar\n").await?;
    Ok(())
  });

  app.get("/bar", |_req, mut res, ctx| async move {
    dbg!(&ctx);
    res.write(b"bar\n").await?;
    Ok(())
  });

  app.get("/fizz/:buzz", |req, mut res, ctx| async move {
    res.write(b"fizz\n").await?;

    let Some(buzz) = req.url_param("buzz") else {
      res.write_head(uhttp::StatusCode::BAD_REQUEST).await?;
      return Ok(());
    };

    res.write_all(format!("Param: {}", buzz).as_bytes()).await?;
    Ok(())
  });

  app.get("/*", |_req, mut res, ctx| async move {
    res.write(b"Not found route").await?;
    Ok(())
  });

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
