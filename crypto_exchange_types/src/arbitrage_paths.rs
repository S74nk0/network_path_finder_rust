
use crate::{lexicon::*, id_types::*};
use serde::{Deserialize, Serialize};
use std::{collections::{HashSet, LinkedList, BTreeMap}, convert::TryInto};

// target from go is the BalanceExchangeCurrencyInfo 'static' Info type
// from this target you search out the paths
pub type Target = BalanceExchangeCurrencyInfo;
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize, Clone)]
pub struct ArbitragePath(pub LinkedList<ExchangeOperationType>);
pub type TargetKnownPaths = HashSet<ArbitragePath>;

// add optimized paths

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct ArbitragePath7Nodes(pub [ExchangeOperationType; 7]);

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct ArbitragePath11Nodes(pub [ExchangeOperationType; 11]);

struct SingleExchangeTransactionOnlyNPairsPath<const N: usize>([CurrencyIDPair; N]);

// TODO this interpolate is used to get next
fn interpolate_next_nodes(exchange: ExchangeID, c: CurrencyID, pair: CurrencyIDPair) -> (TransactionExchangeCurrenciesBuySellInfo, BalanceExchangeCurrencyInfo) {
    // TODO maybe return error
    let (new_balance_currency, side) = pair.next_currency_and_side(c).expect("interpolate_next_nodes currency is bid and sell are equal FATAL ERROR");
    let tx = TransactionExchangeCurrenciesBuySellInfo {
        currency_from: c,
        currency_to: new_balance_currency,
        exchange: exchange,
        side: side
    };
    let b = BalanceExchangeCurrencyInfo {currency: new_balance_currency, exchange: exchange};
    (tx, b)
}

fn tx_to_currency_id_pairs(tx: &TransactionExchangeCurrenciesBuySellInfo) -> CurrencyIDPair {
    let (first, second) = match tx.side {
        TransactionSide::BUY => (tx.currency_from, tx.currency_to),
        TransactionSide::SELL => (tx.currency_to, tx.currency_from),
    };
    CurrencyIDPair::new(first, second)
}

