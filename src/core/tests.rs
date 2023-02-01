use itertools::Itertools;

use crate::core::*;

#[test]
fn invalid_price() {
    assert!(Price::new(f64::NAN).is_err_and(|v| v.is_nan()));
    assert!(Price::new(0.0).contains_err(&0.0));
    assert!(Price::new(-0.1).contains_err(&-0.1));
    assert!(Price::new(f64::INFINITY).contains_err(&f64::INFINITY));
    assert!(Price::new(f64::NEG_INFINITY).contains_err(&f64::NEG_INFINITY));
}

#[test]
fn valid_price() {
    assert!(Price::new(0.1).is_ok_and(|v| v.into_inner() == 0.1));
}

#[test]
fn compare_price() {
    assert_eq!(Price::new(0.1), Price::new(0.1));
    assert_eq!(Price::new(0.2), Price::new(0.2));
    assert_ne!(Price::new(0.2), Price::new(0.3));
    assert_ne!(Price::new(0.3), Price::new(0.2));
    assert!(Price::new(0.1) < Price::new(0.2));
    assert!(Price::new(0.2) > Price::new(0.1));
    assert!(Price::new(0.2) < Price::new(0.3));
    assert!(Price::new(0.3) > Price::new(0.2));
}

#[test]
fn invalid_amount() {
    assert!(Amount::new(f64::NAN).is_err_and(|v| v.is_nan()));
    assert!(Amount::new(-0.1).contains_err(&-0.1));
    assert!(Amount::new(f64::INFINITY).contains_err(&f64::INFINITY));
    assert!(Amount::new(f64::NEG_INFINITY).contains_err(&f64::NEG_INFINITY));
}

#[test]
fn valid_amount() {
    assert_eq!(Amount::default().into_inner(), 0.0);
    assert!(Amount::new(0.0).is_ok_and(|v| v.0 == 0.0));
    assert!(Amount::new(0.1).is_ok_and(|v| v.0 == 0.1));
}

#[test]
fn compare_amount() {
    assert_eq!(Amount::new(0.0), Amount::new(0.0));
    assert_eq!(Amount::new(0.1), Amount::new(0.1));
    assert_ne!(Amount::new(0.2), Amount::new(0.3));
    assert_ne!(Amount::new(0.3), Amount::new(0.2));
    assert!(Amount::new(0.1) < Amount::new(0.2));
    assert!(Amount::new(0.2) > Amount::new(0.1));
    assert!(Amount::new(0.2) < Amount::new(0.3));
    assert!(Amount::new(0.3) > Amount::new(0.2));
}

#[test]
fn invalid_order_book_diff_asks() {
    unsafe {
        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.2, 0.2),
        ];
        assert!(OrderBookDiffAsks::new(orders)
            .contains_err(&OrderBookError::HasOrderWithNotUniquePrice));
        let orders = vec![
            Order::new_unchecked(0.3, 0.1),
            Order::new_unchecked(0.2, 0.2),
        ];
        assert!(OrderBookDiffAsks::new_sorted(orders)
            .contains_err(&OrderBookError::OrdersNotSortedAccordingToQuoteType));
    }
}

#[test]
fn invalid_order_book_asks() {
    unsafe {
        let orders = vec![Order::new_unchecked(0.2, 0.0)];
        assert!(OrderBookAsks::new(orders).contains_err(&OrderBookError::HasOrderWithEmptyAmount));
        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.2, 0.2),
        ];
        assert!(
            OrderBookAsks::new(orders).contains_err(&OrderBookError::HasOrderWithNotUniquePrice)
        );
        let orders = vec![
            Order::new_unchecked(0.3, 0.1),
            Order::new_unchecked(0.2, 0.2),
        ];
        assert!(OrderBookAsks::new_sorted(orders)
            .contains_err(&OrderBookError::OrdersNotSortedAccordingToQuoteType));
    }
}

