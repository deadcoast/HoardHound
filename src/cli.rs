use crate::model::ScanProfile;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "hoardhound",
    author,
    version,
    about = "Sniffs out forgotten user-space storage hogs on Windows.",
    long_about = "HoardHound is a targeted Windows storage scanner. It focuses on user-owned files, game libraries, launchers, AppData bloat, mods, media, downloads, and forgotten installs instead of blindly crawling Windows system directories."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Disable ANSI color in terminal output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Use ASCII-only terminal output and disable ANSI color.
    #[arg(long, global = true)]
    pub plain: bool,

    /// Print less decorative output.
    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Scan targeted user-space locations and report the largest actionable folders.
    Scan(ScanArgs),

    /// Show which roots HoardHound would scan for a profile.
    Roots(RootsArgs),

    /// Open a result from the last scan in File Explorer.
    Open(OpenArgs),

    /// Explain the scanner safety model and category heuristics.
    Explain(ExplainArgs),
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    /// Scan profile to use.
    #[arg(short, long, value_enum, default_value = "default")]
    pub profile: CliProfile,

    /// Number of results to display.
    #[arg(short, long, default_value_t = 30)]
    pub top: usize,

    /// Minimum folder size to report. Examples: 500MB, 1GB, 1.5GB.
    #[arg(long, default_value = "1GB")]
    pub min_size: String,

    /// Also save the report as JSON.
    #[arg(long, value_name = "FILE")]
    pub json: Option<PathBuf>,

    /// Also save the report as Markdown.
    #[arg(long, value_name = "FILE")]
    pub markdown: Option<PathBuf>,

    /// Show protected/skipped directory counts and scan warnings.
    #[arg(long)]
    pub show_skipped: bool,

    /// Follow symlinks and junctions. Off by default to avoid loops and double-counting.
    #[arg(long)]
    pub follow_links: bool,

    /// Permit known launcher roots under Program Files. Default profile already allows only curated game roots.
    #[arg(long)]
    pub include_program_files: bool,

    /// Candidate depth per scan root. Higher means more nested results and more noise.
    #[arg(long, default_value_t = 2)]
    pub candidate_depth: usize,
}

#[derive(Debug, Args)]
pub struct RootsArgs {
    /// Scan profile to preview.
    #[arg(short, long, value_enum, default_value = "default")]
    pub profile: CliProfile,

    /// Include known launcher roots under Program Files in the preview.
    #[arg(long)]
    pub include_program_files: bool,
}

#[derive(Debug, Args)]
pub struct OpenArgs {
    /// Result ID from the last scan.
    pub id: usize,
}

#[derive(Debug, Args)]
pub struct ExplainArgs {
    /// Show extra heuristic detail.
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliProfile {
    /// User folders + AppData + known game/launcher/mod roots.
    Default,
    /// Steam, Epic, Xbox, GOG, EA, Ubisoft, Battle.net, Riot, mods, and game-adjacent AppData.
    Games,
    /// AppData Local/Roaming/LocalLow, grouped into actionable suspects.
    #[value(alias = "appdata", alias = "app-data")]
    AppData,
    /// Downloads, Desktop, Documents, installers, archives, and personal cleanup suspects.
    Downloads,
    /// Videos, Pictures, Music, screenshots, recordings, and media-heavy folders.
    Media,
    /// Entire current user profile, still excluding Windows/system traps and reparse points by default.
    #[value(alias = "fulluser", alias = "full-user")]
    FullUser,
    /// Extra conservative: no Program Files, no launcher manifests, mostly visible user folders.
    Paranoid,
}

impl From<CliProfile> for ScanProfile {
    fn from(value: CliProfile) -> Self {
        match value {
            CliProfile::Default => ScanProfile::Default,
            CliProfile::Games => ScanProfile::Games,
            CliProfile::AppData => ScanProfile::AppData,
            CliProfile::Downloads => ScanProfile::Downloads,
            CliProfile::Media => ScanProfile::Media,
            CliProfile::FullUser => ScanProfile::FullUser,
            CliProfile::Paranoid => ScanProfile::Paranoid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_as_global_output_flag() {
        let cli = Cli::try_parse_from(["hoardhound", "--plain", "roots"]).unwrap();

        assert!(cli.plain);
        assert!(!cli.no_color);
    }
}
