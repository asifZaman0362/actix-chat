use actix_files;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, web::Payload};
use actix_web_actors::ws;

mod session;

use crate::session::Session;

#[get("/")]
async fn index(_req: HttpRequest) -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("./static/index.html")?)
}

#[get("/ws")]
async fn chat(req: HttpRequest, stream: Payload) -> actix_web::Result<HttpResponse> {
    let response = ws::start(Session{}, &req, stream);
    log::info!("connection request: \n{response:?}");
    response
}

#[actix::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("server started...");
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(chat)
            .service(actix_files::Files::new("/res", "./static/res"))
    })
    .bind(("localhost", 8000))?
    .workers(4)
    .run()
    .await
}
