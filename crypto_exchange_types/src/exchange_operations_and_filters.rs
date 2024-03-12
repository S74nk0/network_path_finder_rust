use crate::arbitrage_paths::*;
use crate::id_types::*;
use crate::order_book::*;
use crate::price_amounts::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub struct TransactionFee(pub f64);
pub struct TransferFee(pub f64);
pub struct ExchangeOfflineWithdrawals(pub BTreeSet<CurrencyID>);
pub struct ExchangeOfflineDeposits(pub BTreeSet<CurrencyID>);

pub struct WithdrawTransferFees(pub BTreeMap<CurrencyID, TransferFee>);
pub struct DepositTransferFees(pub BTreeMap<CurrencyID, TransferFee>);

// TODO calculate withdraw and deposit times

pub struct BalancePointInTimeSnapshotData {
    pub amount: CurrencyAmount,
    // operation_duration: // TODO this is probably the duration it took to get here
}

//#[repr(u8)]
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    TAKER,
    MAKER,
}

pub struct TransactionLeftAmount {
    pub amount: CurrencyAmount,
    pub currency: CurrencyID,
}

pub struct TransactionPointInTimeSnapshotData {
    pub fee: TransactionFee,
    pub tr_type: TransactionType,
    pub left_amount: Option<TransactionLeftAmount>,
    pub orderbook_spread_percentage: f64,
    // OperationDuration time.Duration `json:"-"`
    // DataTimestamp     int64         `json:"-"`
}

pub struct TransferPointInTimeSnapshotData {
    pub withdraw_fee: TransferFee,
    pub deposit_fee: TransferFee,
    // //
    // OperationDuration time.Duration `json:"-"`
}

pub enum ExchangeOperationPointInTimeSnapshotData {
    Balance(BalancePointInTimeSnapshotData),
    Transaction(TransactionPointInTimeSnapshotData),
    Transfer(TransferPointInTimeSnapshotData),
}

/// This is the variable data that we execute on known paths.
/// This is variable data to the snapshot in time according to our
/// data we have provided (orderbook streams for now)
pub struct ArbitragePathPointInTimeSnapshotData(pub Vec<ExchangeOperationPointInTimeSnapshotData>);

fn get_delta_balance(path_snapshot: &ArbitragePathPointInTimeSnapshotData) -> Option<f64> {
    if path_snapshot.0.len() < 2 {
        return None;
    }

    match (path_snapshot.0.first(), path_snapshot.0.last()) {
        (
            Some(ExchangeOperationPointInTimeSnapshotData::Balance(first)),
            Some(ExchangeOperationPointInTimeSnapshotData::Balance(last)),
        ) => {
            // TODO examine if this holds true
            Some(1.0 - (first.amount.0 / last.amount.0))
        }
        (_, _) => None,
    }
}

fn orderbook_spread_percentage_average(
    path_snapshot: &ArbitragePathPointInTimeSnapshotData,
) -> Option<f64> {
    let txs: Vec<_> = path_snapshot
        .0
        .iter()
        .filter_map(|pit_data| {
            if let ExchangeOperationPointInTimeSnapshotData::Transaction(tx) = pit_data {
                Some(tx.orderbook_spread_percentage)
            } else {
                None
            }
        })
        .collect();
    if txs.len() > 0 {
        Some(txs.iter().sum::<f64>() / (txs.len() as f64))
    } else {
        None
    }
}

fn left_amounts_percentage_average(
    path_snapshot: &ArbitragePathPointInTimeSnapshotData,
) -> Option<f64> {
    let left_amounts: Vec<_> = path_snapshot
        .0
        .iter()
        .filter_map(|pit_data| {
            if let ExchangeOperationPointInTimeSnapshotData::Transaction(tx) = pit_data {
                if let Some(left_amount) = &tx.left_amount {
                    return Some(left_amount.amount.0);
                }
            }
            None
        })
        .collect();
    if left_amounts.len() > 0 {
        Some(left_amounts.iter().sum::<f64>() / (left_amounts.len() as f64))
    } else {
        None
    }
}

// TODO parse timestamps

fn calculate_with_fee_amount(amount: CurrencyAmount, fee: TransactionFee) -> CurrencyAmount {
    CurrencyAmount(amount.0 - (amount.0 * (fee.0 / 100.0)))
}

// // TODO implement orderbooks sync thingy first in order to finish this one
// pub fn calculate_path_point_in_time_data(
//     start_amount: CurrencyAmount,
//     path: &ArbitragePath,
//     exchanges_orderbooks: &ExchangesOrderBooks,
// ) -> (
//     Option<ArbitragePathPointInTimeSnapshotData>,
//     i32, /*PathStatStatus*/
// ) {
// }
