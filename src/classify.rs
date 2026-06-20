use crate::model::{Category, Classification, ScanRoot};
use crate::windows;
use std::path::Path;

pub fn classify_path(path: &Path, root: &ScanRoot) -> Classification {
    let normalized = windows::normalize_for_compare(path);
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if contains_any(
        &normalized,
        &[
            "\\steamapps\\common\\",
            "\\xboxgames\\",
            "\\gog games\\",
            "\\epic games\\",
            "\\ea games\\",
            "\\origin games\\",
            "\\riot games\\",
        ],
    ) {
        return Classification {
            category: Category::Games,
            confidence: 95,
            reason: "matched a known game library/install path".to_string(),
            advice: "Review through the relevant launcher first when possible; uninstalling from the launcher is safer than manual deletion.".to_string(),
            special_candidate: true,
        };
    }

    if contains_any(
        &normalized,
        &[
            "modorganizer",
            "mod organizer",
            "vortex",
            "nexus mod manager",
            "r2modman",
            "thunderstore",
            "curseforge",
            "\\mods\\",
        ],
    ) {
        return Classification {
            category: Category::ModManager,
            confidence: 93,
            reason: "matched known mod-manager/mod profile naming".to_string(),
            advice: "Open the mod manager first; delete unused profiles/download caches there when available.".to_string(),
            special_candidate: true,
        };
    }

    if contains_any(
        &normalized,
        &["\\.minecraft", "\\roblox", "\\saved games", "\\my games\\"],
    ) {
        return Classification {
            category: Category::Games,
            confidence: 88,
            reason: "matched a per-user game data folder".to_string(),
            advice: "Inspect saves, screenshots, logs, shader caches, and old modpacks before deleting anything.".to_string(),
            special_candidate: true,
        };
    }

    if is_cache_like(&normalized, &name) {
        return Classification {
            category: Category::Cache,
            confidence: 84,
            reason: "matched cache/temp/shader-cache naming".to_string(),
            advice: "Usually reviewable cleanup candidate, but close related apps first. Prefer app/launcher cleanup options when available.".to_string(),
            special_candidate: true,
        };
    }

    if contains_any(
        &normalized,
        &[
            "\\appdata\\local\\programs\\",
            "\\appdata\\local\\packages\\",
            "\\appdata\\local\\microsoft\\windowsapps\\",
        ],
    ) {
        return Classification {
            category: Category::UserApp,
            confidence: 82,
            reason: "matched per-user installed app/package location".to_string(),
            advice: "Likely a user-installed app or Store package. Use the app/launcher uninstaller where possible.".to_string(),
            special_candidate: true,
        };
    }

    if contains_any(
        &normalized,
        &[
            "\\appdata\\local",
            "\\appdata\\roaming",
            "\\appdata\\locallow",
        ],
    ) {
        return Classification {
            category: Category::AppData,
            confidence: 74,
            reason: "inside per-user AppData".to_string(),
            advice: "Do not bulk-delete AppData. Inspect the owning app, caches, logs, downloads, or old app data.".to_string(),
            special_candidate: false,
        };
    }

    if contains_any(&normalized, &["\\downloads", "\\desktop"])
        || matches!(root.category_hint, Category::Downloads)
    {
        return Classification {
            category: Category::Downloads,
            confidence: 78,
            reason: "inside a user cleanup folder such as Downloads/Desktop".to_string(),
            advice: "Look for installers, duplicate archives, extracted folders, recordings, and old downloads.".to_string(),
            special_candidate: false,
        };
    }

    if contains_any(
        &normalized,
        &[
            "\\videos",
            "\\pictures",
            "\\music",
            "captures",
            "recordings",
            "screenshots",
        ],
    ) || matches!(root.category_hint, Category::Media)
    {
        return Classification {
            category: Category::Media,
            confidence: 76,
            reason: "matched media/capture folder naming".to_string(),
            advice: "Review recordings, captures, exported projects, and duplicate media."
                .to_string(),
            special_candidate: false,
        };
    }

    if contains_any(
        &normalized,
        &[
            "\\.gradle",
            "\\.cargo",
            "\\.nuget",
            "\\node_modules",
            "\\npm-cache",
            "\\pnpm-store",
            "\\pip\\cache",
            "\\docker\\",
            "\\wsl\\",
            "\\vcpkg\\",
        ],
    ) {
        return Classification {
            category: Category::Developer,
            confidence: 82,
            reason: "matched developer cache/build/dependency storage naming".to_string(),
            advice: "Review build caches and dependency stores with the owning tool. Some can be regenerated, but active projects may rely on them.".to_string(),
            special_candidate: true,
        };
    }

    if matches!(
        root.category_hint,
        Category::Games | Category::Launcher | Category::ModManager
    ) {
        return Classification {
            category: root.category_hint,
            confidence: 70,
            reason: "inherits category from targeted scan root".to_string(),
            advice: "Review through the owning launcher/app when possible.".to_string(),
            special_candidate: false,
        };
    }

    Classification {
        category: root.category_hint,
        confidence: 55,
        reason: "large folder found inside a targeted user-owned root".to_string(),
        advice: "Open and inspect before deleting; HoardHound is read-only and only identifies candidates.".to_string(),
        special_candidate: false,
    }
}

fn is_cache_like(normalized: &str, name: &str) -> bool {
    let cache_terms = [
        "cache",
        "caches",
        "code cache",
        "gpucache",
        "shadercache",
        "shader-cache",
        "dxcache",
        "d3dscache",
        "deriveddatacache",
        "crashdumps",
        "logs",
        "temp",
        "tmp",
        "installer cache",
    ];

    cache_terms
        .iter()
        .any(|term| name == *term || normalized.contains(term))
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
