use crate::id_types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use string_to_int_mapper::*;
#[derive(Serialize, Deserialize)]
pub struct ExchangeSymbolsJson {
    pub exchange: String,
    pub symbols: Vec<String>, // symbols are split by '/'
}
// struct SymbolPair(String, String);

// TODO split the lexicon to fundamental and non-fundamental part.
// The fundamental part defines the parts of the lexicon that defines the ('Consensus') part.
// The consensus part is used for network generation
// non-fundamental part should hold the list of fiat, stable coins, maybe exchange fees, transfer and withrad limits
// (what can be transfered to and withdrawed from the exchanges hence there are limits)
// again look at the cryptowatch definitions and their API. It will serve as a good example for our case.

// lexicon should contain fiats list, depricated list, exchanges and currencies
#[derive(Serialize, Deserialize)]
pub struct CryptoExchangeLexicon {
    pub exchanges: StringToIntMapper<ExchangeID, Reading>,
    pub currencies: StringToIntMapper<CurrencyID, Reading>,
    pub exchange_currency_pairs: BTreeMap<ExchangeID, BTreeSet<CurrencyIDPair>>,
    pub fiat_currencies: BTreeSet<CurrencyID>,
    pub stable_currencies: BTreeSet<CurrencyID>,
}

// Vec<exchange_symbols>
impl CryptoExchangeLexicon {
    pub fn create_from_exchange_symbols(ex_symbols: &[ExchangeSymbolsJson]) -> Self {
        let fiat_currencies = Vec::new();
        let stable_currencies = Vec::new();
        Self::create_from_exchange_symbols_full(&ex_symbols, &fiat_currencies, &stable_currencies)
    }
    pub fn create_from_exchange_symbols_full(
        ex_symbols: &[ExchangeSymbolsJson],
        fiat_currencies: &[&str],
        stable_currencies: &[&str],
    ) -> Self {
        let mut exchanges = StringToIntMapper::<ExchangeID, Editing>::new();
        let mut currencies = StringToIntMapper::<CurrencyID, Editing>::new();
        // populate exchanges and currencies
        ex_symbols.iter().for_each(|exchange_symbols| {
            exchanges.add(&exchange_symbols.exchange);
            exchange_symbols
                .symbols
                .iter()
                .filter(|symbol| symbol.contains("/"))
                .for_each(|symbol_pair| {
                    let pair: Vec<_> = symbol_pair.split("/").collect();

                    if pair.len() != 2 {
                        println!("symbol more than 2 {}", symbol_pair);
                        return;
                    }
                    pair.iter().for_each(|c| {
                        currencies.add(c);
                    });
                })
        });
        let exchanges = exchanges.to_reader();
        let currencies = currencies.to_reader();
        let exchange_currency_pairs: BTreeMap<ExchangeID, BTreeSet<CurrencyIDPair>> = ex_symbols
            .iter()
            .map(|exchange_symbols| {
                let exchange = exchanges.get_id(&exchange_symbols.exchange).unwrap();
                let exchange = *exchange;
                let pairs: BTreeSet<_> = exchange_symbols
                    .symbols
                    .iter()
                    .filter(|symbol| symbol.contains("/"))
                    .map(|symbol_pair| {
                        let pair: Vec<_> = symbol_pair.split("/").collect();
                        pair
                    })
                    .filter(|pair| pair.len() == 2)
                    .map(|pair| {
                        let c1 = currencies.get_id(pair[0]).unwrap();
                        let c2 = currencies.get_id(pair[1]).unwrap();
                        let c1 = *c1;
                        let c2 = *c2;
                        CurrencyIDPair {
                            first: c1,
                            second: c2,
                        }
                    })
                    .collect();
                (exchange, pairs)
            })
            .collect();

        let fiat_currencies: BTreeSet<CurrencyID> = fiat_currencies
            .iter()
            .map(|fiat| currencies.get_id(fiat).unwrap())
            .map(|c| *c)
            .collect();
        let stable_currencies: BTreeSet<CurrencyID> = stable_currencies
            .iter()
            .map(|c| currencies.get_id(c).unwrap())
            .map(|c| *c)
            .collect();
        CryptoExchangeLexicon {
            exchanges: exchanges,
            currencies: currencies,
            exchange_currency_pairs: exchange_currency_pairs,
            fiat_currencies: fiat_currencies,
            stable_currencies,
        }
    }

    pub fn exchange_to_string(&self, e: &ExchangeID) -> &str {
        if let Some(name) = self.exchanges.get_key(e) {
            return name;
        }
        "N/A"
    }

    pub fn currency_to_string(&self, c: &CurrencyID) -> &str {
        if let Some(name) = self.currencies.get_key(c) {
            return name;
        }
        "N/A"
    }

    pub fn currency_pair_to_string(&self, cp: &CurrencyIDPair) -> String {
        format!(
            "{} : {}",
            &self.currency_to_string(&cp.first),
            &self.currency_to_string(&cp.second)
        )
    }

    pub fn get_currency_pair_strings(&self, cp: &CurrencyIDPair) -> Option<(&str, &str)> {
        self.currencies.get_key(&cp.first).zip(self.currencies.get_key(&cp.second))
    }

    pub fn print_all_exchanges_currencies(&self) {
        self.exchange_currency_pairs
            .iter()
            .for_each(|exchange_currency_pair| {
                let (exchange, currency_pairs) = exchange_currency_pair;
                let exchange_name = self.exchange_to_string(&exchange);
                println!("Exchange id '{}' name '{}'", exchange.0, exchange_name);
                currency_pairs.iter().for_each(|currency_pair| {
                    let currency_pair_name = self.currency_pair_to_string(&currency_pair);
                    println!(
                        "\tcurrency pair '{} : {}'\tname '{}'",
                        currency_pair.first.0, currency_pair.second.0, currency_pair_name
                    )
                });
            });
    }