impl<const N: usize> SingleExchangeTransactionOnlyNPairsPath<N> {
    #[inline(always)]
    fn get_start_end_currency(arr: &[CurrencyIDPair; N]) -> Option<CurrencyID> {
        let first_pair = arr.first()?;
        let last_pair = arr.last()?;
        let (first_c1, first_c2) = (first_pair.first, first_pair.second);
        let (last_c1, last_c2) = (last_pair.first, last_pair.second);
        if first_c1.eq(&last_c1) || first_c1.eq(&last_c2) {
            Some(first_c1)
        } else if first_c2.eq(&last_c1) || first_c2.eq(&last_c2) {
            Some(first_c2)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn is_arbitrage_path_reversed(lhs: &[CurrencyIDPair; N], rhs: &[CurrencyIDPair; N]) -> bool {
        let mut zipped = lhs.iter().zip(rhs.iter().rev());
        zipped.all(|(a,b)| a.eq(&b))
    }
}

pub trait SingleExchangeTransactionOnlyPath {
    fn get_start_end_currency(&self) -> Option<CurrencyID>;
    fn is_arbitrage_path_reversed(&self, rhs: &Self) -> bool;
}

pub fn merge_reversed_paths<T: SingleExchangeTransactionOnlyPath>(vs: Vec<T>) -> Vec<T> {
    let mut ret_paths = Vec::new();
    let mut tx_only_3pairs_path_drain: Vec<T> = vs;
    let size = tx_only_3pairs_path_drain.len();
    for _ in 0..size {
        let poped = tx_only_3pairs_path_drain.pop();
        if let Some(poped) = poped {
            // retain those that aren't duplicates
            tx_only_3pairs_path_drain.retain(|other| poped.is_arbitrage_path_reversed(&other) == false);
            // TODO push the poped one back
            ret_paths.push(poped);
        } else {
            break;
        }
    }
    ret_paths
}

pub fn interpolate_reversed_paths(exchange: ExchangeID, c: CurrencyID, pairs: &[CurrencyIDPair]) -> (ArbitragePath, ArbitragePath) {
    let start = BalanceExchangeCurrencyInfo {currency: c, exchange: exchange};
    let mut first: LinkedList<ExchangeOperationType> = LinkedList::default();
    let mut start_c = c;
    first.push_back(start.into());
    for pair in pairs.iter() {
        let (tx, b) = interpolate_next_nodes(exchange, start_c, pair.clone());
        start_c = b.currency;
        first.push_back(tx.into());
        first.push_back(b.into());
    }
    let first = ArbitragePath(first);

    let start = BalanceExchangeCurrencyInfo {currency: c, exchange: exchange};
    let mut second: LinkedList<ExchangeOperationType> = LinkedList::default();
    let mut start_c = c;
    second.push_back(start.into());
    for pair in pairs.iter().rev() {
        let (tx, b) = interpolate_next_nodes(exchange, start_c, pair.clone());
        start_c = b.currency;
        second.push_back(tx.into());
        second.push_back(b.into());
    }
    let second = ArbitragePath(second);

    (first, second)
}


/// Known Exchange transactions only paths can be deducted from currency pairs in order.
/// This can be used to calculate 2 paths since we can start from the begining and the end.
/// Exchange id is saved outside. CurrencyIDPair can tell us what is the side.
/// The first start end currency can be also deducted from the shared first and last pair
/// but we should probably keep this known start-end currency outside. TODO think about this.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct SingleExchangeTransactionOnly3PairsPath(pub [CurrencyIDPair; 3]);

impl SingleExchangeTransactionOnlyPath for SingleExchangeTransactionOnly3PairsPath {
    #[inline(always)]
    fn get_start_end_currency(&self) -> Option<CurrencyID> {
        SingleExchangeTransactionOnlyNPairsPath::get_start_end_currency(&self.0)
    }
    #[inline(always)]
    fn is_arbitrage_path_reversed(&self, rhs: &Self) -> bool {
        SingleExchangeTransactionOnlyNPairsPath::is_arbitrage_path_reversed(&self.0, &rhs.0)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct SingleExchangeTransactionOnly5PairsPath(pub [CurrencyIDPair; 5]);

impl SingleExchangeTransactionOnlyPath for SingleExchangeTransactionOnly5PairsPath {
    #[inline(always)]
    fn get_start_end_currency(&self) -> Option<CurrencyID> {
        SingleExchangeTransactionOnlyNPairsPath::get_start_end_currency(&self.0)
    }
    #[inline(always)]
    fn is_arbitrage_path_reversed(&self, rhs: &Self) -> bool {
        SingleExchangeTransactionOnlyNPairsPath::is_arbitrage_path_reversed(&self.0, &rhs.0)
    }
}

impl TryInto<ArbitragePath7Nodes> for ArbitragePath {
    type Error = ArbitragePath;

    fn try_into(self) -> Result<ArbitragePath7Nodes, Self::Error> {
        let len = self.0.len();
        if len == 7 && self.is_path_with_transfer() {
            let v: Vec<_> = self.0.into_iter().collect();
            let ret: [ExchangeOperationType; 7] = [
                v[0],
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
                v[6],
            ];
            Ok(ArbitragePath7Nodes(ret))
        } else {
            Err(self)
        }
    }
}

impl TryInto<ArbitragePath11Nodes> for ArbitragePath {
    type Error = ArbitragePath;

    fn try_into(self) -> Result<ArbitragePath11Nodes, Self::Error> {
        let len = self.0.len();
        if len == 11 && self.is_path_with_transfer() {
            let v: Vec<_> = self.0.into_iter().collect();
            let ret: [ExchangeOperationType; 11] = [
                v[0],
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
                v[6],
                v[7],
                v[8],
                v[9],
                v[10],
            ];
            Ok(ArbitragePath11Nodes(ret))
        } else {
            Err(self)
        }
    }
}

impl TryInto<SingleExchangeTransactionOnly3PairsPath> for ArbitragePath {
    type Error = ArbitragePath;

    fn try_into(self) -> Result<SingleExchangeTransactionOnly3PairsPath, Self::Error> {
        let len = self.0.len();
        if len == 7 && self.is_same_exchange_path() {
            let v: Vec<_> = self.0.into_iter().flat_map(|op| if let ExchangeOperationType::Transaction(tx) = op {
                Some(tx_to_currency_id_pairs(&tx))
            } else {
                None
            }).collect();
            let ret: [CurrencyIDPair; 3] = [
                v[0],
                v[1],
                v[2],
            ];
            
            Ok(SingleExchangeTransactionOnly3PairsPath(ret))
        } else {
            Err(self)
        }
    }
}

impl TryInto<SingleExchangeTransactionOnly5PairsPath> for ArbitragePath {
    type Error = ArbitragePath;

    fn try_into(self) -> Result<SingleExchangeTransactionOnly5PairsPath, Self::Error> {
        let len = self.0.len();
        if len == 11 && self.is_same_exchange_path() {
            let v: Vec<_> = self.0.into_iter().flat_map(|op| if let ExchangeOperationType::Transaction(tx) = op {
                Some(tx_to_currency_id_pairs(&tx))
            } else {
                None
            }).collect();
            let ret: [CurrencyIDPair; 5] = [
                v[0],
                v[1],
                v[2],
                v[3],
                v[4],
            ];
            
            Ok(SingleExchangeTransactionOnly5PairsPath(ret))
        } else {
            Err(self)
        }
    }
}

impl ArbitragePath {
    pub fn new() -> Self {
        ArbitragePath(LinkedList::new())
    }

    pub fn is_path_with_transfer(&self) -> bool {
        self.0.iter().any(|op| {
            matches!(op, ExchangeOperationType::Transfer(_))
        })
    }

    pub fn is_same_exchange_path(&self) -> bool {
        let mut iter = self.0.iter().map(|op| {
            match op {
                ExchangeOperationType::Balance(b) => (true, b.exchange),
                ExchangeOperationType::Transaction(tx) => (true, tx.exchange),
                ExchangeOperationType::Transfer(tr) => (false, tr.withdraw_exchange),
            }
        });
        let first = iter.next();
        let first = first.map_or(None, |(ok, exchange)| if ok {Some(exchange)} else {None});
        if let Some(exchange) = first {
            let any_not_ok = iter.any(|(ok, cmp_ex)| !ok || cmp_ex.ne(&exchange) );
            !any_not_ok
        } else {
            false
        }
    }

    pub fn is_arbitrage_path_reversed(&self, cmp: &Self) -> bool {
        let mut zipped = self.0.iter().zip(cmp.0.iter().rev());
        zipped.all(|(a,b)| {
            use ExchangeOperationType::*;
            match (a,b) {
                (Balance(a), Balance(b)) => {
                    a.eq(b)
                },
                _ => ExchangeOperationType::is_inverse(a, b),
            }
        })
    }
}


// inner mod just for the print helpers
mod print_path {
    use super::*;
    #[derive(Serialize, Deserialize)]
    struct ExchangeIDCurrencyIDPairOptString {
        pub exchange: String,
        pub currency: String,
    }

    #[derive(Serialize, Deserialize)]
    enum ExchangeOperationTypeStr {
        Balance(ExchangeIDCurrencyIDPairOptString),
        Transaction(TransactionOptString),
        Transfer(TransferOptString),
    }

    #[derive(Serialize, Deserialize)]
    struct TransactionOptString {
        pub exchange: String,
        pub side: TransactionSide,
        pub currency_from: String,
        pub currency_to: String,
    }

    #[derive(Serialize, Deserialize)]
    struct TransferOptString {
        pub withdraw_exchange: String,
        pub deposit_exchange: String,
        pub currency: String,
    }

    pub fn string_id<'a, I>(it: I) -> String where I: Iterator<Item = & 'a ExchangeOperationType> {
        let strs: Vec<_> = it.map(|p| format!("{}", p)).collect();
        strs.join("-")
    }
    
    pub fn to_named_exchanges_currency_json<'a, I>(
        it: I,
        lexicon: &CryptoExchangeLexicon,
    ) -> Result<String, String> where I: Iterator<Item = & 'a ExchangeOperationType> {
        let path: Vec<_> = it
            .map(|op| match op {
                ExchangeOperationType::Balance(v) => {
                    ExchangeOperationTypeStr::Balance(ExchangeIDCurrencyIDPairOptString {
                        exchange: lexicon.exchange_to_string(&v.exchange).to_owned(),
                        currency: lexicon.currency_to_string(&v.currency).to_owned(),
                    })
                }
                ExchangeOperationType::Transaction(v) => {
                    ExchangeOperationTypeStr::Transaction(TransactionOptString {
                        exchange: lexicon.exchange_to_string(&v.exchange).to_owned(),
                        side: v.side,
                        currency_from: lexicon.currency_to_string(&v.currency_from).to_owned(),
                        currency_to: lexicon.currency_to_string(&v.currency_to).to_owned(),
                    })
                }
                ExchangeOperationType::Transfer(v) => {
                    ExchangeOperationTypeStr::Transfer(TransferOptString {
                        withdraw_exchange: lexicon
                            .exchange_to_string(&v.withdraw_exchange)
                            .to_owned(),
                        deposit_exchange: lexicon
                            .exchange_to_string(&v.deposit_exchange)
                            .to_owned(),
                        currency: lexicon.currency_to_string(&v.currency).to_owned(),
                    })
                }
            })
            .collect();
        match serde_json::to_string(&path) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(format!("Err {}", err)),
        }
    }
}

pub trait PrintPath {
    fn string_id(&self) -> String;
    fn to_named_exchanges_currency_json(&self, lexicon: &CryptoExchangeLexicon) -> Result<String, String>;
    fn path_node_count(&self) -> usize;

