use std::path::PathBuf;
use clap::Parser;

// // WHAT the utility does???
//   - Generate dictionary FOR network generation,
//   - compare networks, dictionaries
//   - check exchanges inverse pairs (THIS indicates an error)
//   - generate network from dictionary and set search settings
//   - list network properties dictionary, paths, search targets
//   - resume network search
//   - read network and print paths as RAW or human readable mapped from ,

/// A command line tool for creating and reading crypto lexicon and network files.  
// #[derive(Debug, Clone, Parser)]
#[derive(Parser)]
#[clap(version = "0.1", author = "S74nk0")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Uncompress generated lexicon and network files.
    #[clap(name = "uncompress", version = "0.1")]
    Uncompress(Uncompress),
    #[clap(name = "lexicon", version = "0.1")]
    Lexicon(Lexicon),
    #[clap(name = "network", version = "0.1")]
    Network(Network),
}

#[derive(Parser)]
pub struct Uncompress {
    #[clap(short = 'i', long = "input-file")]
    pub in_file_path: PathBuf,
    #[clap(short = 'o', long = "output-file")]
    pub out_file_path: PathBuf,
}

/// A subcommand for Lexicon operations
#[derive(Parser)]
struct Lexicon {
    #[clap(subcommand)]
    subcmd: LexiconCommand,
}

#[derive(Parser)]
pub enum LexiconCommand {
    /// Generate a Lexicon file from 'exchanges-currency-pairs JSON definition file'
    #[clap(name = "generate")]
    Generate(LexiconInOutFile),

    #[command(flatten)]
    Print(PrintLexiconCommand),

    // // TODO add COMPARE (exact match, full domain but different id match)
    // // TODO add has exchange
    // // TODO add exchange has currency exchange has pair
    // // TODO add print fiat currencies
    // // TODO add print stable coins
    // // TODO add checksum for exchange_pairs
}

#[derive(Parser)]
pub enum PrintLexiconCommand {
    /// Verify a Lexicon file that there are not any invalid exchange currency pair items/markets.
    /// E.g. equal pairs or inverse pairs on the same exchange indicate a bad 'exchanges-currency-pairs JSON definition file'
    #[clap(name = "verify")]
    Verify{ in_file_path: PathBuf },
    /// Prints all currencies IDs and names for a given Lexicon file
    #[clap(name = "print-all-currencies")]
    PrintAllCurrencies(LexiconPrintPath),
    /// Prints all exchanges currency pairs for a given Lexicon file
    #[clap(name = "print-all-exchanges-pairs")]
    PrintAllExchangesPairs(LexiconPrintPath),
    /// Prints all exchanges for a given Lexicon file
    #[clap(name = "print-exchanges")]
    PrintExchanges(LexiconPrintPath),
    /// Prints all currency pairs just for the selected exchange for a given Lexicon file
    #[clap(name = "print-exchange-pairs")]
    PrintExchangePairs(LexiconPrintExchangePairsInFile), // TODO ADD JSON version as well
}

#[derive(Parser)]
pub struct LexiconInOutFile {
    // #[clap(short = 'i', long = "input-file")]
    pub in_file_path: PathBuf,
    // #[clap(short = 'o', long = "output-file")]
    pub out_file_path: PathBuf,
}

#[derive(Parser)]
pub struct LexiconPrintExchangePairsInFile {
    pub in_file_path: PathBuf,
    #[clap(short = 'e', long = "exchange")]
    pub exchange: String,
}

#[derive(Parser)]
pub struct LexiconPrintPath {
    pub in_file_path: PathBuf,
    #[clap(short = 'j', long = "json")]
    pub json: bool,
}

/// A subcommand for Network operations
#[derive(Parser)]
struct Network {
    #[clap(subcommand)]
    subcmd: NetworkCommand,
}
#[derive(Parser)]
pub enum NetworkCommand {
    // TODO maybe don't recylce lexicon commands here
    // #[clap(name = "lexicon")]
    // Lexicon(LexiconCommand),
    /// Generate a Network file for a given Lexicon input.
    /// This is used to search up all possible exchange paths.
    /// We provide optional search parameters such as MAX_LEVEL, IGNORE_CYLCELS and MAX_TRANSFERS.
    /// The Network generation could be split up in multiple files for deeply nested networks
    /// to reduce RAM usage and make the search possible on systems with lower resources.
    #[clap(name = "generate")]
    Generate(GenerateNetworkInOutFile),
    #[clap(name = "resume")]
    Resume {
        path: PathBuf,

        /// If provided it will split work into chunks. With this one can limit how many resources should be used.
        /// If option is not provided it will use all threads.
        #[clap(long = "chunk-size")]
        max_chunk_size: Option<usize>,
    },
    #[clap(name = "merge-into-network")]
    MergeIntoNetwork { path: PathBuf },
    #[clap(name = "print-all-paths")]
    PrintAllPaths { path: PathBuf },

    #[clap(name = "print-stats")]
    PrintStatistics { path: PathBuf },
}

#[derive(Parser)]
pub struct GenerateNetworkInOutFile {
    /// Path to the input Lexicon file.
    // #[clap(short = 'i', long = "input-file")]
    pub in_file_lexicon_path: PathBuf,
    /// Output path for where to generate the network. In case of a multiple file generation
    /// the otuput path could be directory instead of a single file.
    /// In case of a directory output we should be able to resume last search state using
    /// the resume command
    // #[clap(short = 'o', long = "output-path")]
    pub output_dir_path: PathBuf,
    /// Max level/depth. Setting this to a higher number could increase search times significantly
    #[clap(short = 'l', long = "max-level", default_value = "4")]
    pub max_level: i32,
    /// Allow cycles is used to disable the default 'Ignore cycles' for ignoring paths that have same path states between the first and final node.
    /// This flag should almost always be omitted.
    #[clap(short = 'c', long = "allow-cycles")]
    pub allow_cycles: bool,
    /// Ignore cycles is used for ignoring paths that have same path states between the first and final node.
    /// The default value 'true' should almost always be prefered.
    #[clap(short = 't', long = "max-transfers", default_value = "3")]
    pub max_transfers: i32,

    /// If provided it will split work into chunks. With this one can limit how many resources should be used.
    /// If option is not provided it will use all threads.
    #[clap(long = "chunk-size")]
    pub max_chunk_size: Option<usize>,


    // TODO ADD SYNC AND PARALLEL MODE
}

pub enum ParsedArgs {
    LexiconCommand(LexiconCommand),
    NetworkCommand(NetworkCommand),
    Uncompress(Uncompress),
}

pub fn parse() -> ParsedArgs {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Lexicon(lexicon) => ParsedArgs::LexiconCommand(lexicon.subcmd),
        SubCommand::Network(network) => ParsedArgs::NetworkCommand(network.subcmd),
        SubCommand::Uncompress(u) => ParsedArgs::Uncompress(u),
    }
}
