use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Websocket error: {0}")]
    WS(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Binance error: {0}")]
    Binance(String),
}

pub mod binance;
