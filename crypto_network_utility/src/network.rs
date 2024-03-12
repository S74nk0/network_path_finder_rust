use crate::args_parser::{NetworkCommand, GenerateNetworkInOutFile};
use crate::file_utils;
use crate::lexicon;
use crypto_exchange_path_finder::{OptimizedPreCalcedPathsStats, OptimizedPreCalcedPaths, OptimizedNetworkWithLexicon};
use ::crypto_exchange_path_finder::{Network, SearchStopSettings};
use crypto_exchange_types::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use itertools::Itertools;


pub fn handle_network_command(network_command: NetworkCommand) -> anyhow::Result<()> {
    use NetworkCommand::*;
    match network_command {
        Generate(opts) => generate_network_with_lexicon_file(opts),
        
        Resume { path, max_chunk_size } => {
            resume_network_generation_for_path(&path, max_chunk_size)
        }
        MergeIntoNetwork { path } => {
            merge_into_network_for_path(&path)
        }
        PrintAllPaths { path } => {
            print_all_network_paths(&path)
        }
        
        PrintStatistics { path } => print_network_stats(&path),
    }
}

/// This is a helper struct for the working session
struct WorkingSession {
    in_lexicon_file_path: PathBuf,
    output_working_dir_path: PathBuf,
    search_stop_settings: SearchStopSettings,
    lexicon: CryptoExchangeLexicon,
    max_chunk_size: Option<usize>,
}

impl WorkingSession {
    fn try_init(opts: GenerateNetworkInOutFile, path_sub_dir: &str) -> anyhow::Result<WorkingSession> {
        let in_file_path = opts.in_file_lexicon_path;
        let output_dir_path = opts.output_dir_path;
        file_utils::file_must_exist(&in_file_path)?;
        file_utils::file_must_not_exist(&output_dir_path)?;
        println!(
            "Generating network file '{}' from '{}'",
            &in_file_path.display(), &output_dir_path.display()
        );

        let lexicon_f = lexicon::read_lexicon_file(&in_file_path)?;

        let search_stop_settings = SearchStopSettings {
            max_level: opts.max_level as u8,
            ignore_cycles: !opts.allow_cycles,
            max_transfers: opts.max_transfers,
        };

        let path_root = Path::new(&output_dir_path);
        let path_root_chunks = path_root.join(path_sub_dir);
        fs::create_dir(path_root)?;
        fs::create_dir(&path_root_chunks)?;
        lexicon::save_lexicon_file(path_root.join("lexicon.lex").as_ref(), &lexicon_f)?;
        file_utils::save_json_file(
            &path_root.join("search_stop_settings.json"),
            &search_stop_settings,
        )?;

        Ok(WorkingSession{
            in_lexicon_file_path: in_file_path,
            output_working_dir_path: output_dir_path,
            search_stop_settings: search_stop_settings,
            lexicon: lexicon_f,
            max_chunk_size: opts.max_chunk_size,
        })
    }
}


fn read_search_stop_settings(file_path: &Path) -> anyhow::Result<SearchStopSettings> {
    file_utils::read_json_file(&file_path)
}

fn generate_network_with_lexicon_file(opts: GenerateNetworkInOutFile) -> anyhow::Result<()> {
    let working_session = WorkingSession::try_init(opts, "paths")?;
    generate_network_paths_with_lexicon_file(
        working_session.lexicon,
        working_session.search_stop_settings,
        &working_session.output_working_dir_path,
        working_session.max_chunk_size
    )
}

fn resume_network_generation_for_path(resume_file_path: &Path, max_chunk_size: Option<usize>) -> anyhow::Result<()> {
    file_utils::file_must_exist(&resume_file_path)?;
    println!("Resuming network generation  for '{}'", &resume_file_path.display());

    let path_root = Path::new(&resume_file_path);
    let path_root_lexicon = path_root.join("lexicon.lex");
    let path_root_search_stop_settings = path_root.join("search_stop_settings.json");
    let lexicon_f = lexicon::read_lexicon_file(&path_root_lexicon)?;

    let search_stop_settings = read_search_stop_settings(&path_root_search_stop_settings)?;
    generate_network_paths_with_lexicon_file(
        lexicon_f,
        search_stop_settings,
        &resume_file_path,
        max_chunk_size
    )
}

