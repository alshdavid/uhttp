use actix_web::get;
use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;

#[get("/")]
async fn index() -> impl Responder {
  "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(|| App::new().service(index))
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
