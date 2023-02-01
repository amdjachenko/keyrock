use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{self, WebSocket};
use tungstenite::protocol::Message;

/*pub async fn start() -> Result<(SocketAddr, WebSocket<TcpStream>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let server_addr = listener.local_addr()?;
    println!("Listening on: {}", server_addr);
    let (stream, addr) = listener.accept().await?;
    println!("Incoming TCP connection from: {}", addr);

    let stream = tokio_tungstenite::accept_async(raw_stream).await?;
    println!("WebSocket connection established: {}", addr);
    (server_addr, stream)
}
*/
