use warp::ws::WebSocket;
use futures::stream::{SplitSink, SplitStream};
use warp::ws::Message;


pub struct Client {
    pub websocket_send: SplitSink<WebSocket, Message>,
    pub websocket_recv: SplitStream<WebSocket>,
    pub name: String,
    pub connected: bool,
}

impl Client {
    pub fn new(
        websocket_send: SplitSink<WebSocket, Message>,
        websocket_recv: SplitStream<WebSocket>,
        name: String)
    -> Client {
        Client { name, websocket_send, websocket_recv, connected: true }
    }
}
