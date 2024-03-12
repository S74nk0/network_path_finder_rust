use crate::nodes::*;
use crate::search_stop_settings::*;
use crypto_exchange_types::*;
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet, LinkedList};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[inline]
fn walk_up_linearized(n: FromNode) -> (ArbitragePath, bool) {
    let alloc_reserve_size = ((2 * n.level()) + 1) as usize;
    let mut path_up: ArbitragePath = ArbitragePath::new();
    let mut index = (alloc_reserve_size - 1) as isize;
    let mut next_node = Some(n);
    while next_node.is_some() && (index > -1) {
        next_node = match next_node {
            Some(node) => {
                path_up.0.push_front(node.get_operation_type());
                node.from_node()
            }
            None => None,
        };
        index = index - 1;
    }
    let ok = next_node.is_none() && index == -1;
    (path_up, ok)
}

// TODO there is no need to keep the exchange and all supported pairs here
// this data is and should be from the fundamental lexicon data type
// TODO simplify this to only hold data for transaction pairs and currency to exchanges
// TODO here you can also add aditional filters like withraw from and to limits but this is
// not really important for now
struct ExchangeNetworkHub {
    all_supported_pairs: HashSet<CurrencyIDPair>,
    currency_transaction_pairs: HashMap<CurrencyID, HashSet<CurrencyIDPair>>,
    currency_to_exchanges: HashMap<CurrencyID, HashSet<ExchangeID>>,
}
// SPLIT to build network and search network mode
// TODO split the network so we have a
// seaprate imutable struct that is basically exchange_hubs
// the network from lexicon part doesn't change unless we update the lexicon
// instead of the network have a network search struct
pub struct Network {
    exchange_hubs: HashMap<ExchangeID, ExchangeNetworkHub>,
    is_update_exchange_hubs: bool,
}

impl Network {
    pub fn new() -> Network {
        Network {
            exchange_hubs: HashMap::new(),
            is_update_exchange_hubs: false,
        }
    }

    pub fn add_pairs(&mut self, exchange: ExchangeID, cps: &[CurrencyIDPair]) -> bool {
        let mut is_updated = false;
        if !self.exchange_hubs.contains_key(&exchange) {
            is_updated = true;
            // TODO add
            self.exchange_hubs.insert(
                exchange,
                ExchangeNetworkHub {
                    all_supported_pairs: HashSet::new(),
                    currency_transaction_pairs: HashMap::new(),
                    currency_to_exchanges: HashMap::new(),
                },
            );
        }
        // it should exist
        let exchange_entry = self.exchange_hubs.get_mut(&exchange).unwrap();
        for cp in cps.iter() {
            let new_inserted = exchange_entry.all_supported_pairs.insert(*cp);
            is_updated = is_updated || new_inserted;
        }
        self.is_update_exchange_hubs = self.is_update_exchange_hubs || is_updated;
        is_updated
    }

