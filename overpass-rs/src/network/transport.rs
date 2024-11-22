use crate::network::protocol::Message;
use futures::channel::mpsc;
use futures::StreamExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;
use web_sys::WebSocket;

use js_sys;

pub struct Transport {
    socket: WebSocket,
    receiver: mpsc::UnboundedReceiver<Message>,
    sender: mpsc::UnboundedSender<Message>,
}

impl Transport {
    /// Creates a new `Transport` bound to the specified address.
    pub async fn new(addr: String) -> Result<Self, JsValue> {
        let socket = WebSocket::new(&format!("ws://{}", addr))?;
        let (tx, rx) = mpsc::unbounded();
        let tx_clone = tx.clone();

        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            let tx = tx_clone.clone();
            if let Ok(data) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&data);
                let mut buf = vec![0; array.length() as usize];
                array.copy_to(&mut buf);

                if let Ok(message) = Message::from_bytes(&buf) {
                    let _ = tx.unbounded_send(message);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        Ok(Transport {
            socket,
            receiver: rx,
            sender: tx,
        })
    }

    /// Sends a message to the specified address.
    pub async fn send(&self, message: Message) -> Result<(), JsValue> {
        let bytes = message
            .to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let array = js_sys::Uint8Array::from(&bytes[..]);
        self.socket.send_with_u8_array(&array)
    }

    /// Receives a message from the transport.
    pub async fn receive(&mut self) -> Option<Message> {
        self.receiver.next().await
    }
}
