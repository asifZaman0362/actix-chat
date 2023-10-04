use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};

use crate::{
    server::{DestroyRoom, Server},
    session::OutgoingMessage,
    User,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddUser {
    pub session: User,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveUser(pub User);

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub OutgoingMessage);

pub struct Room {
    users: Vec<User>,
    code: String,
    server: Addr<Server>,
}

impl Room {
    pub fn new(users: Vec<User>, code: String, server: Addr<Server>) -> Self {
        Self {
            users,
            code,
            server,
        }
    }
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Handler<AddUser> for Room {
    type Result = ();
    fn handle(&mut self, msg: AddUser, ctx: &mut Self::Context) -> Self::Result {
        self.users.push(msg.session);
        ctx.address()
            .do_send(BroadcastMessage(OutgoingMessage::UserJoined {
                username: msg.username,
                room: self.code.clone(),
            }));
    }
}

impl Handler<RemoveUser> for Room {
    type Result = ();
    fn handle(&mut self, msg: RemoveUser, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(idx) = self.users.iter().position(|x| *x == msg.0) {
            self.users.remove(idx);
        }
        if self.users.is_empty() {
            self.server.do_send(DestroyRoom(self.code.clone()));
        }
    }
}

impl Handler<BroadcastMessage> for Room {
    type Result = ();
    fn handle(&mut self, msg: BroadcastMessage, _ctx: &mut Self::Context) -> Self::Result {
        for user in &self.users {
            user.do_send(msg.0.clone());
        }
    }
}