    // TODO cache last call to this method. We need to execute the update only when we add targets and/or pairs
    // TODO mark as private
    pub fn update_exchange_hubs(&mut self) {
        if !self.is_update_exchange_hubs {
            return;
        }
        self.is_update_exchange_hubs = true;
        // update/init all exchanges transactions first
        self.exchange_hubs.iter_mut().for_each(|exchange_hub_pair| {
            // TODO here we completely replace the
            let (_, mut exchange_hub) = exchange_hub_pair;
            let mut currency_transaction_pairs: HashMap<CurrencyID, HashSet<CurrencyIDPair>> =
                HashMap::new();
            exchange_hub
                .all_supported_pairs
                .iter()
                .for_each(|currency_pair| {
                    // TODO check if inserted
                    // update sets first and second and check if we have inserted
                    let _inserted = currency_transaction_pairs
                        .entry(currency_pair.first)
                        .or_insert(HashSet::new())
                        .insert(*currency_pair)
                        && currency_transaction_pairs
                            .entry(currency_pair.second)
                            .or_insert(HashSet::new())
                            .insert(*currency_pair);
                    // TODO check insertion
                });
            // set to point to new
            exchange_hub.currency_transaction_pairs = currency_transaction_pairs
        });

        // update/init all transfers after we updated transactions
        let set_pairs: Vec<_> = self
            .exchange_hubs
            .iter()
            .map(|exchange_hub_pair| {
                // TODO here we completely replace the exchange_id_currency_to_exchanges
                let (exchange_id, exchange_hub) = exchange_hub_pair;
                let mut exchange_crypto_currencies_set: HashSet<CurrencyID> = HashSet::new();
                exchange_hub
                    .all_supported_pairs
                    .iter()
                    .for_each(|currency_pair| {
                        // if !currency_pair.first.is_fiat_currency() {
                        //     exchange_crypto_currencies_set.insert(currency_pair.first);
                        // }
                        // if !currency_pair.second.is_fiat_currency() {
                        //     exchange_crypto_currencies_set.insert(currency_pair.second);
                        // }
                        // we will filter fiat currencies later
                        exchange_crypto_currencies_set.insert(currency_pair.first);
                        exchange_crypto_currencies_set.insert(currency_pair.second);
                    });
                let exchange_crypto_currencies_set = exchange_crypto_currencies_set; // drop mutable
                let mut currency_to_exchanges: HashMap<CurrencyID, HashSet<ExchangeID>> =
                    HashMap::new();
                self.exchange_hubs.iter().for_each(|exchange_hub_pair2| {
                    let (exchange_id2, exchange_hub2) = exchange_hub_pair2;
                    if *exchange_id == *exchange_id2 {
                        return;
                    }
                    exchange_hub2
                        .all_supported_pairs
                        .iter()
                        .for_each(|currency_pair| {
                            if exchange_crypto_currencies_set.contains(&currency_pair.first) {
                                currency_to_exchanges
                                    .entry(currency_pair.first)
                                    .or_insert(HashSet::new())
                                    .insert(*exchange_id2);
                            }
                            if exchange_crypto_currencies_set.contains(&currency_pair.second) {
                                currency_to_exchanges
                                    .entry(currency_pair.second)
                                    .or_insert(HashSet::new())
                                    .insert(*exchange_id2);
                            }
                        });
                });
                (*exchange_id, currency_to_exchanges)
            })
            .collect();
        for pair in set_pairs {
            let (exchange_id, currency_to_exchanges) = pair;
            if let Some(exchange_hub) = self.exchange_hubs.get_mut(&exchange_id) {
                exchange_hub.currency_to_exchanges = currency_to_exchanges;
            }
        }
    }

    // TODO make it so we decide parallel or single threded
    // probably max leve 2 or 3 single threaded => TEST TEST TEST!!!
    pub fn search_targets(
        &self,
        targets: HashSet<Target>,
        search_stop_settings: &SearchStopSettings,
        processed_count: Arc<AtomicUsize>,
    ) -> HashMap<Target, TargetKnownPaths> {
        let n_arc = Arc::new(self);
        // for now using chanels instead of mutex, maybe mutex will be faster
        let (sender, receiver) = std::sync::mpsc::channel();
        targets.into_par_iter().for_each_with(sender, |s, target| {
            let target_paths = n_arc.search_target_start(target, &search_stop_settings);
            let new_count = processed_count.load(Ordering::Relaxed) + 1usize;
            processed_count.store(new_count, Ordering::Relaxed);
            s.send((target, target_paths))
                .expect("Unable to send searched target paths");
        });
        // collect results and update network paths
        let pre_calced_paths: HashMap<Target, TargetKnownPaths> = receiver.into_iter().collect();
        pre_calced_paths
    }

    pub fn search_targets_channel(
        &self,
        targets: HashSet<Target>,
        search_stop_settings: &SearchStopSettings,
        processed_count: Arc<AtomicUsize>,
        sender: std::sync::mpsc::Sender<(ExchangeIDCurrencyIDPair, HashSet<ArbitragePath>)>
    ) {
        let n_arc = Arc::new(self);
        targets.into_par_iter().for_each_with(sender, |s, target| {
            let target_paths = n_arc.search_target_start(target, &search_stop_settings);
            let new_count = processed_count.load(Ordering::Relaxed) + 1usize;
            processed_count.store(new_count, Ordering::Relaxed);
            s.send((target, target_paths))
                .expect("Unable to send searched target paths");
        });
    }

