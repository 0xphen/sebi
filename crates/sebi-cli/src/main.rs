use anyhow::Result;
use clap::Parser;

use sebi_core::inspect;
use sebi_core::report::{model::ToolInfo, render};

mod args;

fn main() -> Result<()> {
    let args = args::Args::parse();

    let tool = ToolInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit: args.commit.clone(),
    };

    let report = inspect(&args.wasm_path, tool)?;

    let output = match args.format {
        args::OutputFormat::Json => serde_json::to_string_pretty(&report)?,
        args::OutputFormat::Text => render::render_text(&report),
    };

    match args.out {
        Some(path) => std::fs::write(path, &output)?,
        None => print!("{output}"),
    }

    std::process::exit(report.classification.exit_code);
}
