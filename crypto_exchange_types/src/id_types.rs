use serde::{Deserialize, Serialize};
use string_to_int_mapper::GetCurrentAndIncrementStringToIntMapperId;
use std::fmt;
use thiserror::Error;

/// id_types.rs holds imutable persistant IDs. These IDs is generated by the ('Consensus') Lexicon
/// and they change only with the consensus lexicon changes. TODO make migrationa for the ('Consensus') Lexicon.
/// We might need an aditional currency type mappers.




// TODO rename
#[derive(Error, Debug)]
pub enum CryptoTypesError {
    #[error("Next currency error. Unknown BUY/SELL relation for currency '{0}' and pair '{1}'")]
    NextCurrency(CurrencyID, CurrencyIDPair)
}



/// Immutable strongly typed ExchangeID. This ID is generated by the ('Consensus') Lexicon  
#[derive(Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct ExchangeID(pub u8);

impl From<u8> for ExchangeID {
    fn from(value: u8) -> Self {
        ExchangeID(value)
    }
}

impl GetCurrentAndIncrementStringToIntMapperId for ExchangeID {
    fn get_and_increment(&mut self) -> Self {
        let ret = self.clone();
        self.0 = self.0 + 1u8;
        ret
    }
    fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl ExchangeID {
    pub fn with_currency(&self, c: &CurrencyID) -> ExchangeIDCurrencyIDPair {
        (self.clone(), c.clone()).into()
    }
}

/// Immutable strongly typed CurrencyID. This is used to identify Currencies (Crypto and Fiat)
/// This ID is generated by the ('Consensus') Lexicon
#[derive(Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct CurrencyID(pub u16);

impl From<u16> for CurrencyID {
    fn from(value: u16) -> Self {
        CurrencyID(value)
    }
}

impl GetCurrentAndIncrementStringToIntMapperId for CurrencyID {
    fn get_and_increment(&mut self) -> Self {
        let ret = self.clone();
        self.0 = self.0 + 1u16;
        ret
    }
    fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for CurrencyID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c={}", self.0)
    }
}

// TODO https://www.investopedia.com/terms/c/currencypair.asp
// TODO doesn't add up the bid ask sell buy stuff
/// Immutable CurrencyIDPair used for identifying markets.
/// This ID is generated by the ('Consensus') Lexicon
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct CurrencyIDPair {
    /// first or base currency ID
    pub first: CurrencyID,  // base
    /// second or quote currency ID
    pub second: CurrencyID, // quote
}

impl fmt::Display for CurrencyIDPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "base/first={}, quote/second={}", self.first.0, self.second.0)
    }
}

impl CurrencyIDPair {
    pub fn new(first: CurrencyID, second: CurrencyID) -> Self {
        CurrencyIDPair { first, second }
    }
    pub fn cointains(&self, c: &CurrencyID) -> bool {
        self.first.eq(c) || self.second.eq(c)
    }

    // we don't want to have inverse pairs on the same exchange
    pub fn is_inverse(a: &CurrencyIDPair, b: &CurrencyIDPair) -> bool {
        a.first == b.second && a.second == b.first
    }
    pub fn has_same_currencies(&self) -> bool {
        self.first == self.second
    }
    // pub fn consume_reverse(self) -> Self {
    //     CurrencyIDPair::new CurrencyIDPair { second, second }
    // }

    pub fn next_currency_and_side(&self, from_currency: CurrencyID) -> Result<(CurrencyID, TransactionSide), CryptoTypesError> {
        let is_bid = self.first == from_currency;
        let is_sell = self.second == from_currency;

        if is_bid == is_sell {
            return Err(CryptoTypesError::NextCurrency(from_currency, self.clone()));
        }
    
        let (new_balance_currency, side) = if is_bid {
            (self.second, TransactionSide::BUY)
        } else {
            (self.first, TransactionSide::SELL)
        };
        Ok((new_balance_currency, side))
    }
}

/// Immutable ExchangeIDCurrencyIDPair used for identifying crypto holdings/balance, volume
/// This ID is generated by the ('Consensus') Lexicon
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct ExchangeIDCurrencyIDPair {
    pub exchange: ExchangeID,
    pub currency: CurrencyID,
}

impl From<(ExchangeID, CurrencyID)> for ExchangeIDCurrencyIDPair {
    fn from(value: (ExchangeID, CurrencyID)) -> Self {
        let (exchange, currency) = value;
        Self { exchange, currency }
    }
}

