use crate::id_types::*;
use crate::order_book::*;
use crate::price_amounts::*;
use std::cmp::Ordering;

#[test]
fn bid_price_partial_ord_test() {
    let first_in_line_to_buy = BidPrice(10.0);
    let second_in_line_to_buy = BidPrice(8.9);
    // first to buy is less because less (index) comes first
    // and also the seller preffers to sell at a higher price
    assert_eq!(
        first_in_line_to_buy.partial_cmp(&second_in_line_to_buy),
        Some(Ordering::Less)
    );
}

#[test]
fn ask_price_partial_ord_test() {
    let first_in_line_to_sell = AskPrice(8.9);
    let second_in_line_to_sell = AskPrice(10.0);
    // first to sell is less because less (index) comes first
    // and also the buyer preffers to buy at a lesser price
    assert_eq!(
        first_in_line_to_sell.partial_cmp(&second_in_line_to_sell),
        Some(Ordering::Less)
    );
}

// #[test]
// fn order_book_sort_orderbook_test() {
//     let mut ob = OrderBook::new(CurrencyIDPair::new(CurrencyID(0), CurrencyID(1)));
//     ob.bids
//         .push(BidPriceAmount::new(BidPrice(2.0), PriceAmount(1.0)));
//     ob.bids
//         .push(BidPriceAmount::new(BidPrice(10.0), PriceAmount(1.0)));
//     ob.bids
//         .push(BidPriceAmount::new(BidPrice(5.0), PriceAmount(1.0)));
//     ob.asks
//         .push(AskPriceAmount::new(AskPrice(21.0), PriceAmount(1.0)));
//     ob.asks
//         .push(AskPriceAmount::new(AskPrice(11.0), PriceAmount(1.0)));
//     ob.asks
//         .push(AskPriceAmount::new(AskPrice(15.0), PriceAmount(1.0)));
//     ob.sort_orderbook();
//     let sorted_bids = [10.0, 5.0, 2.0];
//     let sorted_asks = [11.0, 15.0, 21.0];
//     let all_equal = ob
//         .asks
//         .iter()
//         .zip(sorted_asks.iter())
//         .all(|(a, b)| a.price.0 == *b);
//     assert_eq!(all_equal, true);
//     let all_equal = ob
//         .bids
//         .iter()
//         .zip(sorted_bids.iter())
//         .all(|(a, b)| a.price.0 == *b);
//     assert_eq!(all_equal, true);
// }
