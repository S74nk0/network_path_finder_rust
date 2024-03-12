use std::path::Path;

use crate::args_parser;
use crate::file_utils;
use ::crypto_exchange_types::{CryptoExchangeLexicon, ExchangeSymbolsJson};
use colored::*;
use args_parser::{LexiconCommand, PrintLexiconCommand};

pub fn handle_lexicon_command(lexicon_command: args_parser::LexiconCommand) -> anyhow::Result<()> {
    match lexicon_command {
        LexiconCommand::Generate(opts) => {
            generate_lexicon_file(&opts.in_file_path, &opts.out_file_path)
        }
        LexiconCommand::Print(r) => print_lexicon_file(r),
    }
}

pub fn save_lexicon_file(file_path: &Path, lexicon: &CryptoExchangeLexicon) -> anyhow::Result<()> {
    file_utils::save_cbor_lz4_file(&file_path, &lexicon)
}

pub fn read_lexicon_file(file_path: &Path) -> anyhow::Result<CryptoExchangeLexicon> {
    file_utils::read_cbor_lz4_file(&file_path)
}

fn generate_lexicon_file(in_file_path: &Path, out_file_path: &Path) -> anyhow::Result<()> {
    file_utils::file_must_exist(&in_file_path)?;
    file_utils::file_must_not_exist(&out_file_path)?;
    println!(
        "Generating lexicon file '{}' from '{}'",
        &in_file_path.display(), &out_file_path.display()
    );

    let json_bytes = file_utils::read_from_file(&in_file_path)?;
    let exchanges_pairs: Vec<ExchangeSymbolsJson> = serde_json::from_slice(&json_bytes)?;
    let lexicon = CryptoExchangeLexicon::create_from_exchange_symbols(&exchanges_pairs);
    // lexicon.print_all();
    // lexicon.print_exchanges();
    save_lexicon_file(&out_file_path, &lexicon)
}

fn read_lexicon_file_from_command(read_command: &PrintLexiconCommand) -> anyhow::Result<CryptoExchangeLexicon> {
    use PrintLexiconCommand::*;
    let in_file_path = match read_command {
        Verify { in_file_path } => in_file_path,
        PrintAllCurrencies(o) => &o.in_file_path,
        PrintAllExchangesPairs(o) => &o.in_file_path,
        PrintExchanges(o) => &o.in_file_path,
        PrintExchangePairs(o) => &o.in_file_path,
    };
    file_utils::file_must_exist(&in_file_path)?;
    Ok(read_lexicon_file(&in_file_path)?)
}

fn print_lexicon_file(print_command: PrintLexiconCommand) -> anyhow::Result<()> {
    let lexicon = read_lexicon_file_from_command(&print_command)?;
    match print_command {
        PrintLexiconCommand::Verify { .. } => {
            let is_valid = lexicon.verify_exchange_currency_pairs();
            if !is_valid {
                println!("{}", "Lexicon has invalid exchange pairs".red().bold());
            }
        },
        PrintLexiconCommand::PrintAllCurrencies(opt) => {
            if opt.json {
                lexicon.print_all_currencies_in_json_format()
            }
            else {
                lexicon.print_all_currencies()
            }
        },
        PrintLexiconCommand::PrintAllExchangesPairs(opt) => {
            if opt.json {
                lexicon.print_all_exchanges_currencies_in_json_format()
            }
            else {
                lexicon.print_all_exchanges_currencies()
            }
        },
        PrintLexiconCommand::PrintExchanges(opt) => {
            if opt.json {
                lexicon.print_exchanges_in_json_format()
            }
            else {
                lexicon.print_exchanges()
            }
        },
        PrintLexiconCommand::PrintExchangePairs(exchange) => lexicon.print_exchange_pairs(&exchange.exchange),
    };
    Ok(())
}