fn generate_network_paths_with_lexicon_file(
    lexicon_f: CryptoExchangeLexicon,
    search_stop_settings: SearchStopSettings,
    out_file_path: &Path,
    max_chunk_size: Option<usize>,
) -> anyhow::Result<()> {
    let all_targets = {
        let mut all_targets2: BTreeSet<ExchangeIDCurrencyIDPair> = BTreeSet::new();
        lexicon_f.exchange_currency_pairs_iter().for_each(|pair| {
            let (exchange, currency_pairs) = pair;
            currency_pairs.iter().for_each(|pair| {
                all_targets2.insert(ExchangeIDCurrencyIDPair {
                    exchange: *exchange,
                    currency: pair.first,
                });
                all_targets2.insert(ExchangeIDCurrencyIDPair {
                    exchange: *exchange,
                    currency: pair.second,
                });
            });
        });
        let mut all_targets: Vec<_> = all_targets2.into_iter().collect();
        all_targets.sort();
        all_targets
    };
    let targets_count = all_targets.len();
    let pb = ProgressBar::new(targets_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );
    let processed_count = Arc::new(AtomicUsize::new(0usize));
    let processed_count_c = processed_count.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_c = stop.clone();
    let progress_thread = thread::spawn(move || {
        let mut last_count = 0usize;
        loop {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            thread::sleep(Duration::from_millis(100));
            let new_count = processed_count.load(Ordering::Relaxed);
            if last_count == new_count {
                continue;
            }
            let inc_times = new_count - last_count;
            pb.inc(inc_times as u64);
            let left_targets = targets_count - new_count;
            let percentage = ((new_count) as f64 * 100f64) / (targets_count as f64);
            pb.set_message(&format!(
                "targets left #{}\t {:.2}%",
                left_targets, percentage
            ));
            last_count = new_count;
            if new_count == targets_count {
                return;
            }
        }
    });

    // create a network that we will search from
    let net = {
        let mut n = Network::new();
        lexicon_f.exchange_currency_pairs_iter().for_each(|pair| {
            let (exchange, currency_pairs) = pair;
            let currency_pairs_vec: Vec<CurrencyIDPair> =
                currency_pairs.iter().map(|p| *p).collect();
            n.add_pairs(*exchange, &currency_pairs_vec);
        });
        n.update_exchange_hubs();
        n
    };

    let chunk_size = max_chunk_size.unwrap_or(all_targets.len());
    let path_root_chunks = Path::new(&out_file_path).join("paths");
    for chunk in all_targets.chunks(chunk_size) {
        let targets: HashSet<_> = chunk.iter().map(|t| *t).collect();
        let (sender, receiver) = std::sync::mpsc::channel();
        net.search_targets_channel(targets, &search_stop_settings, processed_count_c.clone(), sender);    
        for r in receiver.into_iter() {
            let (id, target_paths) = r;
            let file_name = format!("optimized_paths_e-{}_c-{}", id.exchange.0, id.currency.0);
            let out_file_path_chunk = path_root_chunks.join(&file_name);
            if out_file_path_chunk.exists() {
                let skip_count = processed_count_c.load(Ordering::Relaxed) + 1usize;
                processed_count_c.store(skip_count, Ordering::Relaxed);
                continue;
            }

            let unknown_paths: Vec<_> = target_paths.into_iter().collect();
            let (tr_7_paths, unknown_paths): (Vec<ArbitragePath7Nodes>, Vec<ArbitragePath>) = unknown_paths.into_iter().map(|n_path| n_path.try_into()).partition_result();
            let (tr_11_paths, unknown_paths): (Vec<_>, Vec<ArbitragePath>) = unknown_paths.into_iter().map(|n_path| n_path.try_into()).partition_result();
            let (tx_only_3pairs_paths, unknown_paths): (Vec<_>, Vec<ArbitragePath>) = unknown_paths.into_iter().map(|n_path| n_path.try_into()).partition_result();
            let (tx_only_5pairs_paths, unknown_paths): (Vec<_>, Vec<ArbitragePath>) = unknown_paths.into_iter().map(|n_path| n_path.try_into()).partition_result();

            let tx_only_3pairs_paths = merge_reversed_paths(tx_only_3pairs_paths);
            let tx_only_5pairs_paths = merge_reversed_paths(tx_only_5pairs_paths);

            let optimized_paths = OptimizedPreCalcedPaths {
                id,
                tr_7_paths: tr_7_paths.into(),
                tr_11_paths: tr_11_paths.into(),
                tx_only_3pairs_paths: tx_only_3pairs_paths.into(),
                tx_only_5pairs_paths: tx_only_5pairs_paths.into(),
                unknown_paths: unknown_paths.into(),
            };
            file_utils::save_cbor_lz4_file(&out_file_path_chunk, &optimized_paths)?;
        }
    }
    
    stop_c.store(true, Ordering::SeqCst);
    progress_thread.join().unwrap();

    Ok(())
}


