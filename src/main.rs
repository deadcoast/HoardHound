mod classify;
mod cli;
mod config;
mod filters;
mod human;
mod model;
mod report;
mod roots;
mod scanner;
mod vdf;
mod windows;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use config::ScanConfig;
use std::fs;

fn main() {
    if let Err(err) = run() {
        eprintln!("hoardhound error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let color = !cli.no_color;

    match cli.command {
        Commands::Scan(args) => {
            let profile = args.profile.into();
            let min_size_bytes = human::parse_size(&args.min_size)
                .with_context(|| format!("parsing --min-size {}", args.min_size))?;
            let mut config = ScanConfig::conservative_default(profile, args.top, min_size_bytes);
            config.follow_links = args.follow_links;
            config.include_program_files = args.include_program_files;
            config.show_skipped = args.show_skipped;
            config.candidate_depth = args.candidate_depth;

            let roots = roots::discover_roots(&config)?;
            if roots.is_empty() {
                println!("No existing scan roots found for profile '{}'. Try `hoardhound roots --profile {}` to preview discovery.", profile.as_str(), profile.as_str());
                return Ok(());
            }

            if !cli.quiet {
                println!(
                    "Scanning {} targeted roots with minimum report size {}...",
                    roots.len(),
                    human::format_size(min_size_bytes)
                );
            }

            let report = scanner::scan(&config, &roots)?;

            let last_report = windows::last_report_path()?;
            report::write_json(&report, &last_report)?;

            report::print_report(&report, color, args.show_skipped);

            if let Some(path) = args.json {
                report::write_json(&report, &path)?;
                println!("JSON report written to {}", path.display());
            }
            if let Some(path) = args.markdown {
                report::write_markdown(&report, &path)?;
                println!("Markdown report written to {}", path.display());
            }
        }
        Commands::Roots(args) => {
            let profile = args.profile.into();
            let mut config =
                ScanConfig::conservative_default(profile, 30, human::parse_size("1GB")?);
            config.include_program_files = args.include_program_files;
            let roots = roots::discover_roots(&config)?;
            report::print_roots(&roots, color);
        }
        Commands::Open(args) => {
            let path = windows::last_report_path()?;
            let text = fs::read_to_string(&path)
                .with_context(|| format!("reading last scan report from {}", path.display()))?;
            let report: model::ScanReport = serde_json::from_str(&text)
                .with_context(|| format!("parsing last scan report from {}", path.display()))?;
            let Some(entry) = report.entries.iter().find(|entry| entry.id == args.id) else {
                anyhow::bail!("result ID {} not found in last scan", args.id);
            };
            println!("Opening {}", entry.path.display());
            windows::open_in_explorer(&entry.path)?;
        }
        Commands::Explain(args) => report::print_explain(args.verbose),
    }

    Ok(())
}
