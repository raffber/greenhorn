use crate::pipe::{Pipe, RxMsg, TxMsg};
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::{accept_async, WebSocketStream};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::prelude::*;
use futures::select;
use futures::task::{Context, Poll};
use futures::Stream;
use log::error;
use std::net::SocketAddr;
use std::pin::Pin;
use tungstenite::protocol::Message;
use futures::Sink;
use std::error::Error;
use futures::channel::mpsc::SendError;

struct PipeChannels {
    req_tx: UnboundedSender<Message>,
    req_rx: UnboundedReceiver<Message>,
    resp_tx: UnboundedSender<Message>,
    resp_rx: UnboundedReceiver<Message>,
}

impl PipeChannels {
    fn new() -> Self {
        let (req_tx, req_rx) = unbounded();
        let (resp_tx, resp_rx) = unbounded();
        PipeChannels {
            req_tx,
            resp_tx,
            req_rx,
            resp_rx,
        }
    }
}

pub struct WebsocketPipe {
    resp_rx: UnboundedReceiver<Message>,
    req_tx: UnboundedSender<Message>,
    addr: SocketAddr,
}

impl WebsocketPipe {
    pub fn listen_to_addr(addr: SocketAddr) -> WebsocketPipe {
        let try_socket = task::block_on(async {
            TcpListener::bind(&addr).await
        });
        let listener = try_socket.expect("Failed to bind");
        Self::listen_to_socket(listener)
    }

    pub fn listen_to_socket(listener: TcpListener) -> WebsocketPipe {
        let channels = PipeChannels::new();
        let resp_tx = channels.resp_tx;
        let req_rx = channels.req_rx;
        let local_addr = listener.local_addr().unwrap();
        let local_addr_cloned = local_addr;
        task::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let ws = accept_async(stream).await.expect("Error during handshake");
                let mut handler = ConnectionHandler {
                    ws,
                    resp_tx,
                    req_rx,
                };
                handler.run().await;
            } else {
                error!("Could not accept connection on: {}", local_addr_cloned);
            }
        });
        WebsocketPipe {
            resp_rx: channels.resp_rx,
            req_tx: channels.req_tx,
            addr: local_addr,
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }
}

struct ConnectionHandler {
    ws: WebSocketStream<TcpStream>,
    resp_tx: UnboundedSender<Message>,
    req_rx: UnboundedReceiver<Message>,
}

impl ConnectionHandler {
    async fn run(&mut self) {
        loop {
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


pub struct WebsocketSender {
    req_tx: UnboundedSender<Message>,
}

impl Sink<TxMsg> for WebsocketSender {
    type Error = Box<dyn Error>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let ret: Poll<Result<(), SendError>> = UnboundedSender::poll_ready(&self.req_tx, cx);
        match ret {
            Poll::Ready(x) => Poll::Ready(x.map_err(|x| Box::new(x).into())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: TxMsg) -> Result<(), Self::Error> {
        let msg = match item {
            TxMsg::Patch(p) => {
                Message::Binary(p)
            },
            msg => {
                // for performance notes regarding serialization and underlying transport, refer to index.js
                // tldr: JSON.parse() in the browser is very fast
                let msg = serde_json::to_string(&msg).unwrap();
                Message::Text(msg)
            }
        };

        UnboundedSender::start_send(&mut self.req_tx, msg).map_err(|x| Box::new(x).into())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let pin = Pin::new(&mut self.req_tx);
        pin.poll_flush(cx).map_err(|x| Box::new(x).into())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let pin = Pin::new(&mut self.req_tx);
        pin.poll_close(cx).map_err(|x| Box::new(x).into())
    }
}

impl Clone for WebsocketSender {
    fn clone(&self) -> Self {
        Self {
            req_tx: self.req_tx.clone(),
        }
    }
}

pub struct WebsocketReceiver {
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
                }
                Message::Binary(_) => Poll::Pending,
                Message::Ping(_) => Poll::Pending,
                Message::Pong(_) => Poll::Pending,
                Message::Close(_) => Poll::Ready(None),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use async_tungstenite::async_std::connect_async;
    use std::str::FromStr;
    use url::Url;

    #[test]
    fn test_accept() {
        let addr = SocketAddr::from_str("127.0.0.1:5903").unwrap();
        let mut pipe = WebsocketPipe::listen_to_addr(addr);
        let handle = task::spawn(async move {
            let url = Url::parse("ws://127.0.0.1:5903").unwrap();
            let (mut stream, _) = connect_async(url).await.expect("Failed to connect!");
            if let Some(msg) = stream.next().await {
                match msg.unwrap() {
                    Message::Text(txt) => assert_eq!(txt, "Hello, World".to_string()),
                    _ => panic!(),
                }
            }
            stream.close(None).await.unwrap();
        });
        task::block_on(async move {
            pipe.req_tx
                .unbounded_send(Message::Text("Hello, World".into()))
                .unwrap();
            let msg = pipe.resp_rx.next().await;
            match msg {
                None => pipe.req_tx.close_channel(),
                Some(Message::Close(_)) => pipe.req_tx.close_channel(),
                _ => panic!(),
            }
        });
        task::block_on(handle);
    }

    #[test]
    fn test_close_client() {
        let addr = SocketAddr::from_str("127.0.0.1:5904").unwrap();
        let pipe = WebsocketPipe::listen_to_addr(addr);
        let client = task::spawn(async move {
            let url = Url::parse("ws://127.0.0.1:5904").unwrap();
            let (mut stream, _) = connect_async(url).await.expect("Failed to connect!");
            while let Some(msg) = stream.next().await {
                // receive one message, then terminate
                match msg {
                    Ok(Message::Text(data))=> {
                        let msg: TxMsg = serde_json::from_str(&data).unwrap();
                        assert_matches!(msg, TxMsg::Ping());
                        break;
                    }
                    _ => panic!(),
                }
            }
            stream.close(None).await.unwrap();
        });
        let (tx, mut rx) = pipe.split();
        tx.send(TxMsg::Ping());
        task::block_on(async move {
            while let Some(_) = rx.next().await {
                // we are not expecting any message, but the stream
                // should terminate immediately
                panic!()
            }
        });
        task::block_on(client);
    }

    #[test]
    fn test_close_server() {
        let addr = SocketAddr::from_str("127.0.0.1:5905").unwrap();
        let pipe = WebsocketPipe::listen_to_addr(addr);
        let client = task::spawn(async move {
            let url = Url::parse("ws://127.0.0.1:5905").unwrap();
            let (mut stream, _) = connect_async(url).await.expect("Failed to connect!");
            while let Some(msg) = stream.next().await {
                // receive one message, then terminate
                match msg.unwrap() {
                    Message::Close(None) => {
                        break;
                    }
                    _ => panic!(),
                }
            }
        });
        let (tx, mut rx) = pipe.split();
        tx.close();
        task::block_on(async move {
            while let Some(_) = rx.next().await {
                // we are not expecting any message, but the stream
                // should terminate immediately
                panic!()
            }
        });
        task::block_on(client);
    }
}
