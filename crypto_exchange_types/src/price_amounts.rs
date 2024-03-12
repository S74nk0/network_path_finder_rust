use std::cmp::Ordering;

/// Currency amount is for indicating a
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub struct CurrencyAmount(pub f64);

/// Price amount for a given BUY/SELL side
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub struct PriceAmount(pub f64);

/// Bid/BUY price is the highest price amount a BUYER is willing to PAY (GREEN).
/// Higler bid price is first served.
#[derive(PartialEq, Copy, Clone)]
pub struct BidPrice(pub f64);

/// Ask/SELL price is the lowest price amount a SELLER is willing a SELL (RED).
/// Lower ask price is first served.
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub struct AskPrice(pub f64);

impl PartialOrd for BidPrice {
    fn partial_cmp(&self, other: &BidPrice) -> Option<Ordering> {
        // if let Some(ord) = self.0.partial_cmp(&other.0) {
        //     Some(ord.reverse())
        // } else {
        //     None
        // }
        self.0.partial_cmp(&other.0).map(Ordering::reverse)
    }
}

// impl PartialOrd for AskPrice {}

// TODO bid and ask have a spread and this indicates the assets liquidity

#[derive(PartialEq, Copy, Clone)]
pub struct TPriceAmount<T: PartialEq + PartialOrd> {
    pub price: T,
    pub amount: PriceAmount,
    // pub timestamp: Option<f64>,
}

// impl<T: PartialEq + PartialOrd> TPriceAmount<T> {
//     pub fn new(price: T, amount: PriceAmount) -> TPriceAmount<T> {
//         TPriceAmount { price, amount }
//     }
// }

impl<T> PartialOrd for TPriceAmount<T>
where
    T: PartialEq + PartialOrd,
{
    fn partial_cmp(&self, other: &TPriceAmount<T>) -> Option<Ordering> {
        self.price.partial_cmp(&other.price)
    }
}

// // Editing

// func FindAndUpdateOrAppend(prices *[]PriceAmount, u PriceAmount, isBid bool) {
//     for i := range *prices {
//         if (*prices)[i].Price == u.Price {
//             // found update and exit
//             (*prices)[i] = u
//             return
//         }
//     }
//     // if we get here we insert
//     *prices = append(*prices, u)
//     if isBid {
//         SortBids(*prices)
//     } else {
//         SortAsks(*prices)
//     }
// }

// // should keep order
// func FindAndRemovePrice(prices *[]PriceAmount, removePrice float64) {
//     for i := range *prices {
//         if (*prices)[i].Price == removePrice {
//             *prices = append((*prices)[:i], (*prices)[i+1:]...)
//             return
//         }
//     }
// }

/// Bid/BUY price is the highest price amount a BUYER is willing to PAY (GREEN).
/// Higler bid price is first served. Prices are ordered descending order e.g. [6.5, 6.4, 5.2]
/// The price is paired with an amount
pub type BidPriceAmount = TPriceAmount<BidPrice>;

/// Ask/SELL price is the lowest price amount a SELLER is willing a SELL (RED).
/// Lower ask price is first served. Prices are ordered ascending order e.g. [5.2, 6.4, 6.5]
/// The price is paired with an amount
pub type AskPriceAmount = TPriceAmount<AskPrice>;

/// The spread is the price in between the asks and bids.
/// On Cryptowatch if you look at the graph first are the asks and follow the bids
/// Asks direction goes up (RED PART) and values increase just like the array
/// Bids direction goes down (GREEN PART) and values decrease just like the array
#[allow(unused_variables)]
struct _SpreadTypeNotImplementedYet {}

// TODO add the operations.go variable types here.
// TODO also add variable exchange data like transaction fees, transfer fees, transfer whitelist and transfer blacklist tokens
