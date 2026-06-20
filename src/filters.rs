use crate::config::ScanConfig;
use crate::model::ScanRoot;
use crate::windows;
use std::fs;
use std::path::Path;

pub enum SkipDecision {
    Scan,
    Skip(String),
}

pub fn should_skip_dir(
    path: &Path,
    metadata: &fs::Metadata,
    root: &ScanRoot,
    config: &ScanConfig,
) -> SkipDecision {
    if !config.follow_links
        && (metadata.file_type().is_symlink() || windows::is_reparse_point(metadata))
    {
        return SkipDecision::Skip(
            "reparse point/symlink skipped to avoid loops and double counting".to_string(),
        );
    }

    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if is_never_scan_name(&name) {
        return SkipDecision::Skip("Windows/system directory name excluded".to_string());
    }

    // If a caller accidentally supplies a broad system root, keep the default behavior conservative.
    if !config.include_program_files
        && !root.trusted_user_space
        && is_broad_program_files_path(path)
    {
        return SkipDecision::Skip("broad Program Files traversal is off by default".to_string());
    }

    SkipDecision::Scan
}

fn is_never_scan_name(name: &str) -> bool {
    matches!(
        name,
        "windows"
            | "system volume information"
            | "$recycle.bin"
            | "recovery"
            | "perflogs"
            | "config.msi"
            | "msocache"
            | "default"
            | "all users"
    )
}

fn is_broad_program_files_path(path: &Path) -> bool {
    let p = windows::normalize_for_compare(path);
    p.ends_with("\\program files") || p.ends_with("\\program files (x86)")
}
