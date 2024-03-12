mod args_parser;
mod file_utils;
mod lexicon;
mod network;

use std::path::Path;

use colored::*;

// TODO in main use panic catch unwind
// TODO measure every operation time here as well
// TODO add ctrl+c gracefull shutdown

fn main() {
    let result = match args_parser::parse() {
        args_parser::ParsedArgs::LexiconCommand(lexicon_command) => {
            lexicon::handle_lexicon_command(lexicon_command)
        }
        args_parser::ParsedArgs::NetworkCommand(network_command) => {
            network::handle_network_command(network_command)
        }
        args_parser::ParsedArgs::Uncompress(uncompress_command) => {
            handle_uncompress_command(
                &uncompress_command.in_file_path,
                &uncompress_command.out_file_path,
            )
        }
    };
    match result {
        Ok(_) => println!("{}", "Success!".green().bold()),
        Err(err) => println!("{}: {:?}", "Error!".red().bold(), err)
    }
}

fn handle_uncompress_command(in_file_path: &Path, out_file_path: &Path) -> anyhow::Result<()> {
    file_utils::file_must_exist(&in_file_path)?;
    file_utils::file_must_not_exist(&out_file_path)?;
    println!(
        "Uncompressing file '{}' to '{}'",
        &in_file_path.display(), &out_file_path.display()
    );
    let compressed_bytes = file_utils::read_from_file(&in_file_path)?;
    let decompressed = crypto_serializer::decompress_lz4(&compressed_bytes)?;
    file_utils::write_to_file(&out_file_path, &decompressed)?;
    Ok(())
}
