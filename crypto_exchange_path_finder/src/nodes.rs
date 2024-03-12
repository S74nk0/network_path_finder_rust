use crate::search_stop_settings::SearchStopSettings;
use crypto_exchange_types::*;
use std::rc::Rc;

pub type FromNode = Rc<dyn NodeTrait>;
pub type FromNodeOption = Option<Rc<dyn NodeTrait>>;
// base trait
pub trait NodeTrait: ExchangeOperationTypeInfo {
    // level depth stuff
    fn level(&self) -> u8;
    // node connection stuff
    fn from_node(&self) -> FromNodeOption;
}

// TODO maybe add functions related to NodeTraits here
// e.g. transfer count, transaction count, cycles

pub struct Node<T: ExchangeOperationTypeInfo> {
    level: u8,
    from: FromNodeOption,
    pub operation_data: T,
}

impl<T: ExchangeOperationTypeInfo> Node<T> {
    fn new(parent: FromNodeOption, level: u8, operation_data: T) -> Node<T> {
        Node {
            level: level,
            from: parent,
            operation_data: operation_data,
        }
    }
}

impl<T: ExchangeOperationTypeInfo> ExchangeOperationTypeInfo for Node<T> {
    fn get_operation_type(&self) -> ExchangeOperationType {
        self.operation_data.get_operation_type()
    }
    fn to_json(&self) -> String {
        self.operation_data.to_json()
    }
}

impl<T: ExchangeOperationTypeInfo> NodeTrait for Node<T> {
    fn level(&self) -> u8 {
        self.level
    }
    fn from_node(&self) -> FromNodeOption {
        // consider this cheap op
        match &self.from {
            Some(p) => Some(Rc::clone(&p)),
            None => None,
        }
    }
}

// these are our node types
pub type BalanceNode = Node<BalanceExchangeCurrencyInfo>;
pub type TransferNode = Node<TransferExchangeToExchangeCurrencyInfo>;
pub type TransactionNode = Node<TransactionExchangeCurrenciesBuySellInfo>;

impl BalanceNode {
    #[inline]
    pub fn create(
        parent: FromNodeOption,
        level: u8,
        operation_data: BalanceExchangeCurrencyInfo,
    ) -> BalanceNode {
        Node::new(parent, level, operation_data)
    }
}

impl TransferNode {
    #[inline]
    pub fn create(
        parent: FromNodeOption,
        level: u8,
        operation_data: TransferExchangeToExchangeCurrencyInfo,
    ) -> TransferNode {
        Node::new(parent, level, operation_data)
    }
}

impl TransactionNode {
    #[inline]
    pub fn create(
        parent: FromNodeOption,
        level: u8,
        operation_data: TransactionExchangeCurrenciesBuySellInfo,
    ) -> TransactionNode {
        Node::new(parent, level, operation_data)
    }
}

// TODO bid/ask buy sell doesn't look right
// but the graph works out fine
// TODO https://www.investopedia.com/terms/c/currency_pair.asp
#[inline]
pub fn execute_and_connect_transaction(
    b: Rc<BalanceNode>,
    pair: CurrencyIDPair,
    search_stop_settings: &SearchStopSettings,
) -> Option<BalanceNode> {
    let exchange = b.operation_data.exchange;
    let c = b.operation_data.currency;
    let next_level_depth = b.level() + 1u8;

    // TODO maybe return error
    let (new_balance_currency, side) = pair.next_currency_and_side(c).expect("executeAndConnectTransaction currency is bid and sell are equal FATAL ERROR");

    let t_info = TransactionExchangeCurrenciesBuySellInfo {
        exchange: exchange,
        side: side,
        currency_from: c,
        currency_to: new_balance_currency,
    };
    let t = Rc::new(TransactionNode::create(Some(b), next_level_depth, t_info));
    // filter
    let filter_by_operation = Rc::clone(&t);
    if search_stop_settings.is_skip_search_node(filter_by_operation) {
        return None;
    }
    let tx_balance_info = BalanceExchangeCurrencyInfo {
        exchange: exchange,
        currency: new_balance_currency,
    };
    let tx_b = BalanceNode::create(Some(t), next_level_depth, tx_balance_info);
    Some(tx_b)
}

#[inline]
pub fn execute_and_connect_transfer(
    b: Rc<BalanceNode>,
    to_exchange: ExchangeID,
    search_stop_settings: &SearchStopSettings,
) -> Option<BalanceNode> {
    let exchange = b.operation_data.exchange;
    let currency = b.operation_data.currency;

    let next_level_depth = b.level() + 1u8;
    let tr_info = TransferExchangeToExchangeCurrencyInfo {
        withdraw_exchange: exchange,
        deposit_exchange: to_exchange,
        currency: currency,
    };
    let tr = Rc::new(TransferNode::create(Some(b), next_level_depth, tr_info));
    // filter
    let filter_by_operation = Rc::clone(&tr);
    if search_stop_settings.is_skip_search_node(filter_by_operation) {
        return None;
    }
    let tr_b_info = BalanceExchangeCurrencyInfo {
        exchange: to_exchange,
        currency: currency,
    };
    let tr_b = BalanceNode::create(Some(tr), next_level_depth, tr_b_info);
    Some(tr_b)
}
