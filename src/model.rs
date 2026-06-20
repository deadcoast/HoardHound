use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScanProfile {
    Default,
    Games,
    AppData,
    Downloads,
    Media,
    FullUser,
    Paranoid,
}

impl ScanProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Games => "games",
            Self::AppData => "appdata",
            Self::Downloads => "downloads",
            Self::Media => "media",
            Self::FullUser => "full-user",
            Self::Paranoid => "paranoid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Games,
    Launcher,
    ModManager,
    AppData,
    Cache,
    Downloads,
    Media,
    Documents,
    Developer,
    UserApp,
    PersonalFiles,
    Unknown,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Games => "Games",
            Self::Launcher => "Launcher",
            Self::ModManager => "Mods",
            Self::AppData => "AppData",
            Self::Cache => "Cache",
            Self::Downloads => "Downloads",
            Self::Media => "Media",
            Self::Documents => "Documents",
            Self::Developer => "Dev",
            Self::UserApp => "User App",
            Self::PersonalFiles => "Personal",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRoot {
    pub path: PathBuf,
    pub label: String,
    pub category_hint: Category,
    pub source: RootSource,
    pub reason: String,
    pub trusted_user_space: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RootSource {
    KnownFolder,
    SteamLibrary,
    EpicManifest,
    KnownLauncherPath,
    KnownModPath,
    KnownUserAppPath,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub generated_at_utc: String,
    pub profile: ScanProfile,
    pub min_size_bytes: u64,
    pub roots: Vec<ScanRoot>,
    pub entries: Vec<ResultEntry>,
    pub stats: ScanStats,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultEntry {
    pub id: usize,
    pub path: PathBuf,
    pub display_name: String,
    pub size_bytes: u64,
    pub category: Category,
    pub confidence: u8,
    pub reason: String,
    pub advice: String,
    pub root_label: String,
    pub largest_children: Vec<ChildSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildSummary {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub category: Category,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanStats {
    pub elapsed_ms: u128,
    pub roots_scanned: u64,
    pub dirs_scanned: u64,
    pub files_scanned: u64,
    pub skipped_dirs: u64,
    pub access_errors: u64,
    pub bytes_seen: u64,
}

impl ScanStats {
    pub fn merge(&mut self, other: &ScanStats) {
        self.elapsed_ms += other.elapsed_ms;
        self.roots_scanned += other.roots_scanned;
        self.dirs_scanned += other.dirs_scanned;
        self.files_scanned += other.files_scanned;
        self.skipped_dirs += other.skipped_dirs;
        self.access_errors += other.access_errors;
        self.bytes_seen += other.bytes_seen;
    }
}

#[derive(Debug, Clone)]
pub struct DirNode {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub direct_file_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub children: Vec<DirNode>,
}

impl DirNode {
    pub fn empty(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| path.to_str().unwrap_or("<root>"))
            .to_string();

        Self {
            path,
            name,
            size_bytes: 0,
            direct_file_bytes: 0,
            file_count: 0,
            dir_count: 0,
            children: Vec::new(),
        }
    }

    pub fn largest_children(&self, count: usize) -> Vec<&DirNode> {
        let mut children: Vec<&DirNode> = self.children.iter().collect();
        children.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        children.truncate(count);
        children
    }
}

#[derive(Debug, Clone)]
pub struct Classification {
    pub category: Category,
    pub confidence: u8,
    pub reason: String,
    pub advice: String,
    pub special_candidate: bool,
}
