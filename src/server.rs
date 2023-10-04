use std::collections::HashMap;

use crate::room::{AddUser, BroadcastMessage, Room};
use crate::session::{OutgoingMessage, UpdateRoom};
use crate::User;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use rand::Rng;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub enum SuperDuperError {
    RoomNotFound,
    UsernameTaken,
    AlreadyInRoom,
    InternalServerError,
    NotLoggedIn,
    NotInRoom,
}

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct Login {
    pub username: String,
    pub session: User,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Logout {
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "Result<(), SuperDuperError>")]
pub struct JoinRoom {
    pub code: String,
    pub session: User,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CreateRoom {
    pub session: User,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DestroyRoom(pub String);

pub struct Server {
    users: HashMap<String, User>,
    rooms: HashMap<String, Addr<Room>>,
}

impl Server {
    pub fn new() -> Server {
        Self {
            users: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
    fn create_room(&mut self, leader: &User, addr: &Addr<Self>) -> (Addr<Room>, String) {
        let mut rng = rand::thread_rng();
        static CHARSET: &[u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
        let code = loop {
            let candidate = (0..8)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect::<String>();
            if self.rooms.contains_key(&candidate) {
                continue;
            } else {
                break candidate;
            }
        };
        let room = Room::new(vec![leader.to_owned()], code.clone(), addr.to_owned()).start();
        self.rooms.insert(code.clone(), room.clone());
        (room, code)
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Handler<Login> for Server {
    type Result = Result<(), ()>;
    fn handle(&mut self, msg: Login, _ctx: &mut Self::Context) -> Self::Result {
        if let std::collections::hash_map::Entry::Vacant(e) = self.users.entry(msg.username) {
            e.insert(msg.session);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Handler<Logout> for Server {
    type Result = ();
    fn handle(&mut self, msg: Logout, _ctx: &mut Self::Context) -> Self::Result {
        self.users.remove(&msg.username);
    }
}

impl Handler<JoinRoom> for Server {
    type Result = Result<(), SuperDuperError>;
    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(room) = self.rooms.get(&msg.code) {
            room.do_send(AddUser {
                session: msg.session.clone(),
                username: msg.username,
            });
            msg.session.do_send(UpdateRoom(Some(room.to_owned())));
            Ok(())
        } else {
            Err(SuperDuperError::RoomNotFound)
        }
    }
}

impl Handler<CreateRoom> for Server {
    type Result = ();
    fn handle(&mut self, msg: CreateRoom, ctx: &mut Self::Context) -> Self::Result {
        let (room, code) = self.create_room(&msg.session, &ctx.address());
        room.do_send(BroadcastMessage(OutgoingMessage::UserJoined {
            username: msg.username,
            room: code,
        }));
        msg.session.do_send(UpdateRoom(Some(room)));
    }
}

impl Handler<DestroyRoom> for Server {
    type Result = ();
    fn handle(&mut self, msg: DestroyRoom, _ctx: &mut Self::Context) -> Self::Result {
        self.rooms.remove(&msg.0);
    }
}
