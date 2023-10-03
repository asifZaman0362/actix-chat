use actix::{Actor, Message, Addr, Handler, StreamHandler};
use actix_web_actors::ws::{self, WebsocketContext, ProtocolError};

pub struct Session {}

impl Actor for Session {
    type Context = WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ProtocolError>> for Session {
    fn handle(&mut self, item: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Pong(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Text(json)) => {
                log::info!("message received: {json}");
            }
            Ok(_) => {}
            Err(err) => log::error!("websocket error: {err:?}")
        }
    }
}
