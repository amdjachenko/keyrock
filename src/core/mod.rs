use std::{
    cmp::{min, Ordering},
    fmt::{Debug, Display},
    iter::Peekable,
    slice::Iter,
    sync::Arc,
};

use itertools::kmerge_by;
use strum::{EnumIter, IntoEnumIterator};

/// A normal positive float representing valid price
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub struct Price(f64);

impl Debug for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

impl Eq for Price {}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Price {
    /// # Safety
    ///
    /// Behavior is undefined if value is not normal or negative
    /// Note that 0 is subnormal
    unsafe fn new_unchecked(value: f64) -> Self {
        Self(value)
    }
    pub fn new(value: f64) -> std::result::Result<Self, f64> {
        if value.is_normal() && value.is_sign_positive() {
            unsafe { Ok(Self::new_unchecked(value)) }
        } else {
            Err(value)
        }
    }
    pub fn into_inner(&self) -> f64 {
        self.0
    }
}

/// A normal positive float representing valid amount
#[derive(Default, PartialEq, PartialOrd, Copy, Clone)]
pub struct Amount(f64);

impl Debug for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Ord for Amount {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for Amount {}

impl Amount {
    /// # Safety
    ///
    /// Behavior is undefined if value is not a normal float (except 0) or negative
    unsafe fn new_unchecked(value: f64) -> Self {
        Self(value)
    }
    pub fn new(value: f64) -> std::result::Result<Self, f64> {
        match value.classify() {
            std::num::FpCategory::Nan
            | std::num::FpCategory::Infinite
            | std::num::FpCategory::Subnormal => Err(value),
            _ if value < 0.0 => Err(value),
            _ => unsafe { Ok(Self::new_unchecked(value)) },
        }
    }
    pub fn into_inner(&self) -> f64 {
        self.0
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Order(Price, Amount);

impl Order {
    unsafe fn new_unchecked(price: f64, amount: f64) -> Self {
        unsafe { Self(Price::new_unchecked(price), Amount::new_unchecked(amount)) }
    }
    pub fn new(price: Price, amount: Amount) -> Self {
        Self(price, amount)
    }
    pub fn price(&self) -> Price {
        self.0
    }
    pub fn amount(&self) -> Amount {
        self.1
    }
    pub fn is_empty(&self) -> bool {
        self.1 .0 == 0.0
    }
    pub fn empty(&self) -> Self {
        Self(self.price(), Amount::default())
    }
}

impl Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.price())
            .field(&self.amount())
            .finish()
    }
}

/// Properly sorted vector of unique possibly empty orders
#[derive(Default, Eq, PartialEq, Clone)]
pub struct OrderBookDiff<const QUOTE: bool>(Vec<Order>);

/// Properly sorted fixed size vector of unique non empty orders
/// Note that OrderBook is a valid OrderBookDiff
#[derive(Default, Eq, PartialEq, Clone)]
pub struct OrderBook<const QUOTE: bool, const COUNT: usize>(OrderBookDiff<QUOTE>);

impl<const QUOTE: bool> Debug for OrderBookDiff<QUOTE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let quote = quote_to_str::<QUOTE>();
        write!(f, "{quote} diff {:?}", &self.0)
    }
}

impl<const QUOTE: bool, const COUNT: usize> Debug for OrderBook<QUOTE, COUNT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let quote = quote_to_str::<QUOTE>();
        write!(f, "{quote} book{COUNT} {:?}", &self.0)
    }
}

struct Merger<'a, const QUOTE: bool> {
    book: Peekable<Iter<'a, Order>>,
    diff: Peekable<Iter<'a, Order>>,
}

impl<'a, const QUOTE: bool> Merger<'a, QUOTE> {
    fn new(book: &'a OrderBookDiff<QUOTE>, diff: &'a OrderBookDiff<QUOTE>) -> Self {
        Self {
            book: book.0.iter().peekable(),
            diff: diff.0.iter().peekable(),
        }
    }
}

impl<'a, const QUOTE: bool> Iterator for Merger<'a, QUOTE> {
    type Item = Order;

    fn next(&mut self) -> Option<Self::Item> {
        let which = match (self.book.peek(), self.diff.peek()) {
            (Some(b), Some(d)) => order_comparator::<QUOTE>()(b, d),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => return None,
        };

        match which {
            Ordering::Less => self.book.next().copied(),
            Ordering::Equal => {
                self.book.next();
                self.diff.next().copied()
            }
            Ordering::Greater => self.diff.next().copied(),
        }
    }
}

const fn order_comparator<const QUOTE: bool>() -> impl Fn(&Order, &Order) -> std::cmp::Ordering {
    match QUOTE {
        ASK => |l: &Order, r: &Order| l.price().cmp(&r.price()),
        BID => |l: &Order, r: &Order| r.price().cmp(&l.price()),
    }
}

