use crate::pipe::{Pipe, RxMsg, Sender, TxMsg};
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::{accept_async, WebSocketStream};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::prelude::*;
use futures::select;
use futures::task::{Context, Poll};
use futures::Stream;
use log::{error, info};
use std::net::SocketAddr;
use std::pin::Pin;
use tungstenite::protocol::Message;

struct WebsocketPipeBuilder {
    addr: SocketAddr,
    req_tx: UnboundedSender<Message>,
    req_rx: UnboundedReceiver<Message>,
    resp_tx: UnboundedSender<Message>,
    resp_rx: UnboundedReceiver<Message>,
}

struct WebsocketPipe {
    resp_rx: UnboundedReceiver<Message>,
    req_tx: UnboundedSender<Message>,
}

impl WebsocketPipe {
    fn new(addr: SocketAddr) -> WebsocketPipeBuilder {
        let (req_tx, req_rx) = unbounded();
        let (resp_tx, resp_rx) = unbounded();
        WebsocketPipeBuilder {
            addr,
            req_tx,
            resp_tx,
            req_rx,
            resp_rx,
        }
    }
}

struct ConnectionHandler {
    ws: WebSocketStream<TcpStream>,
    resp_tx: UnboundedSender<Message>,
    req_rx: UnboundedReceiver<Message>,
}

impl ConnectionHandler {
    async fn run(&mut self) {
        select! {
            msg = self.req_rx.next().fuse() => {
                if !self.tx_msg(msg).await {
                    return;
                }
            },
            msg = self.ws.next().fuse() => {
                if !self.rx_msg(msg).await {
                    return;
                }
            }
        }
    }

    async fn tx_msg(&mut self, msg: Option<Message>) -> bool {
        match msg {
            None => {
                self.resp_tx.close_channel();
                self.ws.close(None).await.unwrap();
                false
            }
            Some(msg) => {
                if let Err(_e) = self.ws.send(msg).await {
                    self.resp_tx.close_channel();
                    self.ws.close(None).await.unwrap();
                    return false;
                }
                true
            }
        }
    }

    async fn rx_msg(&mut self, msg: Option<Result<Message, tungstenite::Error>>) -> bool {
        if let Some(msg) = msg {
            match msg {
                Ok(msg) => {
                    self.resp_tx.unbounded_send(msg).unwrap();
                    true
                }
                Err(err) => {
                    self.resp_tx.close_channel();
                    log::error!("Websocket Error Occured: {}", err);
                    false
                }
            }
        } else {
            self.resp_tx.close_channel();
            false
        }
    }
}

impl WebsocketPipeBuilder {
    fn listen(self) -> WebsocketPipe {
        let resp_tx = self.resp_tx;
        let req_rx = self.req_rx;
        let addr = self.addr;
        task::spawn(async move {
            let try_socket = TcpListener::bind(&addr).await;
            let listener = try_socket.expect("Failed to bind");
            info!("Listening on: {}", addr);
            if let Ok((stream, _)) = listener.accept().await {
                let ws = accept_async(stream).await.expect("Error during handshake");
                let mut handler = ConnectionHandler {
                    ws,
                    resp_tx,
                    req_rx,
                };
                handler.run().await;
            } else {
                error!("Could not accept connection on: {}", addr);
            }
        });
        WebsocketPipe {
            resp_rx: self.resp_rx,
            req_tx: self.req_tx,
        }
    }
}

struct WebsocketSender {
    req_tx: UnboundedSender<Message>,
}

impl Clone for WebsocketSender {
    fn clone(&self) -> Self {
        Self {
            req_tx: self.req_tx.clone(),
        }
    }
}

impl Sender for WebsocketSender {
    fn send(&self, msg: TxMsg) {
        let msg = serde_cbor::to_vec(&msg).unwrap();
        self.req_tx.unbounded_send(Message::Binary(msg)).unwrap();
    }
}

struct WebsocketReceiver {
    resp_rx: UnboundedReceiver<Message>,
}

impl Pipe for WebsocketPipe {
    type Sender = WebsocketSender;
    type Receiver = WebsocketReceiver;

    fn split(self) -> (Self::Sender, Self::Receiver) {
        (
            WebsocketSender {
                req_tx: self.req_tx,
            },
            WebsocketReceiver {
                resp_rx: self.resp_rx,
            },
        )
    }
}

impl Stream for WebsocketReceiver {
    type Item = RxMsg;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let pin = Pin::new(&mut self.resp_rx);
        let ret: Poll<Option<Message>> = pin.poll_next(cx);
        match ret {
            Poll::Ready(Some(msg)) => match msg {
                Message::Text(data) => match serde_json::from_str(&data).ok() {
                    None => Poll::Pending,
                    Some(x) => Poll::Ready(Some(x)),
                },
                Message::Binary(data) => match serde_cbor::from_slice(&data).ok() {
                    None => Poll::Pending,
                    Some(x) => Poll::Ready(Some(x)),
                },
                Message::Ping(_) => Poll::Pending,
                Message::Pong(_) => Poll::Pending,
                Message::Close(_) => Poll::Ready(None),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