fn print_network_stats(working_dir_root_path: &Path) -> anyhow::Result<()> {
    file_utils::file_must_exist(&working_dir_root_path)?;
    println!("Stats for network paths '{}'", &working_dir_root_path.display());

    let paths = fs::read_dir(&working_dir_root_path)?;
    let mut sum = OptimizedPreCalcedPathsStats::default();
    for p in paths {
        let p = p?;

        let optimized_paths: OptimizedPreCalcedPaths = file_utils::read_cbor_lz4_file(&p.path())?;
        let stat = optimized_paths.stats();
        println!("{}", p.path().display());
        println!("id {:?}", &optimized_paths.id);
        println!("{:?}", stat);
        println!("");
        println!("");

        sum.estimated_size_in_bytes += stat.estimated_size_in_bytes;
        sum.tr_7_paths += stat.tr_7_paths;
        sum.tr_11_paths += stat.tr_11_paths;
        sum.tx_only_3pairs_paths += stat.tx_only_3pairs_paths;
        sum.tx_only_5pairs_paths += stat.tx_only_5pairs_paths;
        sum.unknown_paths += stat.unknown_paths;
        
    }

    println!("SUM:");
    println!("{:?}", sum);

    Ok(())
}

fn merge_into_network_for_path(working_dir_root_path: &Path) -> anyhow::Result<()> {
    file_utils::file_must_exist(&working_dir_root_path)?;
    println!("Merge network chunks for '{}'", &working_dir_root_path.display());

    let path_root = Path::new(&working_dir_root_path);
    let path_root_lexicon = path_root.join("lexicon.lex");
    let path_root_search_stop_settings = path_root.join("search_stop_settings.json");
    let lexicon_f = lexicon::read_lexicon_file(&path_root_lexicon)?;

    let search_stop_settings = read_search_stop_settings(&path_root_search_stop_settings)?;

    let mut merge_pre_calced_paths: BTreeMap<BalanceExchangeCurrencyInfo, OptimizedPreCalcedPaths> =
        BTreeMap::new();
    let path_root_chunks = Path::new(&working_dir_root_path).join("paths");
    let paths = fs::read_dir(&path_root_chunks)?;
    for p in paths {
        let p = p?;
        let optimized_paths: OptimizedPreCalcedPaths = file_utils::read_cbor_lz4_file(&p.path())?;
        merge_pre_calced_paths.insert(optimized_paths.id, optimized_paths);
    }
    
    let lexicon_network_paths = Path::new(&working_dir_root_path).join("lexicon_network_paths.net");
    let network_with_lexicon = OptimizedNetworkWithLexicon {
        lexicon: lexicon_f,
        pre_calced_paths: merge_pre_calced_paths,
        search_stop_settings: search_stop_settings,
    };
    file_utils::save_cbor_lz4_file(&lexicon_network_paths, &network_with_lexicon)?;

    Ok(())
}

fn print_all_network_paths(network_lexicon_path: &Path) -> anyhow::Result<()> {
    let (lexicon, pre_calced_paths) = {
        
        let network_with_lexicon: OptimizedNetworkWithLexicon = file_utils::read_cbor_lz4_file(&network_lexicon_path)?;
        (
            network_with_lexicon.lexicon,
            network_with_lexicon.pre_calced_paths,
        )
    };
    // TODO ADD FILTERS!!! MORE OPTIONS!!!
    pre_calced_paths.into_iter().for_each(|pair| {

        let (target, optimized_paths) = pair;
        let tr_7_paths: BTreeSet<_> = optimized_paths.tr_7_paths.into_iter().flatten().collect();
        tr_7_paths.into_iter().for_each(|path| {
            path.print_path_all(&lexicon);
        });

        let tr_11_paths: BTreeSet<_> = optimized_paths.tr_11_paths.into_iter().flatten().collect();
        tr_11_paths.into_iter().for_each(|path| {
            path.print_path_all(&lexicon);
        });

        let tx_only_3pairs_paths: BTreeSet<_> = optimized_paths.tx_only_3pairs_paths.into_iter().flatten().collect();
        tx_only_3pairs_paths.into_iter().for_each(|path| {
            let (first, second) = interpolate_reversed_paths(target.exchange, target.currency, &path.0);
            first.print_path_all(&lexicon);
            println!("--");
            second.print_path_all(&lexicon);
        });

        let tx_only_5pairs_paths: BTreeSet<_> = optimized_paths.tx_only_5pairs_paths.into_iter().flatten().collect();
        tx_only_5pairs_paths.into_iter().for_each(|path| {
            let (first, second) = interpolate_reversed_paths(target.exchange, target.currency, &path.0);
            first.print_path_all(&lexicon);
            println!("--");
            second.print_path_all(&lexicon);
        });

        let unknown_paths: BTreeSet<_> = optimized_paths.unknown_paths.into_iter().flatten().collect();
        unknown_paths.into_iter().for_each(|path| {
            path.print_path_all(&lexicon);
        });
    });

    Ok(())
}