const fn order_partial_comparator<const QUOTE: bool>(
) -> impl Fn(&Order, &Order) -> Option<std::cmp::Ordering> {
    match QUOTE {
        ASK => |l: &Order, r: &Order| l.price().partial_cmp(&r.price()),
        BID => |l: &Order, r: &Order| r.price().partial_cmp(&l.price()),
    }
}

const fn quote_to_str<const QUOTE: bool>() -> &'static str {
    match QUOTE {
        ASK => "ask",
        BID => "bid",
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum OrderBookError {
    /// We are not allowed neither merge nor peek one. Something wrong with the feed data
    HasOrderWithNotUniquePrice,
    /// Likely diff was used instead of snapshot for initialization by mistake
    HasOrderWithEmptyAmount,
    /// Likely asks used instead of bids or the other way around by mistake
    OrdersNotSortedAccordingToQuoteType,
}

impl Display for OrderBookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OrderBookError::HasOrderWithNotUniquePrice => {
                "order book has multiple orders with the same price"
            }
            OrderBookError::HasOrderWithEmptyAmount => "order book has order with 0 amount",
            OrderBookError::OrdersNotSortedAccordingToQuoteType => {
                "order book is not properly sorted"
            }
        };
        f.write_str(str)
    }
}

impl<const QUOTE: bool, const COUNT: usize> OrderBook<QUOTE, COUNT> {
    /// # Safety
    ///
    /// Behavior is undefined if orders are not unique or empty or not sorted according to QUOTE
    unsafe fn new_unchecked(mut orders: Vec<Order>) -> Self {
        orders.truncate(COUNT);
        Self(OrderBookDiff::new_unchecked(orders))
    }
    /// # Safety
    ///
    /// Behavior is undefined if orders not sorted according to QUOTE
    unsafe fn new_sorted_unchecked(
        orders: Vec<Order>,
    ) -> std::result::Result<Self, OrderBookError> {
        // Checking all orders isn't more reliable than checking first COUNT
        // because diff may have no empty orders. On the other hand
        // checking it in runtime sooner or later will return an error
        // if diff was used instead of snapshot for initialization by mistake
        let (is_unique, is_empty) = orders
            .split_first()
            .map(|(first, rest)| {
                let (unique, empty, _) = rest.iter().take(COUNT).fold(
                    (true, first.is_empty(), first.price()),
                    |(unique, empty, price), order| {
                        (
                            unique && price != order.price(),
                            empty || order.is_empty(),
                            order.price(),
                        )
                    },
                );
                (unique, empty)
            })
            .unwrap_or((true, false));

        if is_empty {
            Err(OrderBookError::HasOrderWithEmptyAmount)
        } else if !is_unique {
            Err(OrderBookError::HasOrderWithNotUniquePrice)
        } else {
            unsafe { Ok(Self::new_unchecked(orders)) }
        }
    }
    pub fn new_sorted(orders: Vec<Order>) -> std::result::Result<Self, OrderBookError> {
        if !orders[0..min(orders.len(), COUNT)].is_sorted_by(order_partial_comparator::<QUOTE>()) {
            return Err(OrderBookError::OrdersNotSortedAccordingToQuoteType);
        }
        unsafe { Self::new_sorted_unchecked(orders) }
    }
    pub fn new(mut orders: Vec<Order>) -> std::result::Result<Self, OrderBookError> {
        if orders.is_empty() {
            return Ok(Self::default());
        }

        let index = min(orders.len(), COUNT) - 1;
        orders
            .select_nth_unstable_by(index, order_comparator::<QUOTE>())
            .0
            .sort_unstable_by(order_comparator::<QUOTE>());
        unsafe { Self::new_sorted_unchecked(orders) }
    }
    pub fn update(&self, diff: &OrderBookDiff<QUOTE>) -> OrderBook<QUOTE, COUNT> {
        let mut book = Vec::with_capacity(COUNT);
        Merger::new(&self.0, &diff)
            .filter(|order| !order.is_empty())
            .take(COUNT)
            .collect_into(&mut book);
        Self(OrderBookDiff::<QUOTE>(book))
    }
}