impl From<(CurrencyID, ExchangeID)> for ExchangeIDCurrencyIDPair {
    fn from(value: (CurrencyID, ExchangeID)) -> Self {
        let (currency, exchange) = value;
        Self { exchange, currency }
    }
}

//#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionSide {
    BUY,
    SELL,
}

// TODO consider Removing Clone and Copy
// TODO you could simply put data inside the enums
// joined enum TransactionSide+TransactionType pair type
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum ExchangeOperationType {
    Balance(BalanceExchangeCurrencyInfo),
    Transaction(TransactionExchangeCurrenciesBuySellInfo),
    Transfer(TransferExchangeToExchangeCurrencyInfo),
}

impl From<BalanceExchangeCurrencyInfo> for ExchangeOperationType {
    fn from(value: BalanceExchangeCurrencyInfo) -> Self {
        Self::Balance(value)
    }
}

impl From<TransactionExchangeCurrenciesBuySellInfo> for ExchangeOperationType {
    fn from(value: TransactionExchangeCurrenciesBuySellInfo) -> Self {
        Self::Transaction(value)
    }
}

impl From<TransferExchangeToExchangeCurrencyInfo> for ExchangeOperationType {
    fn from(value: TransferExchangeToExchangeCurrencyInfo) -> Self {
        Self::Transfer(value)
    }
}

impl fmt::Display for ExchangeOperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExchangeOperationType::Balance(b) => {
                write!(f, "b(e={},c={})", b.exchange.0, b.currency.0)
            }
            ExchangeOperationType::Transaction(tx) => write!(
                f,
                "tx(e={},s={},cf={},ct={})",
                tx.exchange.0, tx.side as i32, tx.currency_from.0, tx.currency_to.0
            ),
            ExchangeOperationType::Transfer(tr) => write!(
                f,
                "tr(we={},de={},c={})",
                tr.withdraw_exchange.0, tr.deposit_exchange.0, tr.currency.0
            ),
        }
    }
}