// TODO add benchmarks case
// TODO use this for benchmarking
pub fn exchange_domain_currency_pairs_generation_exaustive(
    max_currencies: u16,
) -> Vec<CurrencyIDPair> {
    let c1_range = 0..max_currencies;
    let c2_range = 0..max_currencies;
    let mut exchange_pairs: HashSet<CurrencyIDPair> = HashSet::new();
    for c1 in c1_range.clone() {
        for c2 in c2_range.clone() {
            if c1 == c2 {
                continue;
            }
            exchange_pairs.insert(CurrencyIDPair {
                first: CurrencyID(c1),
                second: CurrencyID(c2),
            });
        }
    }
    let exchange_pairs: Vec<_> = exchange_pairs.into_iter().map(|pair| pair).collect();
    exchange_pairs
}

pub fn exchange_domain_currency_pairs_generation_representative(
    max_currencies: u16,
) -> Vec<CurrencyIDPair> {
    let c1_range = 0..max_currencies / 2;
    let c2_range = max_currencies / 2..max_currencies;
    let mut exchange_pairs: HashSet<CurrencyIDPair> = HashSet::new();
    for c1 in c1_range.clone() {
        for c2 in c2_range.clone() {
            if c1 == c2 {
                continue;
            }
            exchange_pairs.insert(CurrencyIDPair {
                first: CurrencyID(c1),
                second: CurrencyID(c2),
            });
        }
    }
    let exchange_pairs: Vec<_> = exchange_pairs.into_iter().map(|pair| pair).collect();
    exchange_pairs
}

// BENCHMARK TIME TO OPERATE
// BENCHMARK ACTIVE RAM USAGE
// BENCHMARK COMPRESSED AND UNCOMPRESSED RAM/FILE USAGE

// fn main() {
//     // let max_exchanges = 50;
//     // let max_currencies = 70;
//     let max_exchanges = 10;
//     let max_currencies = 20;

//     let e_range = 0..max_exchanges;
//     let mut n = Network::new();
//     let pairs_markets = exchange_domain_currency_pairs_generation_representative(max_currencies);
//     // let pairs_markets = exchange_domain_currency_pairs_generation_exaustive(max_currencies);
//     for e in e_range.clone() {
//         n.add_pairs(ExchangeID(e), &pairs_markets);
//     }
//     println!("Main test run! STAR");
//     // move back to self
//     let n = n.search_all_possible_targets(SearchTargetsFilterType::Level4);
//     let print = true;
//     if print {
//         n.pre_calced_paths_iter().for_each(|pair| {
//             let (_, kp) = pair;
//             kp.known_paths.iter().for_each(|pair2| {
//                 let (id, path) = pair2;
//                 println!("{}", id);
//                 let json_objs: Vec<_> = path.iter().map(|op| op.to_json()).collect();
//                 println!("{}", format!("[{}]", json_objs.join(",")));
//                 println!();
//             });
//         });
//         let cbor_buf = crypto_serializer::cbor_to_vec(&n).unwrap();
//         let cbor_buf_lz4 = crypto_serializer::compress_lz4(&cbor_buf);
//         let cbor_buf_lz4_2 = crypto_serializer::compress_lz4(&cbor_buf_lz4);
//         let cbor_buf_lz4_3 = crypto_serializer::compress_lz4(&cbor_buf_lz4_2);
//         let cbor_buf_lz4_4 = crypto_serializer::compress_lz4(&cbor_buf_lz4_3);
//         to_file("cbor_buf", &cbor_buf);
//         to_file("cbor_buf_lz4", &cbor_buf_lz4);
//         to_file("cbor_buf_lz4_2", &cbor_buf_lz4_2);
//         to_file("cbor_buf_lz4_3", &cbor_buf_lz4_3);
//         to_file("cbor_buf_lz4_4", &cbor_buf_lz4_4);
//     } else {
//         println!("Skip print");
//     }
//     println!("Main test run! END");
// }
