use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "sebi",
    version,
    about = "Static execution-boundary inspection for Stylus WASM"
)]
pub struct Args {
    /// Path to the .wasm artifact
    pub wasm_path: PathBuf,

    /// Output format
    #[arg(long, default_value = "json")]
    pub format: OutputFormat,

    /// Write output to a file instead of stdout
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Optional git commit hash for tool metadata
    #[arg(long)]
    pub commit: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}