#[test]
fn valid_order_book_diff_asks() {
    assert!(OrderBookDiffAsks::default().0.is_empty());
    assert!(OrderBookDiffAsks::new(vec![]).is_ok_and(|asks| asks.0.is_empty()));
    unsafe {
        let orders = vec![Order::new_unchecked(0.2, 0.1)];
        assert!(OrderBookDiffAsks::new(orders.clone()).is_ok_and(|asks| asks.0 == orders));

        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.1, 0.0),
            Order::new_unchecked(0.3, 0.2),
            Order::new_unchecked(0.25, 0.3),
        ];
        let mut expected = orders.clone();
        expected.sort_by_key(|o| o.0);
        assert!(OrderBookDiffAsks::new(orders).is_ok_and(|asks| asks.0 == expected));
    }
}

#[test]
fn valid_order_book_asks() {
    assert!(OrderBookAsks::default().0 .0.is_empty());
    assert!(OrderBookAsks::new(vec![]).is_ok_and(|asks| asks.0 .0.is_empty()));
    unsafe {
        let orders = vec![Order::new_unchecked(0.2, 0.1)];
        assert!(OrderBookAsks::new(orders.clone()).is_ok_and(|asks| asks.0 .0 == orders));

        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.1, 0.2),
            Order::new_unchecked(0.3, 0.2),
            Order::new_unchecked(0.25, 0.2),
        ];
        let mut expected = orders.clone();
        expected.sort_by_key(|o| o.0);
        assert!(OrderBookAsks::new(orders).is_ok_and(|asks| asks.0 .0 == expected));
    }
}

#[test]
fn invalid_order_book_diff_bids() {
    unsafe {
        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.2, 0.2),
        ];
        assert!(OrderBookDiffBids::new(orders)
            .contains_err(&OrderBookError::HasOrderWithNotUniquePrice));
        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.3, 0.2),
        ];
        assert!(OrderBookDiffBids::new_sorted(orders)
            .contains_err(&OrderBookError::OrdersNotSortedAccordingToQuoteType));
    }
}

#[test]
fn invalid_order_book_bids() {
    unsafe {
        let orders = vec![Order::new_unchecked(0.2, 0.0)];
        assert!(OrderBookBids::new(orders).contains_err(&OrderBookError::HasOrderWithEmptyAmount));
        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.2, 0.2),
        ];
        assert!(
            OrderBookBids::new(orders).contains_err(&OrderBookError::HasOrderWithNotUniquePrice)
        );
        let orders = vec![
            Order::new_unchecked(0.2, 0.1),
            Order::new_unchecked(0.3, 0.2),
        ];
        assert!(OrderBookBids::new_sorted(orders)
            .contains_err(&OrderBookError::OrdersNotSortedAccordingToQuoteType));
    }
}

#[test]
fn valid_order_book_diff_bids() {
    assert!(OrderBookDiffBids::default().0.is_empty());
    assert!(OrderBookDiffBids::new(vec![]).is_ok_and(|bids| bids.0.is_empty()));
    unsafe {
        let orders = vec![Order::new_unchecked(0.2, 0.1)];
        assert!(OrderBookDiffBids::new(orders.clone()).is_ok_and(|bids| bids.0 == orders));

        let orders = vec![
            Order::new_unchecked(0.25, 0.3),
            Order::new_unchecked(0.3, 0.2),
            Order::new_unchecked(0.1, 0.0),
            Order::new_unchecked(0.2, 0.1),
        ];
        let mut expected = orders.clone();
        expected.sort_by(|l, r| r.0.cmp(&l.0));
        assert!(OrderBookDiffBids::new(orders).is_ok_and(|bids| bids.0 == expected));
    }
}