impl ExchangeOperationType {
    pub fn is_inverse(a: &Self, b: &Self) -> bool {
        match (a, b) {
            (Self::Transaction(a_v), Self::Transaction(b_v)) => {
                a_v.is_transaction_inverse(&b_v)
            }
            (Self::Transfer(a_v), Self::Transfer(b_v)) => {
                a_v.is_transfer_inverse(&b_v)
            }
            (_, _) => false,
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

pub trait ExchangeOperationTypeInfo {
    fn get_operation_type(&self) -> ExchangeOperationType;
    fn to_json(&self) -> String; // TODO this here is problematic
}

// immutable Exchange Opration Types
pub type BalanceExchangeCurrencyInfo = ExchangeIDCurrencyIDPair;
impl ExchangeOperationTypeInfo for BalanceExchangeCurrencyInfo {
    fn get_operation_type(&self) -> ExchangeOperationType {
        ExchangeOperationType::Balance(*self)
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TransactionExchangeCurrenciesBuySellInfo {
    pub exchange: ExchangeID,
    pub side: TransactionSide,
    pub currency_from: CurrencyID,
    pub currency_to: CurrencyID,
}

impl ExchangeOperationTypeInfo for TransactionExchangeCurrenciesBuySellInfo {
    fn get_operation_type(&self) -> ExchangeOperationType {
        ExchangeOperationType::Transaction(*self)
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl TransactionExchangeCurrenciesBuySellInfo {
    pub fn is_transaction_inverse(&self, a: &TransactionExchangeCurrenciesBuySellInfo) -> bool {
        self.exchange == a.exchange
            && self.side != a.side
            && self.currency_from == a.currency_to
            && self.currency_to == a.currency_from
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TransferExchangeToExchangeCurrencyInfo {
    pub withdraw_exchange: ExchangeID,
    pub deposit_exchange: ExchangeID,
    pub currency: CurrencyID,
}

impl ExchangeOperationTypeInfo for TransferExchangeToExchangeCurrencyInfo {
    fn get_operation_type(&self) -> ExchangeOperationType {
        ExchangeOperationType::Transfer(*self)
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl TransferExchangeToExchangeCurrencyInfo {
    pub fn is_transfer_inverse(&self, a: &TransferExchangeToExchangeCurrencyInfo) -> bool {
        return a.currency == self.currency
            && a.withdraw_exchange == self.deposit_exchange
            && a.deposit_exchange == self.withdraw_exchange;
    }
}


#[cfg(test)]
mod tests {
    #![feature(layout_for_ptr)]

    use crate::arbitrage_paths;

    use super::*;

    macro_rules! print_struct_size
    {
        ($struct_name:ident) =>
        {
            println!("{}: is {}", stringify!($struct_name), std::mem::size_of::<$struct_name>());
        };
    }

    enum Side {
        Buy,
        Sell,
    }

    struct ExchangeTransactionPath {
        pub pairs: Vec<(u16, u16)>,
        pub side: Side,
    }

    struct Buy;
    struct Sell;

    struct Pair(u16, u16);

    struct Path3([Pair; 3]);

    struct Path5([Pair; 5]);

    struct ExchangeTransaction3Path {
        pub pairs: Vec<Path3>,
        pub side: std::marker::PhantomData<Buy>,
    }

    struct Tuple(bool, u16, u16);

    struct PathTuple3([Tuple; 3]);

    struct PathTuple5([Tuple; 5]);

    struct PathTuple7([Tuple; 7]);

    struct PathTuple11([Tuple; 11]);

    enum TransactionPath {
        Path3(Path3),
        Path5(Path5),
    }

    enum ExchangeOperation {
        Balance(u16),
        Currency(u16, u16)
    }

    struct ExBalance(u16);
    struct ExPair(u16, u16);
    struct ExchangeTransition(ExBalance, ExPair);
    struct ExchangeOperation7(ExBalance, ExPair, ExBalance, ExPair, ExBalance, ExPair, ExBalance);

    struct ExchangeTransition3(ExchangeTransition, ExchangeTransition, ExchangeTransition);


    // TODO THIS IS THE BEST CANDIDATE For internal exchange transactions!!!
    struct ExchangeTransition3Start(ExBalance, ExPair, ExPair, ExPair);
    struct ExchangeTransition3Start2 {
        start_and_end_balance_id: u16,
        // pairs_in_order: [ExPair; 3]
        middle_pair: ExPair,
    }

    struct ExchangeTransition3Start3 {
        // pairs_in_order: [ExPair; 3]
        middle_pairs : Vec<ExPair>,
        start_and_end_balance_id: u16,
    }
    
    use crate::arbitrage_paths::*;

    #[test]
    fn print_memory_usages_for_experimental() {
        print_struct_size!(Buy);
        print_struct_size!(Sell);
        print_struct_size!(Pair);
        print_struct_size!(Path3);
        print_struct_size!(Path5);
        
        print_struct_size!(TransactionPath);

        print_struct_size!(ExchangeOperation);
        print_struct_size!(ExchangeOperation7);
        print_struct_size!(ExchangeTransition3);
        print_struct_size!(ExchangeTransition3Start);
        print_struct_size!(ExchangeTransition3Start2);
        print_struct_size!(ExchangeTransition3Start3);
        
        

        print_struct_size!(PathTuple3);
        print_struct_size!(PathTuple5);

        struct VecTransactionPath(Vec<TransactionPath>);
        print_struct_size!(VecTransactionPath);

        print_struct_size!(ExchangeTransaction3Path);
        
    }

    #[test]
    fn print_memory_usages() {
        print_struct_size!(ExchangeOperationType);
        print_struct_size!(BalanceExchangeCurrencyInfo);
        print_struct_size!(TransactionExchangeCurrenciesBuySellInfo);
        print_struct_size!(TransferExchangeToExchangeCurrencyInfo);
        

        print_struct_size!(Side);

        print_struct_size!(ExchangeTransactionPath);

        struct tmp(Vec<ExchangeTransactionPath>);
        struct tmp2(Side, tmp);

        let pairs: Vec<(u16, u16)> = (0..100u16).map(|i| (i, i)).collect();
        let tuple3: Vec<(u16, u16, u16)> = (0..100u16).map(|i| (i, i, i)).collect();

        
        print_struct_size!(Target);
        print_struct_size!(ArbitragePath);
        print_struct_size!(TargetKnownPaths);

        println!("{}", std::mem::size_of_val(&pairs[0]));
        println!("{}", std::mem::size_of_val(&tuple3[0]));

        // print_struct_size!(std::collections::LinkedList);

        

        // let pairs = Box::new(pairs);
        // println!("{}", unsafe { std::mem::size_of_val_raw(&*pairs) });
        // println!("{}", std::mem::size_of_val(&tuple3[0]));
        

    }
}