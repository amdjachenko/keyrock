use std::{borrow::BorrowMut, sync::Arc, time::Duration};

use crate::feeds::{
    binance::{self, BookDepth, BookPeriod, Config, Feed},
    Error,
};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time::{timeout, Timeout},
};
use tokio_tungstenite::tungstenite::{
    handshake::server::{Callback, ErrorResponse},
    http::{Request, Response, Uri},
    Message,
};

#[cfg(test)]
mod server;

#[derive(Default)]
enum Server {
    #[default]
    Default,
    Bound(TcpListener),
}

enum Connection {
    Default(Option<TcpStream>),
    Selected(UnboundedSender<Message>, UnboundedReceiver<Message>),
    Disconnected,
}

impl Server {
    async fn bind(&mut self) -> url::Url {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind");
        let addr = listener.local_addr().expect("no address");
        *self = Self::Bound(listener);
        let url = format!("ws://{addr}");
        url::Url::parse(&url).expect("invalid url")
    }

    async fn accept(&mut self) -> io::Result<Connection> {
        let listener = if let Self::Bound(listener) = self {
            listener
        } else {
            panic!("server should be in bound state")
        };

        listener.accept().await.map(|(stream, addr)| {
            println!("Incoming TCP connection from: {addr}");
            Connection::Default(Some(stream))
        })
    }
}

impl Connection {
    async fn handshake(&mut self) -> Uri {
        let stream = if let Self::Default(socket) = self {
            socket.take().unwrap()
        } else {
            panic!("request can be got only once")
        };

        let uri = Arc::new(std::sync::Mutex::new(Uri::default()));
        let uri_move = uri.clone();
        let (outgoing, incoming) =
            tokio_tungstenite::accept_hdr_async(stream, |request: &Request<()>, response| {
                uri_move.lock().unwrap().clone_from(request.uri());
                Ok(response)
            })
            .await
            .expect("Error during the websocket handshake occurred")
            .split();

        let uri = uri.lock().unwrap().to_owned();
        println!("WebSocket connection established: {uri}");

        let (tx1, rx1) = unbounded();
        let transfer = incoming.try_for_each(move |msg| {
            match &msg {
                Message::Text(_) => todo!(),
                Message::Binary(_) => todo!(),
                Message::Ping(_) => todo!(),
                Message::Pong(p) => println!("Received pong: {p:?}"),
                Message::Close(_) => todo!(),
                Message::Frame(_) => todo!(),
            }
            //println!("Received a message from {addr}: {msg}");
            tx1.unbounded_send(msg)
                .expect("error during transferring incoming message");
            future::ok(())
        });

        let (tx2, rx2) = unbounded();
        let receive = rx2.map(Ok).forward(outgoing);
        //pin_mut!(transfer, receive);
        tokio::spawn(async move {
            future::select(transfer, receive).await;
            println!("disconnected");
        });

        *self = Self::Selected(tx2, rx1);
        uri
    }

    fn send(&mut self, msg: Message) {
        match self {
            Connection::Default(_) => panic!("can't send before handshake finished"),
            Connection::Disconnected => panic!("can't send after connection closed"),
            Connection::Selected(tx, _) => {
                if let Err(_) = tx.unbounded_send(msg) {
                    *self = Self::Disconnected;
                }
            }
        }
    }

    async fn receive(&mut self) -> Option<Message> {
        match self {
            Connection::Default(_) => panic!("can't receive before handshake finished"),
            Connection::Disconnected => panic!("can't receive after connection closed"),
            Connection::Selected(_, rx) => {
                if let Some(msg) = rx.next().await {
                    Some(msg)
                } else {
                    *self = Self::Disconnected;
                    None
                }
            }
        }
    }

    fn try_receive(&mut self) -> Option<Message> {
        match self {
            Connection::Default(_) => panic!("can't receive before handshake finished"),
            Connection::Disconnected => panic!("can't receive after connection closed"),
            Connection::Selected(_, rx) => match rx.try_next() {
                Ok(Some(m)) => Some(m),
                Ok(None) | Err(_) => {
                    *self = Self::Disconnected;
                    None
                }
            },
        }
    }
}

#[tokio::test]
async fn connect() {
    let mut server = Server::default();
    let url = url::Url::parse("ws://127.0.0.1").unwrap();
    let mut feed = Config::new(url).connect();

    let feed = timeout(Duration::from_secs(1), feed);
    assert!(feed.await.is_err_and(|e| matches!(e, Ellapsed)));

    let url = server.bind().await;
    assert!(Config::new(url).connect().await.is_ok());
}

#[tokio::test]
async fn ping_from_server() {
    let mut server = Server::default();
    let url = server.bind().await;

    let mut feed = Config::new(url).connect();
    let mut connection = server.accept().await.expect("incoming connection");
    feed.await.expect("connection");
    assert!(connection.try_receive().is_none());

    connection.send(Message::Ping(Vec::default()));
    assert!(connection.receive().await.is_some_and(|msg| msg.is_pong()));
    assert!(connection.try_receive().is_none());
}

// #[tokio::test]
// async fn subscribe() {
//     let ticker = "scamcrap";
//     let depth = BookDepth::Medium;
//     let period = BookPeriod::Normal;

//     let mut server = Server::default();
//     let url = server.bind().await;
//     let mut feed = Config::new(url)
//         .subscribe(
//             ticker,
//             Stream::OrderBook(depth, period),
//         )
//         .connect();
//     let mut connection = server.accept().await.expect("incoming connection");
//     let uri = connection.handshake().await;
//     let depth = depth as u32;
//     let period = period as u32;
//     assert_eq!(uri.path(), format!("ws/{ticker}@depth{depth}@{period}ms"));
// }