    pub fn search_targets_parallel(
        &self,
        targets: HashSet<Target>,
        search_settings: &SearchStopSettings
    ) -> HashMap<Target, TargetKnownPaths> {
        let pre_calced_paths: HashMap<Target, TargetKnownPaths> = targets
            .into_par_iter()
            .map(|target| {
                let target_paths = self.search_target_start(target, &search_settings);
                (target, target_paths)
            })
            .collect();
        pre_calced_paths
    }

    pub fn search_targets_sync(
        &self,
        targets: HashSet<Target>,
        search_settings: &SearchStopSettings
    ) -> HashMap<Target, TargetKnownPaths> {
        let pre_calced_paths: HashMap<Target, TargetKnownPaths> = targets
            .into_iter()
            .map(|target| {
                let target_paths = self.search_target_start(target, &search_settings);
                (target, target_paths)
            })
            .collect();
        pre_calced_paths
    }

    pub fn search_targets_sync_progress(
        &self,
        targets: HashSet<Target>,
        search_stop_settings: &SearchStopSettings,
        processed_count: Arc<AtomicUsize>,
    ) -> HashMap<Target, TargetKnownPaths> {
        let sync_algo = targets.into_iter().map(|target| {
            let target_paths = self.search_target_start(target, &search_stop_settings);
            let new_count = processed_count.load(Ordering::Relaxed) + 1usize;
            processed_count.store(new_count, Ordering::Relaxed);
            (target, target_paths)
        });
        // collect results and update network paths
        let pre_calced_paths: HashMap<Target, TargetKnownPaths> = sync_algo.collect();
        pre_calced_paths
    }

    #[inline]
    fn search_target_start(
        &self,
        t: Target,
        search_stop_settings: &SearchStopSettings,
    ) -> TargetKnownPaths {
        // step #01
        let target_node = Rc::new(BalanceNode::create(None, 0, t));
        // search_filter ultra expensive but could run in parallel
        let leafs = self.search_filter(true, t.currency, target_node, &search_stop_settings);
        // step #02 walk_up_target_leafs_to_known_paths
        let target_paths: TargetKnownPaths = leafs
            .into_iter()
            .map(|leaf| {
                // TODO this ok here should we handle it??
                let (path_up, _ok) = walk_up_linearized(leaf);
                path_up
            })
            .collect();
        target_paths
    }

    // strategies depend here, executing transfers in a row makes no sense
    // but what about transactions in a row? they could make sense
    fn search_filter(
        &self,
        is_last_transfer: bool, // so we ignore transfers in a row, this would make first transfer filter obsolete
        target_currency: CurrencyID,
        next: Rc<BalanceNode>,
        search_stop_settings: &SearchStopSettings,
    ) -> LinkedList<Rc<BalanceNode>> {
        let mut leafs: LinkedList<Rc<BalanceNode>> = LinkedList::new();

        let next_data = next.operation_data;
        let should_append = target_currency == next_data.currency && next.from_node().is_some();
        if should_append {
            leafs.push_back(Rc::clone(&next));
        }
        let check_next_skip = Rc::clone(&next);
        if search_stop_settings.is_skip_search_node(check_next_skip) {
            return leafs;
        }

        // Transactions
        if let Some(exchange_hub) = self.exchange_hubs.get(&next_data.exchange) {
            if let Some(transaction_pairs) = exchange_hub
                .currency_transaction_pairs
                .get(&next_data.currency)
            {
                for currency_pair in transaction_pairs {
                    if let Some(new_next) = execute_and_connect_transaction(
                        Rc::clone(&next),
                        *currency_pair,
                        &search_stop_settings,
                    ) {
                        let mut leafs_append = self.search_filter(
                            !is_last_transfer,
                            target_currency,
                            Rc::new(new_next),
                            &search_stop_settings,
                        );
                        leafs.append(&mut leafs_append);
                    }
                }
            }
        }
        if is_last_transfer {
            return leafs;
        }

        // Transfers
        if let Some(exchange_hub) = self.exchange_hubs.get(&next_data.exchange) {
            if let Some(transfer_pairs) =
                exchange_hub.currency_to_exchanges.get(&next_data.currency)
            {
                // TODO par_iter Rayon test
                transfer_pairs.iter().for_each(|exchange| {
                    if let Some(new_next) = execute_and_connect_transfer(
                        Rc::clone(&next),
                        *exchange,
                        &search_stop_settings,
                    ) {
                        let mut new_leafs = self.search_filter(
                            !is_last_transfer,
                            target_currency,
                            Rc::new(new_next),
                            &search_stop_settings,
                        );
                        leafs.append(&mut new_leafs);
                    }
                });
            }
        }
        leafs
    }
}


