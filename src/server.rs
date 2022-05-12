use actix::prelude::*;
use std::collections::HashMap;

use crate::message;

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<String, Recipient<message::Message>>,
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<message::Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::Connect, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}
