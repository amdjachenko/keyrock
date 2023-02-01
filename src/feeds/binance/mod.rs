use crate::{
    core::{
        Amount, Order, OrderBookAsks, OrderBookBids, OrderBookDiffAsks, OrderBookDiffBids, Price,
    },
    *,
};
use std::{
    collections::HashMap, error::Error, fmt::Pointer, net::SocketAddr, sync::Arc, time::Duration,
};

use futures_channel::mpsc::UnboundedSender;
use futures_util::{
    future,
    lock::{Mutex, MutexGuard},
    select, SinkExt, StreamExt, TryStreamExt,
};
use strum::{EnumIter, IntoEnumIterator};
use tokio::{
    net::TcpStream,
    task::JoinHandle,
    time::{self, Timeout},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::request::Builder, Message},
    MaybeTlsStream, WebSocketStream,
};

use self::events::{OrderBook, OrderBookDiff};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum BookPeriod {
    #[default]
    Normal = 1000,
    Fast = 100,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum BookDepth {
    Small = 5,
    #[default]
    Medium = 10,
    Large = 20,
}

type OrderBookTx = UnboundedSender<(core::OrderBookBids, core::OrderBookAsks)>;

#[derive(EnumIter)]
enum SubscriptionMember {
    OrderBook,
}

// #[derive(Clone)]
// struct SubscriptionIter<'a> {
//     subscription: &'a Subscriptions,
//     iter: SubscriptionMemberIter,
// }

// impl<'a> Iterator for SubscriptionIter<'a> {
//     type Item = String;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter
//             .filter_map(|member| match member {
//                 SubscriptionMember::OrderBook => self
//                     .subscription
//                     .order_book
//                     .as_ref()
//                     .map(|state| state.to_subscription_string()),
//             })
//             .next()
//     }
// }

#[derive(Clone)]
struct OrderBookSubscriptionState {
    tx: OrderBookTx,
    period: BookPeriod,
    depth: Option<BookDepth>,
    bids: OrderBookBids,
    asks: OrderBookAsks,
}

impl OrderBookSubscriptionState {
    fn new(tx: OrderBookTx, period: BookPeriod, depth: Option<BookDepth>) -> Self {
        Self {
            tx,
            period,
            depth,
            bids: Default::default(),
            asks: Default::default(),
        }
    }

    fn to_subscription_string(&self) -> String {
        format!(
            "depth{}@{}ms",
            self.depth
                .map_or(String::default(), |d| (d as u8).to_string()),
            self.period as u8
        )
    }
}

#[derive(Clone)]
struct Subscriptions {
    order_book: Option<OrderBookSubscriptionState>,
}

// impl<'a> IntoIterator for &'a Subscriptions {
//     type Item = <Self::IntoIter as Iterator>::Item;
//     type IntoIter = SubscriptionIter<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }

#[derive(Clone)]
struct Config {
    url: url::Url,
    subscriptions: HashMap<String, Subscriptions>,
    depth_order_book: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(url::Url::parse("wss://stream.binance.com:443").unwrap())
    }
}

impl Config {
    pub fn new(url: url::Url) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
    pub fn subscribe_order_book(
        mut self,
        tx: OrderBookTx,
        symbol: String,
        period: BookPeriod,
        depth: Option<BookDepth>,
    ) -> Self {
        assert!(
            self.subscriptions[&symbol].order_book.is_none(),
            "order book stream has already subscribed for {symbol}"
        );
        assert!(
            depth.is_none() || self.depth_order_book.is_empty(),
            "Partial Book Depth Streams don't contain symbol to distinguish them"
        );
        if depth.is_some() {
            self.depth_order_book = symbol.clone();
        };
        self.subscriptions.insert(
            symbol,
            feeds::binance::Subscriptions {
                order_book: Some(OrderBookSubscriptionState::new(tx, period, depth)),
            },
        );
        self
    }
    pub async fn connect(self) -> Result<Feed, feeds::Error> {
        let mut url = self.url.clone();
        url.set_path("stream");
        url.set_query(Some(
            format!(
                "streams={}",
                self.subscriptions
                    .iter()
                    .map(|(symbol, subscriptions)| {
                        SubscriptionMember::iter()
                            .filter_map(|member| match member {
                                SubscriptionMember::OrderBook => subscriptions
                                    .order_book
                                    .as_ref()
                                    .map(|state| state.to_subscription_string()),
                            })
                            .next()
                            .into_iter()
                            .map(move |string| format!("{symbol}@{string}"))
                            .intersperse("/".into())
                    })
                    .flatten()
                    .intersperse("/".into())
                    .collect::<String>()
            )
            .as_str(),
        ));
        Ok(Feed::new(connect_async(url).await?.0, self))
    }
}

