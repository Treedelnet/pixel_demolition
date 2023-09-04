use warp::ws::WebSocket;
use futures::stream::{SplitSink, SplitStream};
use warp::ws::Message;

use crate::client::Client;

pub struct GameMatch {
    pub state: u8,
    pub code: String,
    pub clients: Vec<Client>,
}

impl GameMatch {
    pub const GAME_CODE_LEN:i32 = 6;

    pub const UNUSED:u8 = 0;
    pub const LOBBY:u8 = 1;
    pub const GAME:u8 = 2;

    pub fn new() -> GameMatch {
        GameMatch {
            state: Self::UNUSED,
            code: String::new(),
            clients: Vec::new(),
        }
    }

    pub fn push_client(
        &mut self,
        websocket_send: SplitSink<WebSocket, Message>,
        websocket_recv: SplitStream<WebSocket>,
        name: String
    ) {
        let new_client = Client::new(websocket_send, websocket_recv, name);

        self.clients.push(new_client);
    }
}
