use wasm_bindgen::prelude::*;
use web_sys::{WebSocket, MessageEvent};
use wasm_bindgen_futures::JsFuture;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct Connection {
    ws:WebSocket,
    message_queue:Rc<RefCell<VecDeque<Vec<u8>>>>,
}

impl Connection {
    pub fn sleep() -> js_sys::Promise {
        js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10)
                .unwrap();
        })
    }

    pub async fn new(window: &web_sys::Window) -> Connection {
        let hostname = window.location().hostname().unwrap();
        let port = window.location().port().unwrap();

        let url = format!("ws://{}:{}/ws", hostname, port);

        let ws = WebSocket::new(&url).expect("Error connecting to websocket");
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let message_queue:Rc<RefCell<VecDeque<Vec<u8>>>> = Rc::new(RefCell::new(VecDeque::new()));
        {
            let message_queue = message_queue.clone();

            let callback = Closure::<dyn FnMut(_)>::new(move |message: MessageEvent| {
                let message = message
                    .data()
                    .dyn_into::<js_sys::ArrayBuffer>()
                    .expect("Unable to parse server message");

                let message = js_sys::Uint8Array::new(&message);
                let message: Vec<u8> = message.to_vec();
                message_queue.borrow_mut().push_back(message);
            });

            ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        }


        while ws.ready_state() != WebSocket::OPEN {
            let _ = JsFuture::from(Self::sleep()).await;
        }

        return Connection {
            ws,
            message_queue,
        }
    }

    pub fn send(&self, message:Vec<u8>) {
        self.ws.send_with_u8_array(&message).expect("Unable to send websocket message");
    }

    pub fn next_message(&self) -> Option<Vec<u8>> {
        let mut message_queue = self.message_queue.borrow_mut();
        let message = message_queue.pop_front().clone();
        return message;
    }

    pub fn disconnect(&self) {
        let _ = self.ws.close();
    }
}
