use crate::nodes::FromNode;
use crypto_exchange_types::ExchangeOperationType;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
pub struct SearchStopSettings {
    /// Max node level indicates to stop searching further/ to deeper levels
    /// when the this node is encountered
    pub max_level: u8,
    /// Ignore cycles indicates to stop searching when the encountered node
    /// current path contains a cylce node. A cycle node is any equal or reverse node
    /// in the given path. We don't wan't to buy an asset and sell back to the same asset
    /// e.g. Buy ETH with BTC and sell ETH back for BTC.
    /// Another cycle case is the transfer an asset from one exchange to another
    /// and to transfer that same asset back form the excnage we transfered it from.
    /// e.g. Transfer BTC from Kraken to CoinbasePro and transfer BTC to Kraken back.   
    pub ignore_cycles: bool,
    /// This is to limit the number of transfer operations for a given search.
    /// Since the transfers only happen on the second level depth it doesn't make sense
    /// to set this value higher than the `max_level/2`.
    pub max_transfers: i32,
}

impl SearchStopSettings {
    pub fn new_default() -> Self {
        SearchStopSettings {
            max_level: 4,
            ignore_cycles: true,
            max_transfers: 2,
        }
    }
    pub fn new(max_level: u8, ignore_cycles: bool, max_transfers: i32) -> Self {
        SearchStopSettings {
            max_level,
            ignore_cycles,
            max_transfers,
        }
    }
    pub fn is_skip_search_node(&self, n: FromNode) -> bool {
        if n.level() >= self.max_level {
            return true;
        }
        if self.ignore_cycles && node_has_cycles(n.clone()) {
            return true;
        }

        node_transfer_count(n) > self.max_transfers
    }
}

// A node is considered to have a cycle if there are equal or inverse nodes in the parent path
fn node_has_cycles(n: FromNode) -> bool {
    let a = n.get_operation_type();
    let mut parent_node = n.from_node();
    while parent_node.is_some() {
        parent_node = match parent_node {
            Some(node) => {
                let b = node.get_operation_type();
                let parent = node.from_node();
                let is_skip = ExchangeOperationType::eq(&a, &b)
                    || ExchangeOperationType::is_inverse(&a, &b);
                if parent.is_some() && is_skip {
                    return true;
                }
                parent
            }
            None => None,
        };
    }
    false
}

fn node_transfer_count(n: FromNode) -> i32 {
    let mut transfer_count: i32 = 0;
    let mut next_node = Some(Rc::clone(&n));
    while next_node.is_some() {
        next_node = match next_node {
            Some(node) => {
                match node.get_operation_type() {
                    ExchangeOperationType::Transfer(__) => transfer_count += 1,
                    _ => (),
                }
                node.from_node()
            }
            None => None,
        }
    }
    transfer_count
}