#[derive(Serialize, Deserialize)]
pub struct OptimizedPreCalcedPaths {
    pub id: BalanceExchangeCurrencyInfo,
    pub tr_7_paths: Option<Vec<ArbitragePath7Nodes>>,
    pub tr_11_paths: Option<Vec<ArbitragePath11Nodes>>,
    pub tx_only_3pairs_paths: Option<Vec<SingleExchangeTransactionOnly3PairsPath>>,
    pub tx_only_5pairs_paths: Option<Vec<SingleExchangeTransactionOnly5PairsPath>>,
    pub unknown_paths: Option<Vec<ArbitragePath>>,
}

#[derive(Default, Debug)]
pub struct OptimizedPreCalcedPathsStats {
    pub estimated_size_in_bytes: usize,
    pub tr_7_paths: usize,
    pub tr_11_paths: usize,
    pub tx_only_3pairs_paths: usize,
    pub tx_only_5pairs_paths: usize,
    pub unknown_paths: usize,
}

// TODO get stats and 
impl OptimizedPreCalcedPaths {
    pub fn stats(&self) -> OptimizedPreCalcedPathsStats {
        let tr_7_paths: usize = self.tr_7_paths.as_ref().map_or(0usize, |v| v.len());
        let tr_11_paths: usize = self.tr_11_paths.as_ref().map_or(0usize, |v| v.len());
        let tx_only_3pairs_paths: usize = self.tx_only_3pairs_paths.as_ref().map_or(0usize, |v| v.len());
        let tx_only_5pairs_paths: usize = self.tx_only_5pairs_paths.as_ref().map_or(0usize, |v| v.len());
        let unknown_paths: usize = self.unknown_paths.as_ref().map_or(0usize, |v| v.len());

        let estimated_size_in_bytes = 
        std::mem::size_of::<BalanceExchangeCurrencyInfo>() + 
        std::mem::size_of::<Option<Vec<ArbitragePath7Nodes>>>() + std::mem::size_of::<ArbitragePath7Nodes>() * tr_7_paths +
        std::mem::size_of::<Option<Vec<ArbitragePath11Nodes>>>() + std::mem::size_of::<ArbitragePath11Nodes>() * tr_11_paths +
        std::mem::size_of::<Option<Vec<SingleExchangeTransactionOnly3PairsPath>>>() + std::mem::size_of::<SingleExchangeTransactionOnly3PairsPath>() * tx_only_3pairs_paths +
        std::mem::size_of::<Option<Vec<SingleExchangeTransactionOnly5PairsPath>>>() + std::mem::size_of::<SingleExchangeTransactionOnly5PairsPath>() * tx_only_5pairs_paths +
        std::mem::size_of::<Option<Vec<ArbitragePath>>>() + self.unknown_paths.iter().flat_map(|p| p.iter().map(|p| p.0.len())).sum::<usize>() * std::mem::size_of::<ArbitragePath>() ; // 
        
        OptimizedPreCalcedPathsStats {
            estimated_size_in_bytes,
            tr_7_paths,
            tr_11_paths,
            tx_only_3pairs_paths,
            tx_only_5pairs_paths,
            unknown_paths,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OptimizedNetworkWithLexicon {
    pub lexicon: CryptoExchangeLexicon,
    pub pre_calced_paths: BTreeMap<BalanceExchangeCurrencyInfo, OptimizedPreCalcedPaths>,
    pub search_stop_settings: SearchStopSettings,
}

impl OptimizedNetworkWithLexicon {
    pub fn read_from_file<P: AsRef<Path>>(file_path: &P) -> anyhow::Result<OptimizedNetworkWithLexicon> {
        let mut file = File::open(file_path)?;
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents)?;
        let ret = crypto_serializer::cbor_from_slice_lz4(&contents)?;
        Ok(ret)
    }
}