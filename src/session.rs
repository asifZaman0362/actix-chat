use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler, Message,
    StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{self, ProtocolError, WebsocketContext};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::room::{BroadcastMessage, RemoveUser, Room};
use crate::server::{CreateRoom, JoinRoom, Login, Logout, Server, SuperDuperError};

#[derive(Debug, Deserialize)]
pub enum IncomingMessage {
    Login(String),
    Message(String),
    Logout,
    JoinRoom(String),
    CreateRoom,
    LeaveRoom,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateRoom(pub Option<Addr<Room>>);

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub enum OutgoingMessage {
    Chat { username: String, message: String },
    RemoveFromRoom,
    Error(SuperDuperError),
    UserJoined { username: String, room: String },
    UserLeft(String),
    LoginSuccess,
}

pub struct Session {
    pub username: Option<String>,
    pub room: Option<Addr<Room>>,
    pub server: Addr<Server>,
}

impl Session {
    pub fn new(server: Addr<Server>) -> Self {
        Session {
            username: None,
            room: None,
            server,
        }
    }

    fn handle_message(&mut self, msg: IncomingMessage, ctx: &mut <Self as Actor>::Context) {
        let session = ctx.address().clone();
        match msg {
            IncomingMessage::Login(username) => {
                let username_copy = username.clone();
                self.server
                    .send(Login { username, session })
                    .into_actor(self)
                    .then(|res, act, inner_ctx| {
                        match res {
                            Ok(Ok(())) => {
                                act.username = Some(username_copy);
                                inner_ctx.text(to_string(&OutgoingMessage::LoginSuccess).unwrap())
                            }
                            Ok(Err(())) => inner_ctx.text(
                                to_string(&OutgoingMessage::Error(SuperDuperError::UsernameTaken))
                                    .unwrap(),
                            ),
                            Err(_) => {
                                log::error!("server mailbox full!");
                                inner_ctx.text(
                                    to_string(&OutgoingMessage::Error(
                                        SuperDuperError::InternalServerError,
                                    ))
                                    .unwrap(),
                                );
                            }
                        }
                        actix::fut::ready(())
                    })
                    .wait(ctx);
            }
            IncomingMessage::Logout => {
                if let Some(username) = self.username.clone() {
                    if let Some(room) = &self.room {
                        room.do_send(RemoveUser {
                            session,
                            username: username.clone(),
                        });
                    }
                    self.server.do_send(Logout { username });
                    self.username = None;
                }
            }
            IncomingMessage::CreateRoom => {
                if self.room.is_some() {
                    ctx.text(
                        to_string(&OutgoingMessage::Error(SuperDuperError::AlreadyInRoom)).unwrap(),
                    );
                } else if let Some(username) = self.username.clone() {
                    self.server.do_send(CreateRoom { username, session });
                } else {
                    ctx.text(
                        to_string(&OutgoingMessage::Error(SuperDuperError::NotLoggedIn)).unwrap(),
                    );
                }
            }
            IncomingMessage::JoinRoom(code) => {
                if self.room.is_some() {
                    ctx.text(
                        to_string(&OutgoingMessage::Error(SuperDuperError::AlreadyInRoom)).unwrap(),
                    );
                } else if let Some(username) = self.username.clone() {
                    self.server
                        .send(JoinRoom {
                            code,
                            username,
                            session,
                        })
                        .into_actor(self)
                        .then(|res, _, inner_ctx| {
                            match res {
                                Ok(Ok(())) => {}
                                Ok(Err(err)) => {
                                    inner_ctx.text(to_string(&OutgoingMessage::Error(err)).unwrap())
                                }
                                Err(_) => inner_ctx.text(
                                    to_string(&OutgoingMessage::Error(
                                        SuperDuperError::InternalServerError,
                                    ))
                                    .unwrap(),
                                ),
                            }
                            actix::fut::ready(())
                        })
                        .wait(ctx);
                } else {
                    ctx.text(
                        to_string(&OutgoingMessage::Error(SuperDuperError::NotLoggedIn)).unwrap(),
                    );
                }
            }
            IncomingMessage::Message(message) => {
                if let Some(username) = self.username.clone() {
                    if let Some(room) = self.room.clone() {
                        room.do_send(BroadcastMessage(OutgoingMessage::Chat {
                            username,
                            message,
                        }));
                    } else {
                        ctx.text(
                            to_string(&OutgoingMessage::Error(SuperDuperError::NotInRoom)).unwrap(),
                        );
                    }
                } else {
                    ctx.text(
                        to_string(&OutgoingMessage::Error(SuperDuperError::NotLoggedIn)).unwrap(),
                    );
                }
            }
            IncomingMessage::LeaveRoom => {
                if let Some(room) = self.room.clone() {
                    if let Some(username) = self.username.clone() {
                        room.do_send(RemoveUser { username, session });
                        self.room = None;
                    }
                }
            }
        }
    }
}

impl Actor for Session {
    type Context = WebsocketContext<Self>;
    fn stopped(&mut self, _ctx: &mut Self::Context) {
    }
    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        if let Some(username) = self.username.clone() {
            log::info!("logging out");
            self.server.do_send(Logout { username });
        }
        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ProtocolError>> for Session {
    fn handle(&mut self, item: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Pong(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Text(json)) => {
                log::info!("message received: {json}");
                match from_str::<IncomingMessage>(&json) {
                    Ok(msg) => self.handle_message(msg, ctx),
                    Err(err) => log::error!("error parsing message from json: {err:?}"),
                }
            }
            Ok(_) => {}
            Err(err) => log::error!("websocket error: {err:?}"),
        }
    }
}

impl Handler<OutgoingMessage> for Session {
    type Result = ();
    fn handle(&mut self, msg: OutgoingMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(to_string(&msg).unwrap());
    }
}

impl Handler<UpdateRoom> for Session {
    type Result = ();
    fn handle(&mut self, msg: UpdateRoom, ctx: &mut Self::Context) -> Self::Result {
        self.room = msg.0.clone();
        if msg.0.is_none() {
            ctx.text(to_string(&OutgoingMessage::RemoveFromRoom).unwrap());
        }
    }
}
