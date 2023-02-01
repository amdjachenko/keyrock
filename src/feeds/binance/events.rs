use serde::Deserialize;

fn float_as_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Order {
    #[serde(deserialize_with = "float_as_string")]
    pub price: f64,
    #[serde(deserialize_with = "float_as_string")]
    pub quantity: f64,
}

#[cfg(test)]
macro_rules! assert_feq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if (*left_val - *right_val).abs() > 1E-15 {
                    panic!("left: {} not equal to right: {}", &*left_val, &*right_val);
                }
            }
        }
    };
    ($left:expr, $right:expr, $delta:expr $(,)?) => {
        match (&$left, &$right, &$delta) {
            (left_val, right_val, delta_val) => {
                if (*left_val - *right_val).abs() > *delta_val {
                    panic!(
                        "left: {} not equal to right: {} with precision: {}",
                        &*left_val, &*right_val, &*delta_val
                    );
                }
            }
        }
    };
}

#[test]
fn order() {
    let json = r#"["0.1", "0.2"]"#;
    let order: Order = serde_json::from_str(json).unwrap();
    assert_feq!(order.price, 0.1);
    assert_feq!(order.quantity, 0.2);
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderBookDiff {
    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "U")]
    pub first_update_id: u64,

    #[serde(rename = "u")]
    pub final_update_id: u64,

    #[serde(rename = "b")]
    pub bids: Vec<Order>,

    #[serde(rename = "a")]
    pub asks: Vec<Order>,
}

#[test]
fn order_book_update() {
    let json = r#"
    {
        "E": 123456789,
        "s": "BNBBTC",
        "U": 157,
        "u": 160,
        "b": [
          [
            "0.0024",
            "10"
          ]
        ],
        "a": [
          [
            "0.0026",
            "100"
          ]
        ]
    }
    "#;
    let update: OrderBookDiff = serde_json::from_str(json).unwrap();
    assert_eq!(update.event_time, 123456789);
    assert_eq!(update.symbol, "BNBBTC");
    assert_eq!(update.first_update_id, 157);
    assert_eq!(update.final_update_id, 160);
}

#[derive(Deserialize, Debug)]
#[serde(tag = "e", rename_all = "camelCase")]
pub enum TypedEvent {
    DepthUpdate(OrderBookDiff),
}
pub use TypedEvent::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: u64,
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

#[test]
fn order_book() {
    let json =
        r#"{ "lastUpdateId" : 160, "bids": [["0.0024", "10"]], "asks": [["0.0026", "100.1"]] }"#;
    let book: OrderBook = serde_json::from_str(json).unwrap();
    assert_eq!(book.last_update_id, 160);
    assert_feq!(book.bids[0].price, 0.0024);
    assert_feq!(book.bids[0].quantity, 10.0);
    assert_feq!(book.asks[0].price, 0.0026);
    assert_feq!(book.asks[0].quantity, 100.1);
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Event {
    Typed(TypedEvent),
    OrderBook(OrderBook),
}

pub use Event::*;

#[test]
fn event() {
    let json =
        r#"{ "lastUpdateId" : 160, "bids": [["0.0024", "10"]], "asks": [["0.0026", "100.1"]] }"#;
    let event: Event = serde_json::from_str(json).unwrap();
    assert!(matches!(event, OrderBook(_)));

    let json = r#"
        {
            "e": "depthUpdate",
            "E": 123456789,
            "s": "BNBBTC",
            "U": 157,
            "u": 160,
            "b": [
              [
                "0.0024",
                "10"
              ]
            ],
            "a": [
              [
                "0.0026",
                "100"
              ]
            ]
        }
        "#;
    let event: Event = serde_json::from_str(json).unwrap();
    assert!(matches!(event, Typed(DepthUpdate(_))));

    let json = r#"{"code": 0, "msg": "Unknown property","id": %s}"#;
    let result: serde_json::error::Result<Event> = serde_json::from_str(json);
    assert!(matches!(result, Err(_)));
}
