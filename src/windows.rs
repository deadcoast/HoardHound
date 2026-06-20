use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

pub fn system_drive() -> PathBuf {
    let raw = std::env::var_os("SystemDrive")
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "C:".to_string());

    if raw.ends_with(':') {
        PathBuf::from(format!("{}\\", raw))
    } else {
        PathBuf::from(raw)
    }
}

pub fn last_report_path() -> Result<PathBuf> {
    let base = env_path("LOCALAPPDATA")
        .or_else(|| env_path("APPDATA"))
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| anyhow!("unable to determine a writable report directory"))?;
    let dir = base.join("HoardHound");
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    Ok(dir.join("last-scan.json"))
}

pub fn open_in_explorer(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("path no longer exists: {}", path.display()));
    }

    #[cfg(windows)]
    {
        Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .with_context(|| format!("opening {} in File Explorer", path.display()))?;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        Command::new(opener)
            .arg(path)
            .spawn()
            .with_context(|| format!("opening {}", path.display()))?;
        Ok(())
    }
}

pub fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        (metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0
    }

    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

pub fn normalize_for_compare(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

pub fn is_probably_same_or_child(parent: &Path, child: &Path) -> bool {
    let parent = normalize_for_compare(parent);
    let child = normalize_for_compare(child);
    child == parent || child.starts_with(&(parent + "\\"))
}