struct Feed {
    config: Config,
}

impl Drop for Feed {
    fn drop(&mut self) {}
}

impl Feed {
    fn depth_update(config: &mut Config, diff: OrderBookDiff) -> Result<(), feeds::Error> {
        let state = config
            .subscriptions
            .get_mut(&diff.symbol)
            .expect("message for unsubscribed symbol")
            .order_book
            .as_mut()
            .expect("message for unsubscribed stream");

        let bids: Result<Vec<_>, f64> = diff
            .bids
            .iter()
            .map(|order| {
                Ok(Order::new(
                    Price::new(order.price)?,
                    Amount::new(order.quantity)?,
                ))
            })
            .collect();
        let bids = OrderBookDiffBids::new(bids.map_err(|e| feeds::Error::Binance(e.to_string()))?)
            .map_err(|e| feeds::Error::Binance(e.to_string()))?;
        let bids = state.bids.update(&bids);
        state.bids = bids.clone();

        let asks: Result<Vec<_>, f64> = diff
            .asks
            .iter()
            .map(|order| {
                Ok(Order::new(
                    Price::new(order.price)?,
                    Amount::new(order.quantity)?,
                ))
            })
            .collect();
        let asks = OrderBookDiffAsks::new(asks.map_err(|e| feeds::Error::Binance(e.to_string()))?)
            .map_err(|e| feeds::Error::Binance(e.to_string()))?;
        let asks = state.asks.update(&asks);
        state.asks = asks.clone();

        state.tx.unbounded_send((bids, asks));
        Ok(())
    }
    fn order_book(config: &mut Config, book: OrderBook) -> Result<(), feeds::Error> {
        let state = config
            .subscriptions
            .get_mut(&config.depth_order_book)
            .expect("message for unsubscribed symbol")
            .order_book
            .as_mut()
            .expect("message for unsubscribed stream");

        let bids: Result<Vec<_>, f64> = book
            .bids
            .iter()
            .map(|order| {
                Ok(Order::new(
                    Price::new(order.price)?,
                    Amount::new(order.quantity)?,
                ))
            })
            .collect();
        let bids = OrderBookBids::new(bids.map_err(|e| feeds::Error::Binance(e.to_string()))?)
            .map_err(|e| feeds::Error::Binance(e.to_string()))?;

        let asks: Result<Vec<_>, f64> = book
            .asks
            .iter()
            .map(|order| {
                Ok(Order::new(
                    Price::new(order.price)?,
                    Amount::new(order.quantity)?,
                ))
            })
            .collect();
        let asks = OrderBookAsks::new(asks.map_err(|e| feeds::Error::Binance(e.to_string()))?)
            .map_err(|e| feeds::Error::Binance(e.to_string()))?;

        state.tx.unbounded_send((bids, asks));
        Ok(())
    }

    pub fn new(mut stream: WebSocketStream<MaybeTlsStream<TcpStream>>, mut config: Config) -> Self {
        let (sink, stream) = stream.split();
        let shared_config = Arc::new(Mutex::new(config.clone()));
        let stream = stream
            .err_into::<feeds::Error>()
            .try_for_each(move |message| {
                let mut config_copy = shared_config.clone();
                async move {
                    let mut config = config_copy.lock_owned().await;
                    if let Message::Text(json) = message {
                        let event = serde_json::from_str::<events::Event>(&json)
                            .map_err(|e| feeds::Error::Binance(e.to_string()))?;
                        match event {
                            events::Event::Typed(events::TypedEvent::DepthUpdate(diff)) => {
                                Self::depth_update(&mut config, diff)
                            }
                            events::Event::OrderBook(book) => Self::order_book(&mut config, book),
                        }
                    } else {
                        Ok(())
                    }
                }
            });
        Self { config }
    }
}

pub mod events;
#[cfg(test)]
mod tests;