impl<const QUOTE: bool> OrderBookDiff<QUOTE> {
    /// # Safety
    ///
    /// Behavior is undefined if orders are not unique or not sorted according to QUOTE
    unsafe fn new_unchecked(orders: Vec<Order>) -> Self {
        Self(orders)
    }
    /// # Safety
    ///
    /// Behavior is undefined if orders not sorted according to QUOTE
    unsafe fn new_sorted_unchecked(
        orders: Vec<Order>,
    ) -> std::result::Result<Self, OrderBookError> {
        let is_unique = orders
            .split_first()
            .map(|(first, rest)| {
                rest.iter()
                    .fold((true, first.price()), |(unique, price), order| {
                        (unique && price != order.price(), order.price())
                    })
                    .0
            })
            .unwrap_or(true);

        if is_unique {
            Ok(Self(orders))
        } else {
            Err(OrderBookError::HasOrderWithNotUniquePrice)
        }
    }
    pub fn new_sorted(orders: Vec<Order>) -> std::result::Result<Self, OrderBookError> {
        if !orders.is_sorted_by(order_partial_comparator::<QUOTE>()) {
            Err(OrderBookError::OrdersNotSortedAccordingToQuoteType)
        } else {
            unsafe { Self::new_sorted_unchecked(orders) }
        }
    }
    pub fn new(mut orders: Vec<Order>) -> std::result::Result<Self, OrderBookError> {
        orders.sort_unstable_by(order_comparator::<QUOTE>());
        unsafe { Self::new_sorted_unchecked(orders) }
    }
}

const ASK: bool = false;
const BID: bool = true;
const BEST_ORDER_BOOK_SIZE: usize = 10;

pub type OrderBookDiffAsks = OrderBookDiff<ASK>;
pub type OrderBookDiffBids = OrderBookDiff<BID>;

pub type OrderBookAsks = OrderBook<ASK, BEST_ORDER_BOOK_SIZE>;
pub type OrderBookBids = OrderBook<BID, BEST_ORDER_BOOK_SIZE>;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, EnumIter)]
pub enum Exchange {
    Binance,
    Bitstamp,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SummaryOrder(Exchange, Order);

impl SummaryOrder {
    pub fn exchange(&self) -> Exchange {
        self.0
    }
    pub fn order(&self) -> Order {
        self.1
    }
}

impl Debug for SummaryOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.exchange())
            .field(&self.order().price())
            .field(&self.order().amount())
            .finish()
    }
}

pub struct SummaryOrderBook {
    books: Vec<(Exchange, OrderBookBids, OrderBookAsks)>,
}

impl Default for SummaryOrderBook {
    fn default() -> Self {
        let books = Exchange::iter()
            .map(|exchange| (exchange, OrderBookBids::default(), OrderBookAsks::default()))
            .collect();
        Self { books }
    }
}

impl SummaryOrderBook {
    fn quotes<const QUOTE: bool>(&self) -> impl Iterator<Item = SummaryOrder> + '_ {
        kmerge_by(
            self.books.iter().map(|books| {
                let (exchange, bids, asks) = books;
                match QUOTE {
                    ASK => &asks.0 .0,
                    BID => &bids.0 .0,
                }
                .iter()
                .copied()
                .map(|order| SummaryOrder(*exchange, order))
            }),
            match QUOTE {
                ASK => |l: &SummaryOrder, r: &SummaryOrder| match l
                    .order()
                    .price()
                    .cmp(&r.order().price())
                {
                    Ordering::Less => true,
                    Ordering::Equal => l.order().amount() > r.order().amount(),
                    Ordering::Greater => false,
                },
                BID => |l: &SummaryOrder, r: &SummaryOrder| match r
                    .order()
                    .price()
                    .cmp(&l.order().price())
                {
                    Ordering::Less => true,
                    Ordering::Equal => l.order().amount() > r.order().amount(),
                    Ordering::Greater => false,
                },
            },
        )
        .take(BEST_ORDER_BOOK_SIZE)
    }
    /*fn new(spread: Price, bids: SummaryBookBestBids, asks: SummaryBookBestAsks) -> Self {
        Self { spread, bids, asks }
    }*/
    /// -INF == no bids
    /// +INF == no asks
    ///  NAN == neither asks nor bids
    /// else == difference between best aks and best bid
    /// note that it can be negative
    pub fn spread<I: Iterator<Item = SummaryOrder>>(mut bids: I, mut asks: I) -> f64 {
        match (bids.next(), asks.next()) {
            (None, None) => f64::NAN,
            (Some(_), None) => f64::INFINITY,
            (None, Some(_)) => f64::NEG_INFINITY,
            (Some(bid), Some(ask)) => bid.1.price().into_inner() - ask.1.price().into_inner(),
        }
    }
    /// returns up to BEST_ORDER_BOOK_SIZE best asks
    pub fn asks(&self) -> impl Iterator<Item = SummaryOrder> + '_ {
        self.quotes::<ASK>()
    }
    /// returns up to BEST_ORDER_BOOK_SIZE best bids
    pub fn bids(&self) -> impl Iterator<Item = SummaryOrder> + '_ {
        self.quotes::<BID>()
    }
    /// resets order books for specified exchange only
    pub fn reset(&mut self, exchange: Exchange, bids: OrderBookBids, asks: OrderBookAsks) {
        self.books[exchange as usize] = (exchange, bids, asks);
    }
}

#[cfg(test)]
mod tests;