#[test]
fn valid_order_book_bids() {
    assert!(OrderBookBids::default().0 .0.is_empty());
    assert!(OrderBookBids::new(vec![]).is_ok_and(|bids| bids.0 .0.is_empty()));
    unsafe {
        let orders = vec![Order::new_unchecked(0.2, 0.1)];
        assert!(OrderBookBids::new(orders.clone()).is_ok_and(|bids| bids.0 .0 == orders));

        let orders = vec![
            Order::new_unchecked(0.25, 0.2),
            Order::new_unchecked(0.3, 0.2),
            Order::new_unchecked(0.1, 0.2),
            Order::new_unchecked(0.2, 0.1),
        ];
        let mut expected = orders.clone();
        expected.sort_by(|l, r| r.0.cmp(&l.0));
        assert!(OrderBookBids::new(orders).is_ok_and(|bids| bids.0 .0 == expected));
    }
}

fn default_amount<const QUOTE: bool>(orders: &OrderBookDiff<QUOTE>) -> OrderBookDiff<QUOTE> {
    OrderBookDiff(orders.0.iter().map(|o| o.empty()).collect())
}

#[test]
fn merge_bids() {
    unsafe {
        let bids = OrderBookDiffBids::default();
        let diff = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(1.5, 1.0),
            Order::new_unchecked(0.5, 2.5),
        ]);

        let bids = OrderBookDiffBids::new_unchecked(Merger::new(&bids, &diff).collect());
        assert_eq!(&bids, &diff);

        let bids = OrderBookDiffBids::new_unchecked(
            Merger::new(&bids, &OrderBookDiff::default()).collect(),
        );
        assert_eq!(&bids, &diff);

        let bids = OrderBookDiffBids::new_unchecked(Merger::new(&bids, &diff).collect());
        assert_eq!(&bids, &diff);

        let bids =
            OrderBookDiffBids::new_unchecked(Merger::new(&bids, &default_amount(&diff)).collect());
        assert_eq!(&bids, &default_amount(&diff));

        let bids = OrderBookDiffBids::new_unchecked(Merger::new(&bids, &diff).collect());
        assert_eq!(&bids, &diff);

        let diff = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.5),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.1, 1.5),
        ]);
        let expected = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.5),
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.1, 1.5),
        ]);

        let bids = OrderBookDiffBids::new_unchecked(Merger::new(&bids, &diff).collect());
        assert_eq!(&bids, &expected);

        let diff = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.0),
            Order::new_unchecked(1.9, 0.8),
            Order::new_unchecked(1.5, 0.1),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.2, 0.5),
        ]);

        let expected = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.0),
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(1.9, 0.8),
            Order::new_unchecked(1.5, 0.1),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.2, 0.5),
            Order::new_unchecked(0.1, 1.5),
        ]);
        let bids = OrderBookDiffBids::new_unchecked(Merger::new(&bids, &diff).collect());
        assert_eq!(&bids, &expected);
    }
}

#[test]
fn merge_asks() {
    unsafe {
        let asks = OrderBookDiff::default();
        let diff = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(1.5, 1.0),
            Order::new_unchecked(2.0, 1.5),
        ]);

        let asks = OrderBookDiffAsks::new_unchecked(Merger::new(&asks, &diff).collect());
        assert_eq!(&asks, &diff);

        let asks = OrderBookDiffAsks::new_unchecked(
            Merger::new(&asks, &OrderBookDiff::default()).collect(),
        );
        assert_eq!(&asks, &diff);

        let asks = OrderBookDiffAsks::new_unchecked(Merger::new(&asks, &diff).collect());
        assert_eq!(&asks, &diff);

        let asks =
            OrderBookDiffAsks::new_unchecked(Merger::new(&asks, &default_amount(&diff)).collect());
        assert_eq!(&asks, &default_amount(&diff));

        let asks = OrderBookDiffAsks::new_unchecked(Merger::new(&asks, &diff).collect());
        assert_eq!(&asks, &diff);

        let diff = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.1, 1.5),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(2.1, 0.5),
        ]);
        let expected = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.1, 1.5),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(2.1, 0.5),
        ]);

        let asks = OrderBookDiffAsks::new_unchecked(Merger::new(&asks, &diff).collect());
        assert_eq!(&asks, &expected);

        let diff = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.2, 0.5),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(1.5, 0.1),
            Order::new_unchecked(1.9, 0.8),
            Order::new_unchecked(2.1, 0.0),
        ]);

        let expected = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.1, 1.5),
            Order::new_unchecked(0.2, 0.5),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(1.5, 0.1),
            Order::new_unchecked(1.9, 0.8),
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(2.1, 0.0),
        ]);
        let asks = OrderBookDiffAsks::new_unchecked(Merger::new(&asks, &diff).collect());
        assert_eq!(&asks, &expected);
    }
}

