/*
  Test with:
  curl http://localhost:8080
  curl http://localhost:8080/foo
  curl http://localhost:8080/bar
  curl http://localhost:8080/fizz/something
*/
use uhttp::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut app = uhttp::router::Router::new_without_context();

  app.get("/foo", |_req, mut res, _ctx| async move {
    res.write(b"foo\n").await?;
    Ok(())
  });

  app.post("/bar", |_req, mut res, _ctx| async move {
    res.write(b"bar\n").await?;
    Ok(())
  });

  app.get("/bar", |_req, mut res, _ctx| async move {
    res.write(b"bar\n").await?;
    Ok(())
  });

  app.get("/fizz/:buzz", |req, mut res, _ctx| async move {
    res.write(b"fizz\n").await?;

    let Some(buzz) = req.url_param("buzz") else {
      res.write_head(uhttp::StatusCode::BAD_REQUEST).await?;
      return Ok(());
    };

    res.write_all(format!("Param: {}", buzz).as_bytes()).await?;
    Ok(())
  });

  app.any("/*", |_req, mut res, _ctx| async move {
    res.write(b"Not found route").await?;
    Ok(())
  });

  uhttp::http1::create_server(app.handler())
    .listen("0.0.0.0:8080")
    .await?;

  Ok(())
}
