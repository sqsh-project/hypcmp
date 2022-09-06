use clap::Parser;
use std::path::PathBuf;

/// Command-line Interface (CLI) for the hypcmp library
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Configuration file [*.toml]
    #[clap(value_parser)]
    pub config: PathBuf,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}