    pub fn print_all_exchanges_currencies_in_json_format(&self) {
        let map: HashMap<String, Vec<String>> = self.exchange_currency_pairs
            .iter()
            .map(|exchange_currency_pair| {
                let (exchange, currency_pairs) = exchange_currency_pair;
                let exchange_name = self.exchange_to_string(&exchange);
                let pairs: Vec<String> = currency_pairs.iter().map(|currency_pair| {
                    let currency_pair_name = self.currency_pair_to_string(&currency_pair);
                    currency_pair_name.replace(" : ", "-")
                }).collect();
                (exchange_name.to_string(), pairs)
            }).collect();
        println!("{}", serde_json::to_string_pretty(&map).unwrap());
    }

    pub fn print_all_currencies(&self) {
        self.currencies.iter_in_order().for_each(|c_name| {
            let c = self.currencies.get_id(&c_name).unwrap();
            println!("Currency id '{}' name '{}'", c, c_name);
        });
    }

    pub fn print_all_currencies_in_json_format(&self) {
        let currencies: Vec<String> = self.currencies.iter_in_order().map(|c_name| c_name.to_string()).collect();
        println!("{}", serde_json::to_string_pretty(&currencies).unwrap());
    }
    
    pub fn print_exchanges(&self) {
        self.exchange_currency_pairs
            .iter()
            .for_each(|exchange_currency_pair| {
                let (exchange, _) = exchange_currency_pair;
                let exchange_name = self.exchange_to_string(&exchange);
                println!("Exchange id '{}' name '{}'", exchange.0, exchange_name);
            });
    }

    pub fn print_exchanges_in_json_format(&self) {
        let exchanges: Vec<String> = self.exchange_currency_pairs
            .iter()
            .map(|exchange_currency_pair| {
                let (exchange, _) = exchange_currency_pair;
                let exchange_name = self.exchange_to_string(&exchange);
                exchange_name.to_string()
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&exchanges).unwrap());
    }

    pub fn print_exchange_pairs(&self, exchange: &str) {
        let exchange_id_op = self.exchanges.get_id(&exchange);
        match exchange_id_op {
            Some(exchange_id) => {
                // let exchange_id = ExchangeID(exchange_id as u8);
                if let Some(currency_pairs) = self.exchange_currency_pairs.get(&exchange_id) {
                    let exchange_name = self.exchange_to_string(&exchange_id);
                    println!("Exchange id '{}' name '{}'", exchange_id.0, exchange_name);
                    currency_pairs.iter().for_each(|currency_pair| {
                        let currency_pair_name = self.currency_pair_to_string(&currency_pair);
                        println!(
                            "\tcurrency pair '{} : {}'\tname '{}'",
                            currency_pair.first.0, currency_pair.second.0, currency_pair_name
                        )
                    });
                } else {
                    println!("No exchange {} found!", exchange);
                }
            }
            None => println!("No exchange {} found!", exchange),
        }
    }

    fn get_inverse_pairs(pairs: &BTreeSet<CurrencyIDPair>) -> BTreeSet<&CurrencyIDPair> {
        let inverse_pairs: BTreeSet<&CurrencyIDPair> = pairs.iter().enumerate().flat_map(|pair| {
            let (index_at, pair_a) = pair;
            let mut inverse_b_pairs: BTreeSet<_> = pairs.iter().skip(index_at).filter(|pair_b| CurrencyIDPair::is_inverse(pair_a, pair_b)).collect();
            if !inverse_b_pairs.is_empty() {
                inverse_b_pairs.insert(pair_a);
            }
            inverse_b_pairs
        }).collect();
        inverse_pairs
    }

    pub fn verify_exchange_currency_pairs(&self) -> bool {
        let mut ok = true;
        self.exchange_currency_pairs
            .iter()
            .for_each(|exchange_currency_pair| {
                let (exchange, pairs) = exchange_currency_pair;
                let bad_pairs: Vec<_> = pairs
                    .iter()
                    .filter(|pair| pair.has_same_currencies())
                    .collect();
                let inverse_pairs = Self::get_inverse_pairs(pairs);

                if !bad_pairs.is_empty() || !inverse_pairs.is_empty() {
                    ok = false;
                    let exchange_name = self.exchange_to_string(&exchange);
                    println!("Exchange id '{}' name '{}'", exchange.0, exchange_name);
                    bad_pairs.iter().for_each(|currency_pair| {
                        let currency_pair_name = self.currency_pair_to_string(&currency_pair);
                        println!(
                            "\tBad currency pair '{} : {}'\tname '{}'",
                            currency_pair.first.0, currency_pair.second.0, currency_pair_name
                        )
                    });
                    inverse_pairs.iter().for_each(|currency_pair| {
                        let currency_pair_name = self.currency_pair_to_string(&currency_pair);
                        println!(
                            "\tInverse currency pair '{} : {}'\tname '{}'",
                            currency_pair.first.0, currency_pair.second.0, currency_pair_name
                        )
                    });
                }
            });
        ok
    }

    pub fn exchange_currency_pairs_iter(
        &self,
    ) -> std::collections::btree_map::Iter<'_, ExchangeID, BTreeSet<CurrencyIDPair>> {
        self.exchange_currency_pairs.iter()
    }
}
