use clap::Parser;

use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    /// The path to search in
    pub path: PathBuf,
}