#[test]
fn update_bids() {
    unsafe {
        let bids = OrderBookBids::default();
        let diff = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(1.5, 1.0),
            Order::new_unchecked(0.5, 2.5),
        ]);

        let bids = bids.update(&diff);
        assert_eq!(&bids.0, &diff);

        let bids = bids.update(&OrderBookDiff::default());
        assert_eq!(&bids.0, &diff);

        let bids = bids.update(&diff);
        assert_eq!(&bids.0, &diff);

        let bids = bids.update(&default_amount(&diff));
        assert!(&bids.0 .0.is_empty());

        let bids = bids.update(&diff);
        assert_eq!(&bids.0, &diff);

        let diff = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.5),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.1, 1.5),
        ]);
        let expected = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.5),
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.1, 1.5),
        ]);

        let bids = bids.update(&diff);
        assert_eq!(&bids.0, &expected);

        let diff = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.0),
            Order::new_unchecked(1.9, 0.8),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(1.0, 0.1),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.2, 0.5),
        ]);

        let expected = OrderBookDiffBids::new_unchecked(vec![
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(1.9, 0.8),
            Order::new_unchecked(1.0, 0.1),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.2, 0.5),
        ]);
        let bids = bids.update(&diff);
        assert_eq!(&bids.0, &expected);
    }
}

#[test]
fn update_asks() {
    unsafe {
        let asks = OrderBookAsks::default();
        let diff = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(1.5, 1.0),
            Order::new_unchecked(2.0, 1.5),
        ]);

        let asks = asks.update(&diff);
        assert_eq!(&asks.0, &diff);

        let asks = asks.update(&OrderBookDiff::default());
        assert_eq!(&asks.0, &diff);

        let asks = asks.update(&diff);
        assert_eq!(&asks.0, &diff);

        let asks = asks.update(&default_amount(&diff));
        assert!(&asks.0 .0.is_empty());

        let asks = asks.update(&diff);
        assert_eq!(&asks.0, &diff);

        let diff = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.1, 1.5),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(2.1, 0.5),
        ]);
        let expected = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.1, 1.5),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(1.9, 0.7),
            Order::new_unchecked(2.0, 1.5),
            Order::new_unchecked(2.1, 0.5),
        ]);

        let asks = asks.update(&diff);
        assert_eq!(&asks.0, &expected);

        let diff = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.2, 0.5),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(1.0, 0.1),
            Order::new_unchecked(1.5, 0.0),
            Order::new_unchecked(1.9, 0.8),
            Order::new_unchecked(2.1, 0.0),
        ]);

        let expected = OrderBookDiffAsks::new_unchecked(vec![
            Order::new_unchecked(0.1, 1.5),
            Order::new_unchecked(0.2, 0.5),
            Order::new_unchecked(0.3, 4.0),
            Order::new_unchecked(0.4, 4.0),
            Order::new_unchecked(0.5, 2.5),
            Order::new_unchecked(0.6, 5.0),
            Order::new_unchecked(0.7, 4.0),
            Order::new_unchecked(0.9, 4.0),
            Order::new_unchecked(1.0, 0.1),
            Order::new_unchecked(1.9, 0.8),
        ]);
        let asks = asks.update(&diff);
        assert_eq!(&asks.0, &expected);
    }
}

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
fn spread_summary() {
    let spread = SummaryOrderBook::spread(
        Vec::<SummaryOrder>::default().into_iter(),
        Vec::<SummaryOrder>::default().into_iter(),
    );
    assert!(spread.is_nan());

    unsafe {
        let bids = vec![SummaryOrder(
            Exchange::Bitstamp,
            Order::new_unchecked(2.3, 0.1),
        )];
        let asks = vec![SummaryOrder(
            Exchange::Binance,
            Order::new_unchecked(2.1, 1.1),
        )];
        let spread =
            SummaryOrderBook::spread(Vec::<SummaryOrder>::default().into_iter(), asks.into_iter());
        assert!(spread.is_infinite() && spread.is_sign_negative());

        let spread =
            SummaryOrderBook::spread(bids.into_iter(), Vec::<SummaryOrder>::default().into_iter());
        assert!(spread.is_infinite() && spread.is_sign_positive());

        let bids = vec![SummaryOrder(
            Exchange::Bitstamp,
            Order::new_unchecked(2.3, 0.1),
        )];
        let asks = vec![SummaryOrder(
            Exchange::Binance,
            Order::new_unchecked(2.1, 1.1),
        )];
        let spread = SummaryOrderBook::spread(bids.into_iter(), asks.into_iter());
        assert_feq!(spread, 0.2);

        let bids = vec![SummaryOrder(
            Exchange::Bitstamp,
            Order::new_unchecked(2.1, 0.1),
        )];
        let asks = vec![SummaryOrder(
            Exchange::Binance,
            Order::new_unchecked(2.3, 1.1),
        )];
        let spread = SummaryOrderBook::spread(bids.into_iter(), asks.into_iter());
        assert_feq!(spread, -0.2);
    }
}

