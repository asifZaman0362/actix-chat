use actix::{Addr, Actor};
use actix_web::web::Data;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, web::Payload};
use actix_web_actors::ws;

mod session;
mod server;
mod room;

use crate::session::Session;
use crate::server::Server;

pub type User = Addr<Session>;

#[get("/")]
async fn index(_req: HttpRequest) -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("./static/index.html")?)
}

#[get("/ws")]
async fn chat(req: HttpRequest, stream: Payload, server: Data<Addr<Server>>) -> actix_web::Result<HttpResponse> {
    let response = ws::start(Session::new(server.get_ref().to_owned()), &req, stream);
    log::info!("connection request: \n{response:?}");
    response
}

#[actix::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("server started...");
    let server = Server::default().start();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(server.clone()))
            .service(index)
            .service(chat)
            .service(actix_files::Files::new("/res", "./static/res"))
    })
    .bind(("localhost", 8000))?
    .workers(4)
    .run()
    .await
}
