use crate::id_types::*;
use crate::price_amounts::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// ExchangeMarketOrderBook is the market currency pair / market orderbook for a given exchange
#[derive(Clone)]
pub struct ExchangeMarketOrderBook {
    pub exchange: ExchangeID,
    pub pair: CurrencyIDPair,
    pub bids: Vec<BidPriceAmount>,
    pub asks: Vec<AskPriceAmount>,
    // TODO add last updated timestamp
}

impl ExchangeMarketOrderBook {
    pub fn new(exchange: ExchangeID, pair: CurrencyIDPair) -> Self {
        ExchangeMarketOrderBook {
            exchange: exchange,
            pair: pair,
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn sort_orderbook(&mut self) {
        self.sort_asks();
        self.sort_bids();
    }

    pub fn sort_bids(&mut self) {
        self.bids.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    }

    pub fn sort_asks(&mut self) {
        self.asks.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    }
}

/// This is the ported code from Go and this could be flattened to a ExchangeMarketKey
struct _VOID_PROTED_GO_CODE {}
// type ExchangeOrderBooks = HashMap<CurrencyIDPair, OrderBook>;
// // TODO look at cross-beam maybe you can replace this
// type ArcExchangeOrderBooks = Arc<Mutex<ExchangeOrderBooks>>;
// pub struct ExchangesOrderBooks(pub HashMap<ExchangeID, ArcExchangeOrderBooks>);

// impl ExchangesOrderBooks {
//     pub fn new_arc_exchange_order_books(&mut self, exchange: ExchangeID) -> ArcExchangeOrderBooks {
//         let exchange_order_books = Arc::new(Mutex::new(HashMap::new()));
//         self.0.insert(exchange, Arc::clone(&exchange_order_books));
//         exchange_order_books
//     }

//     pub fn remove_arc_exchange_order_books(
//         &mut self,
//         exchange: ExchangeID,
//     ) -> ArcExchangeOrderBooks {
//         let exchange_order_books = Arc::new(Mutex::new(HashMap::new()));
//         self.0.insert(exchange, Arc::clone(&exchange_order_books));
//         exchange_order_books
//     }

//     pub fn get_exchange_currency_orderbook(
//         &self,
//         exchange: &ExchangeID,
//         c1: CurrencyID,
//         c2: CurrencyID,
//     ) -> Option<(OrderBook, CurrencyIDPair)> {
//         if let Some(exchange_orderbooks) = self.0.get(exchange) {
//             let pair1 = CurrencyIDPair::new(c1, c2);
//             let pair2 = CurrencyIDPair::new(c2, c1);
//             let guard = match exchange_orderbooks.lock() {
//                 Ok(guard) => guard,
//                 Err(poisoned) => poisoned.into_inner(),
//             };
//             // TODO Clone for now. This is what you did in Go
//             if let Some(pair1_book) = guard.get(&pair1) {
//                 return Some((pair1_book.clone(), pair1));
//             }
//             if let Some(pair2_book) = guard.get(&pair2) {
//                 return Some((pair2_book.clone(), pair2));
//             }
//         }
//         None
//     }
// }

/// The ExchangeMarketKey is the flattened inverse aware key for an exchange markat/currency pair.
/// We could have a situation where different exchanges have these currencies sorted differently.
/// Lets assume that the 'c1-c2' on e1 can be corellated on e2 with 'c2-c1' where the keys are inverse.
/// This also makes it possible to easily look for an exchange market type without going into specifics
/// of the exchange market transaction type (BUY/SELL), we get the actual currency pair from the orderbook   
pub struct ExchangeMarketKey {
    pub exchange: ExchangeID,
    pub sorted_currency_pair_less: CurrencyID,
    pub sorted_currency_pair_greater: CurrencyID,
}

impl ExchangeMarketKey {
    #[inline]
    pub fn create(exchange: ExchangeID, c1: CurrencyID, c2: CurrencyID) -> Self {
        if c1.0 < c2.0 {
            ExchangeMarketKey {
                exchange: exchange,
                sorted_currency_pair_less: c1,
                sorted_currency_pair_greater: c2,
            }
        } else if c1.0 > c2.0 {
            ExchangeMarketKey {
                exchange: exchange,
                sorted_currency_pair_less: c2,
                sorted_currency_pair_greater: c1,
            }
        } else {
            panic!("Invalid ExchangeMarketKey construction!!! Problem c1 == c2!");
        }
    }
    #[inline]
    pub fn create_from_pair(exchange: ExchangeID, pair: CurrencyIDPair) -> Self {
        Self::create(exchange, pair.first, pair.second)
    }
}

pub type ArcExchangeMarketOrderBook = Arc<Mutex<ExchangeMarketOrderBook>>;
pub struct ExchangeMarketsOrderbooks(pub HashMap<ExchangeMarketKey, ArcExchangeMarketOrderBook>);
pub type ArcExchangeMarketsOrderbooks = Arc<Mutex<ExchangeMarketsOrderbooks>>;
