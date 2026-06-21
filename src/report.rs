use crate::human::{format_size, truncate_middle};
use crate::model::{ResultEntry, ScanReport, ScanRoot};
use anyhow::{Context, Result};
use colored::{Color, Colorize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputStyle {
    color: bool,
    plain: bool,
}

impl OutputStyle {
    pub fn from_flags(color: bool, plain: bool) -> Self {
        Self {
            color: color && !plain,
            plain,
        }
    }

    fn color_enabled(self) -> bool {
        self.color
    }

    fn rule(self, width: usize) -> String {
        if self.plain {
            "-".repeat(width)
        } else {
            "─".repeat(width)
        }
    }
}

pub fn print_report(report: &ScanReport, style: OutputStyle, show_skipped: bool) {
    let title = "HoardHound";
    if style.color_enabled() {
        println!(
            "{} {}",
            title.bold().bright_blue(),
            "targeted storage report".bright_white()
        );
    } else {
        println!("{} targeted storage report", title);
    }

    println!(
        "Profile: {} | Roots: {} | Scanned: {} | Files: {} | Dirs: {} | Time: {:.2}s",
        report.profile.as_str(),
        report.stats.roots_scanned,
        format_size(report.stats.bytes_seen),
        report.stats.files_scanned,
        report.stats.dirs_scanned,
        report.stats.elapsed_ms as f64 / 1000.0,
    );

    if report.entries.is_empty() {
        println!(
            "\nNo folders met the minimum size threshold of {}.",
            format_size(report.min_size_bytes)
        );
        return;
    }

    println!("\n{}", style.rule(94));
    println!(
        "{:<4} {:>11}  {:<11} {:<5}  FOLDER",
        "ID", "SIZE", "CATEGORY", "CONF"
    );
    println!("{}", style.rule(94));

    for entry in &report.entries {
        print_entry(entry, style);
    }

    println!("{}", style.rule(94));
    println!("Open a result: hoardhound open <ID>");

    if show_skipped || report.stats.access_errors > 0 || report.stats.skipped_dirs > 0 {
        println!(
            "Skipped dirs: {} | Access errors: {}",
            report.stats.skipped_dirs, report.stats.access_errors
        );
    }

    if show_skipped && !report.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_entry(entry: &ResultEntry, style: OutputStyle) {
    let size = format_size(entry.size_bytes);
    let category = entry.category.label();
    let name = truncate_middle(&entry.display_name, 44);
    let path = truncate_middle(&entry.path.display().to_string(), 88);

    if style.color_enabled() {
        println!(
            "{:<4} {:>11}  {:<11} {:>3}%   {}",
            entry.id.to_string().bright_blue(),
            size.bold(),
            color_category(category).bold(),
            entry.confidence,
            name.bold()
        );
        println!("      {}", path.dimmed());
        println!("      {} {}", "why:".bright_black(), entry.reason);
        println!("      {} {}", "do:".bright_black(), entry.advice);
    } else {
        println!(
            "{:<4} {:>11}  {:<11} {:>3}%   {}",
            entry.id, size, category, entry.confidence, name
        );
        println!("      {}", path);
        println!("      why: {}", entry.reason);
        println!("      do: {}", entry.advice);
    }

    if !entry.largest_children.is_empty() {
        let children = entry
            .largest_children
            .iter()
            .map(|child| {
                format!(
                    "{} ({})",
                    truncate_middle(&child.name, 24),
                    format_size(child.size_bytes)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("      top children: {}", children);
    }
    println!();
}

fn color_category(label: &str) -> colored::ColoredString {
    let color = match label {
        "Games" => Color::BrightMagenta,
        "Mods" => Color::BrightCyan,
        "Cache" => Color::Yellow,
        "Downloads" => Color::BrightYellow,
        "Media" => Color::BrightGreen,
        "AppData" => Color::BrightBlue,
        "Dev" => Color::BrightRed,
        _ => Color::White,
    };
    label.color(color)
}

pub fn print_roots(roots: &[ScanRoot], style: OutputStyle) {
    if roots.is_empty() {
        println!("No matching scan roots exist on this machine for the selected profile.");
        return;
    }

    if style.color_enabled() {
        println!("{}", "Targeted scan roots".bold().bright_blue());
    } else {
        println!("Targeted scan roots");
    }

    for (index, root) in roots.iter().enumerate() {
        println!(
            "{:<3} {:<18} {}",
            index + 1,
            root.category_hint.label(),
            root.path.display()
        );
        println!("    {}", root.reason);
    }
}

pub fn write_json(report: &ScanReport, path: &Path) -> Result<()> {
    let text = serde_json::to_string_pretty(report)?;
    fs::write(path, text).with_context(|| format!("writing JSON report to {}", path.display()))
}

pub fn write_markdown(report: &ScanReport, path: &Path) -> Result<()> {
    let mut md = String::new();
    md.push_str("# HoardHound targeted storage report\n\n");
    md.push_str(&format!("Generated: `{}`  \n", report.generated_at_utc));
    md.push_str(&format!("Profile: `{}`  \n", report.profile.as_str()));
    md.push_str(&format!(
        "Minimum size: `{}`  \n",
        format_size(report.min_size_bytes)
    ));
    md.push_str(&format!(
        "Scanned size: `{}`  \n",
        format_size(report.stats.bytes_seen)
    ));
    md.push_str(&format!(
        "Files: `{}` | Directories: `{}` | Roots: `{}`\n\n",
        report.stats.files_scanned, report.stats.dirs_scanned, report.stats.roots_scanned
    ));

    md.push_str("| ID | Size | Category | Confidence | Folder | Advice |\n");
    md.push_str("|---:|---:|---|---:|---|---|\n");
    for entry in &report.entries {
        md.push_str(&format!(
            "| {} | {} | {} | {}% | `{}` | {} |\n",
            entry.id,
            format_size(entry.size_bytes),
            entry.category.label(),
            entry.confidence,
            entry.path.display(),
            escape_md(&entry.advice)
        ));
    }

    md.push_str("\n## Scan roots\n\n");
    for root in &report.roots {
        md.push_str(&format!(
            "- `{}` — {}\n",
            root.path.display(),
            escape_md(&root.reason)
        ));
    }

    fs::write(path, md).with_context(|| format!("writing Markdown report to {}", path.display()))
}

pub fn print_explain(verbose: bool) {
    println!("HoardHound is intentionally not a whole-drive analyzer. It searches targeted roots likely to contain user-owned reclaimable space:");
    println!("  - visible user folders: Downloads, Desktop, Documents, Videos, Pictures, Music, Saved Games");
    println!("  - per-user app data: AppData Local/Roaming/LocalLow");
    println!("  - known game and launcher locations: Steam, Epic, XboxGames, GOG, EA, Ubisoft, Riot, Battle.net-style folders");
    println!("  - known mod/app storage: Vortex, Mod Organizer, CurseForge, r2modman, Thunderstore, Minecraft, Roblox, itch.io");
    println!();
    println!("Safety defaults:");
    println!("  - read-only; no delete command in v1");
    println!("  - does not follow symlinks/junctions by default");
    println!("  - does not broadly crawl C:\\Windows, Program Files, ProgramData, Recycle Bin, Recovery, or System Volume Information");
    println!("  - specific known game roots under Program Files are allowed because they are targeted, not system-wide scans");

    if verbose {
        println!();
        println!("Classification is heuristic. A result gets a category and confidence from path evidence: launcher roots, manifest discovery, AppData location, cache names, mod-manager names, media/capture names, and developer cache names.");
        println!("Advice is conservative: review in the owning app or launcher first; use manual deletion only when you understand the folder.");
    }
}

fn escape_md(input: &str) -> String {
    input.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_style_uses_ascii_rules_and_disables_color() {
        let style = OutputStyle::from_flags(true, true);

        assert!(!style.color_enabled());
        assert_eq!(style.rule(6), "------");
    }

    #[test]
    fn decorated_style_keeps_unicode_rules() {
        let style = OutputStyle::from_flags(true, false);

        assert!(style.color_enabled());
        assert_eq!(style.rule(6), "──────");
    }
}