    fn print_path_all(&self, lexicon: &CryptoExchangeLexicon) {
        println!("{}", self.string_id());
        if let Ok(json_str) = self.to_named_exchanges_currency_json(&lexicon) {
            println!("{}", json_str);
        } else {
            println!("Unable to serialize");
        }
        println!("SIZE={}", self.path_node_count());
        println!();
    }
}

impl PrintPath for ArbitragePath {
    fn string_id(&self) -> String {
        print_path::string_id(self.0.iter())
    }

    fn to_named_exchanges_currency_json(
        &self,
        lexicon: &CryptoExchangeLexicon,
    ) -> Result<String, String> {
        print_path::to_named_exchanges_currency_json(self.0.iter(), &lexicon) 
    }

    fn path_node_count(&self) -> usize {
        self.0.len()
    }
}

impl PrintPath for ArbitragePath7Nodes {
    fn string_id(&self) -> String {
        print_path::string_id(self.0.iter())
    }

    fn to_named_exchanges_currency_json(
        &self,
        lexicon: &CryptoExchangeLexicon,
    ) -> Result<String, String> {
        print_path::to_named_exchanges_currency_json(self.0.iter(), &lexicon) 
    }

    fn path_node_count(&self) -> usize {
        self.0.len()
    }
}

impl PrintPath for ArbitragePath11Nodes {
    fn string_id(&self) -> String {
        print_path::string_id(self.0.iter())
    }

    fn to_named_exchanges_currency_json(
        &self,
        lexicon: &CryptoExchangeLexicon,
    ) -> Result<String, String> {
        print_path::to_named_exchanges_currency_json(self.0.iter(), &lexicon) 
    }

    fn path_node_count(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_exchange_only_path() {
        
    }

}