#[test]
fn reset_summary() {
    let mut summary = SummaryOrderBook::default();

    unsafe {
        let bin_bids = OrderBook::new_unchecked(vec![
            Order::new_unchecked(1.1, 0.1),
            Order::new_unchecked(1.0, 0.1),
        ]);
        let bin_asks = OrderBook::new_unchecked(vec![
            Order::new_unchecked(0.8, 0.1),
            Order::new_unchecked(0.9, 0.1),
        ]);
        summary.reset(Exchange::Binance, bin_bids.clone(), bin_asks.clone());
        assert!(summary.asks().eq(bin_asks
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Binance, *o))));
        assert!(summary.bids().eq(bin_bids
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Binance, *o))));

        let bit_bids = OrderBook::new_unchecked(vec![
            Order::new_unchecked(2.1, 1.1),
            Order::new_unchecked(2.0, 1.1),
        ]);
        let bit_asks = OrderBook::new_unchecked(vec![
            Order::new_unchecked(1.8, 2.1),
            Order::new_unchecked(1.9, 2.1),
        ]);
        summary.reset(Exchange::Bitstamp, bit_bids.clone(), bit_asks.clone());
        assert!(summary.asks().eq(bin_asks
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Binance, *o))
            .chain(
                bit_asks
                    .0
                     .0
                    .iter()
                    .map(|o| SummaryOrder(Exchange::Bitstamp, *o))
            )));
        assert!(summary.bids().eq(bit_bids
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Bitstamp, *o))
            .chain(
                bin_bids
                    .0
                     .0
                    .iter()
                    .map(|o| SummaryOrder(Exchange::Binance, *o))
            )));

        let bin_bids = OrderBook::new_unchecked(vec![
            Order::new_unchecked(2.1, 0.1),
            Order::new_unchecked(2.0, 0.1),
        ]);
        let bin_asks = OrderBook::new_unchecked(vec![
            Order::new_unchecked(1.8, 0.1),
            Order::new_unchecked(1.9, 0.1),
        ]);
        summary.reset(Exchange::Binance, bin_bids.clone(), bin_asks.clone());
        assert!(summary.asks().eq(bit_asks
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Bitstamp, *o))
            .interleave(
                bin_asks
                    .0
                     .0
                    .iter()
                    .map(|o| SummaryOrder(Exchange::Binance, *o))
            )));

        assert!(summary.bids().eq(bit_bids
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Bitstamp, *o))
            .interleave(
                bin_bids
                    .0
                     .0
                    .iter()
                    .map(|o| SummaryOrder(Exchange::Binance, *o))
            )));

        summary.reset(
            Exchange::Bitstamp,
            OrderBook::default(),
            OrderBook::default(),
        );
        assert!(summary.asks().eq(bin_asks
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Binance, *o))));
        assert!(summary.bids().eq(bin_bids
            .0
             .0
            .iter()
            .map(|o| SummaryOrder(Exchange::Binance, *o))));

        summary.reset(
            Exchange::Binance,
            OrderBook::default(),
            OrderBook::default(),
        );
        assert_eq!(summary.asks().count(), 0);
        assert_eq!(summary.bids().count(), 0);

        let bin_bids = OrderBook::new_unchecked(vec![
            Order::new_unchecked(2.0, 1.1),
            Order::new_unchecked(1.9, 1.2),
            Order::new_unchecked(1.8, 1.3),
            Order::new_unchecked(1.7, 1.4),
            Order::new_unchecked(1.6, 1.5),
            Order::new_unchecked(1.5, 1.6),
            Order::new_unchecked(1.4, 1.7),
            Order::new_unchecked(1.3, 1.8),
            Order::new_unchecked(1.2, 1.9),
            Order::new_unchecked(1.1, 2.0),
        ]);
        let bin_asks = OrderBook::new_unchecked(vec![
            Order::new_unchecked(2.1, 1.1),
            Order::new_unchecked(2.2, 1.2),
            Order::new_unchecked(2.3, 1.3),
            Order::new_unchecked(2.4, 1.4),
            Order::new_unchecked(2.5, 1.5),
            Order::new_unchecked(2.6, 1.6),
            Order::new_unchecked(2.7, 1.7),
            Order::new_unchecked(2.8, 1.8),
            Order::new_unchecked(2.9, 1.9),
            Order::new_unchecked(3.0, 2.0),
        ]);
        let bit_bids = OrderBook::new_unchecked(vec![
            Order::new_unchecked(2.3, 0.1),
            Order::new_unchecked(2.2, 0.2),
            Order::new_unchecked(2.1, 0.3),
            Order::new_unchecked(2.0, 0.4),
            Order::new_unchecked(1.9, 0.5),
            Order::new_unchecked(1.8, 0.6),
            Order::new_unchecked(1.7, 0.7),
            Order::new_unchecked(1.6, 0.8),
            Order::new_unchecked(1.5, 0.9),
            Order::new_unchecked(1.4, 1.0),
        ]);
        let bit_asks = OrderBook::new_unchecked(vec![
            Order::new_unchecked(2.4, 0.1),
            Order::new_unchecked(2.5, 0.2),
            Order::new_unchecked(2.6, 0.3),
            Order::new_unchecked(2.7, 0.4),
            Order::new_unchecked(2.8, 0.5),
            Order::new_unchecked(2.9, 0.6),
            Order::new_unchecked(3.0, 0.7),
            Order::new_unchecked(3.1, 0.8),
            Order::new_unchecked(3.2, 0.9),
            Order::new_unchecked(3.3, 1.0),
        ]);
        summary.reset(Exchange::Binance, bin_bids, bin_asks);
        summary.reset(Exchange::Bitstamp, bit_bids, bit_asks);
        let bids = vec![
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(2.3, 0.1)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(2.2, 0.2)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(2.1, 0.3)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.0, 1.1)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(2.0, 0.4)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(1.9, 1.2)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(1.9, 0.5)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(1.8, 1.3)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(1.8, 0.6)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(1.7, 1.4)),
        ];
        let asks = vec![
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.1, 1.1)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.2, 1.2)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.3, 1.3)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.4, 1.4)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(2.4, 0.1)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.5, 1.5)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(2.5, 0.2)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.6, 1.6)),
            SummaryOrder(Exchange::Bitstamp, Order::new_unchecked(2.6, 0.3)),
            SummaryOrder(Exchange::Binance, Order::new_unchecked(2.7, 1.7)),
        ];
        assert!(summary.asks().eq(asks.into_iter()));
        assert!(summary.bids().eq(bids.into_iter()));
    }
}
