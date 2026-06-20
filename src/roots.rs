use crate::config::ScanConfig;
use crate::model::{Category, RootSource, ScanProfile, ScanRoot};
use crate::vdf;
use crate::windows;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_roots(config: &ScanConfig) -> Result<Vec<ScanRoot>> {
    let mut roots = Vec::new();

    match config.profile {
        ScanProfile::Default => {
            add_visible_user_roots(&mut roots);
            add_appdata_roots(&mut roots);
            add_game_roots(&mut roots, true)?;
            add_mod_roots(&mut roots);
        }
        ScanProfile::Games => {
            add_saved_games(&mut roots);
            add_game_roots(&mut roots, true)?;
            add_mod_roots(&mut roots);
        }
        ScanProfile::AppData => {
            add_appdata_roots(&mut roots);
        }
        ScanProfile::Downloads => {
            add_download_cleanup_roots(&mut roots);
        }
        ScanProfile::Media => {
            add_media_roots(&mut roots);
        }
        ScanProfile::FullUser => {
            if let Some(profile) = windows::env_path("USERPROFILE") {
                push_if_exists(
                    &mut roots,
                    profile,
                    "Current user profile",
                    Category::PersonalFiles,
                    RootSource::KnownFolder,
                    "Full-user profile requested; Windows/system traps are still excluded while scanning.",
                    true,
                );
            }
        }
        ScanProfile::Paranoid => {
            add_download_cleanup_roots(&mut roots);
            add_media_roots(&mut roots);
            add_saved_games(&mut roots);
        }
    }

    Ok(dedupe_roots(roots))
}

fn add_visible_user_roots(roots: &mut Vec<ScanRoot>) {
    if let Some(profile) = windows::env_path("USERPROFILE") {
        let known = [
            ("Desktop", "Desktop", Category::PersonalFiles),
            ("Downloads", "Downloads", Category::Downloads),
            ("Documents", "Documents", Category::Documents),
            ("Videos", "Videos", Category::Media),
            ("Pictures", "Pictures", Category::Media),
            ("Music", "Music", Category::Media),
            ("Saved Games", "Saved Games", Category::Games),
        ];

        for (child, label, category) in known {
            push_if_exists(
                roots,
                profile.join(child),
                label,
                category,
                RootSource::KnownFolder,
                "Visible user-owned folder commonly used by games, media tools, launchers, and installers.",
                true,
            );
        }
    }
}

fn add_download_cleanup_roots(roots: &mut Vec<ScanRoot>) {
    if let Some(profile) = windows::env_path("USERPROFILE") {
        let known = [
            ("Downloads", "Downloads", Category::Downloads),
            ("Desktop", "Desktop", Category::PersonalFiles),
            ("Documents", "Documents", Category::Documents),
        ];
        for (child, label, category) in known {
            push_if_exists(
                roots,
                profile.join(child),
                label,
                category,
                RootSource::KnownFolder,
                "Cleanup profile: installers, archives, extracted folders, recordings, and forgotten personal files often accumulate here.",
                true,
            );
        }
    }
}

fn add_media_roots(roots: &mut Vec<ScanRoot>) {
    if let Some(profile) = windows::env_path("USERPROFILE") {
        let known = [
            ("Videos", "Videos", Category::Media),
            ("Pictures", "Pictures", Category::Media),
            ("Music", "Music", Category::Media),
            ("Desktop", "Desktop", Category::PersonalFiles),
        ];
        for (child, label, category) in known {
            push_if_exists(
                roots,
                profile.join(child),
                label,
                category,
                RootSource::KnownFolder,
                "Media profile: screenshots, recordings, captures, exported projects, and personal media can hide large space usage.",
                true,
            );
        }
    }
}

fn add_saved_games(roots: &mut Vec<ScanRoot>) {
    if let Some(profile) = windows::env_path("USERPROFILE") {
        push_if_exists(
            roots,
            profile.join("Saved Games"),
            "Saved Games",
            Category::Games,
            RootSource::KnownFolder,
            "Per-user game save/config folder; useful for old saves, screenshots, logs, and modded game state.",
            true,
        );
    }
}

fn add_appdata_roots(roots: &mut Vec<ScanRoot>) {
    for (env_name, label) in [
        ("LOCALAPPDATA", "AppData Local"),
        ("APPDATA", "AppData Roaming"),
    ] {
        if let Some(path) = windows::env_path(env_name) {
            push_if_exists(
                roots,
                path,
                label,
                Category::AppData,
                RootSource::KnownFolder,
                "Per-user application data. Targeted because launchers, Electron apps, shader caches, game configs, and forgotten user-installed apps often live here.",
                true,
            );
        }
    }

    if let Some(profile) = windows::env_path("USERPROFILE") {
        push_if_exists(
            roots,
            profile.join("AppData").join("LocalLow"),
            "AppData LocalLow",
            Category::AppData,
            RootSource::KnownFolder,
            "Per-user low-integrity app/game data; Unity games and browser/plugin-era apps commonly store data here.",
            true,
        );
    }
}

fn add_game_roots(roots: &mut Vec<ScanRoot>, read_manifests: bool) -> Result<()> {
    add_steam_roots(roots)?;

    if read_manifests {
        add_epic_manifest_roots(roots)?;
    }

    let system = windows::system_drive();
    let pf =
        windows::env_path("ProgramFiles").unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    let pfx86 = windows::env_path("ProgramFiles(x86)")
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let local = windows::env_path("LOCALAPPDATA");
    let roaming = windows::env_path("APPDATA");
    let profile = windows::env_path("USERPROFILE");

    let known = vec![
        (
            system.join("XboxGames"),
            "XboxGames",
            Category::Games,
            "Known Xbox app PC game install root.",
        ),
        (
            system.join("GOG Games"),
            "GOG Games",
            Category::Games,
            "Common GOG offline installer / Galaxy game install root.",
        ),
        (
            pf.join("GOG Galaxy").join("Games"),
            "GOG Galaxy Games",
            Category::Games,
            "Known GOG Galaxy game library folder.",
        ),
        (
            pfx86.join("GOG Galaxy").join("Games"),
            "GOG Galaxy Games x86",
            Category::Games,
            "Known GOG Galaxy game library folder.",
        ),
        (
            pf.join("Epic Games"),
            "Epic Games",
            Category::Games,
            "Known Epic Games install root.",
        ),
        (
            pfx86.join("Epic Games"),
            "Epic Games x86",
            Category::Games,
            "Known Epic Games install root.",
        ),
        (
            pf.join("EA Games"),
            "EA Games",
            Category::Games,
            "Known EA app game install root.",
        ),
        (
            pfx86.join("Origin Games"),
            "Origin Games",
            Category::Games,
            "Known Origin/EA legacy game install root.",
        ),
        (
            pf.join("Ubisoft")
                .join("Ubisoft Game Launcher")
                .join("games"),
            "Ubisoft Connect Games",
            Category::Games,
            "Known Ubisoft Connect games folder.",
        ),
        (
            pfx86
                .join("Ubisoft")
                .join("Ubisoft Game Launcher")
                .join("games"),
            "Ubisoft Connect Games x86",
            Category::Games,
            "Known Ubisoft Connect games folder.",
        ),
        (
            system.join("Riot Games"),
            "Riot Games",
            Category::Games,
            "Known Riot Games install root.",
        ),
        (
            pf.join("Riot Games"),
            "Riot Games Program Files",
            Category::Games,
            "Known Riot Games install root.",
        ),
        (
            pfx86.join("World of Warcraft"),
            "World of Warcraft",
            Category::Games,
            "Common Battle.net game install folder.",
        ),
        (
            pf.join("World of Warcraft"),
            "World of Warcraft",
            Category::Games,
            "Common Battle.net game install folder.",
        ),
        (
            pf.join("Diablo IV"),
            "Diablo IV",
            Category::Games,
            "Common Battle.net game install folder.",
        ),
        (
            pfx86.join("Diablo IV"),
            "Diablo IV",
            Category::Games,
            "Common Battle.net game install folder.",
        ),
        (
            pf.join("Overwatch"),
            "Overwatch",
            Category::Games,
            "Common Battle.net game install folder.",
        ),
        (
            pfx86.join("Overwatch"),
            "Overwatch",
            Category::Games,
            "Common Battle.net game install folder.",
        ),
    ];

    for (path, label, category, reason) in known {
        push_if_exists(
            roots,
            path,
            label,
            category,
            RootSource::KnownLauncherPath,
            reason,
            true,
        );
    }

    if let Some(path) = local.as_ref().map(|p| p.join("Roblox")) {
        push_if_exists(
            roots,
            path,
            "Roblox",
            Category::Games,
            RootSource::KnownUserAppPath,
            "Per-user Roblox install/cache folder.",
            true,
        );
    }
    if let Some(path) = roaming.as_ref().map(|p| p.join(".minecraft")) {
        push_if_exists(
            roots,
            path,
            "Minecraft Java",
            Category::Games,
            RootSource::KnownUserAppPath,
            "Minecraft Java per-user game/mod/cache folder.",
            true,
        );
    }
    if let Some(path) = profile.as_ref().map(|p| p.join("curseforge")) {
        push_if_exists(
            roots,
            path,
            "CurseForge instances",
            Category::ModManager,
            RootSource::KnownModPath,
            "Common CurseForge Minecraft/modded-game instances folder.",
            true,
        );
    }
    if let Some(path) = local.as_ref().map(|p| p.join("itch")) {
        push_if_exists(
            roots,
            path,
            "itch.io Local",
            Category::Games,
            RootSource::KnownUserAppPath,
            "Per-user itch.io app/game data folder.",
            true,
        );
    }
    if let Some(path) = roaming.as_ref().map(|p| p.join("itch")) {
        push_if_exists(
            roots,
            path,
            "itch.io Roaming",
            Category::Games,
            RootSource::KnownUserAppPath,
            "Per-user itch.io app/game data folder.",
            true,
        );
    }

    Ok(())
}

fn add_mod_roots(roots: &mut Vec<ScanRoot>) {
    let local = windows::env_path("LOCALAPPDATA");
    let roaming = windows::env_path("APPDATA");
    let profile = windows::env_path("USERPROFILE");

    let mut known: Vec<(PathBuf, &str, &str)> = Vec::new();

    if let Some(local) = local.as_ref() {
        known.extend([
            (
                local.join("ModOrganizer"),
                "Mod Organizer",
                "Known Mod Organizer 2 profile/download/cache location.",
            ),
            (
                local.join("Vortex"),
                "Vortex Local",
                "Known Nexus Vortex per-user data location.",
            ),
            (
                local.join("Programs").join("CurseForge Windows"),
                "CurseForge App",
                "Per-user CurseForge application install location.",
            ),
        ]);
    }
    if let Some(roaming) = roaming.as_ref() {
        known.extend([
            (
                roaming.join("Vortex"),
                "Vortex Roaming",
                "Known Nexus Vortex per-user data location.",
            ),
            (
                roaming.join("Nexus Mod Manager"),
                "Nexus Mod Manager",
                "Known Nexus Mod Manager data location.",
            ),
            (
                roaming.join("r2modmanPlus-local"),
                "r2modman",
                "Known r2modman/Thunderstore mod profile folder.",
            ),
            (
                roaming.join("Thunderstore Mod Manager"),
                "Thunderstore Mod Manager",
                "Known Thunderstore mod profile folder.",
            ),
            (
                roaming.join("CurseForge"),
                "CurseForge Roaming",
                "Known CurseForge data location.",
            ),
        ]);
    }
    if let Some(profile) = profile.as_ref() {
        known.extend([
            (
                profile.join("Documents").join("My Games"),
                "Documents My Games",
                "Common user-owned game configs, saves, screenshots, and modded-game state.",
            ),
            (
                profile
                    .join("Documents")
                    .join("Electronic Arts")
                    .join("The Sims 4")
                    .join("Mods"),
                "Sims 4 Mods",
                "Common Sims 4 mod folder.",
            ),
        ]);
    }

    for (path, label, reason) in known {
        push_if_exists(
            roots,
            path,
            label,
            Category::ModManager,
            RootSource::KnownModPath,
            reason,
            true,
        );
    }
}

fn add_steam_roots(roots: &mut Vec<ScanRoot>) -> Result<()> {
    let pf =
        windows::env_path("ProgramFiles").unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    let pfx86 = windows::env_path("ProgramFiles(x86)")
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let system = windows::system_drive();

    let steam_dirs = [
        pfx86.join("Steam"),
        pf.join("Steam"),
        system.join("Steam"),
        system.join("Games").join("Steam"),
    ];

    for steam in steam_dirs {
        let common = steam.join("steamapps").join("common");
        push_if_exists(
            roots,
            common,
            "Steam common",
            Category::Games,
            RootSource::SteamLibrary,
            "Steam library common folder; each child is usually an installed game.",
            true,
        );

        let vdf_path = steam.join("steamapps").join("libraryfolders.vdf");
        if vdf_path.exists() {
            let content = fs::read_to_string(&vdf_path)
                .with_context(|| format!("reading Steam library file {}", vdf_path.display()))?;
            for library in vdf::parse_steam_libraryfolders_vdf(&content) {
                let common = library.join("steamapps").join("common");
                push_if_exists(
                    roots,
                    common,
                    "Steam library",
                    Category::Games,
                    RootSource::SteamLibrary,
                    "Discovered from Steam libraryfolders.vdf; each child is usually an installed game.",
                    true,
                );
            }
        }
    }

    Ok(())
}

fn add_epic_manifest_roots(roots: &mut Vec<ScanRoot>) -> Result<()> {
    let program_data =
        windows::env_path("ProgramData").unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    let manifest_dir = program_data
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");

    if !manifest_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(&manifest_dir)
        .with_context(|| format!("reading {}", manifest_dir.display()))?
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("item") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(&content) else {
            continue;
        };
        let Some(install) = json.get("InstallLocation").and_then(|v| v.as_str()) else {
            continue;
        };
        let display = json
            .get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or("Epic game");
        push_if_exists(
            roots,
            PathBuf::from(install),
            display,
            Category::Games,
            RootSource::EpicManifest,
            "Discovered from Epic Games Launcher manifest InstallLocation.",
            true,
        );
    }

    Ok(())
}

fn push_if_exists(
    roots: &mut Vec<ScanRoot>,
    path: PathBuf,
    label: impl Into<String>,
    category_hint: Category,
    source: RootSource,
    reason: impl Into<String>,
    trusted_user_space: bool,
) {
    if path.is_dir() {
        roots.push(ScanRoot {
            path,
            label: label.into(),
            category_hint,
            source,
            reason: reason.into(),
            trusted_user_space,
        });
    }
}

fn dedupe_roots(mut roots: Vec<ScanRoot>) -> Vec<ScanRoot> {
    roots.sort_by(|a, b| {
        a.path
            .components()
            .count()
            .cmp(&b.path.components().count())
            .then_with(|| a.path.cmp(&b.path))
    });

    let mut seen = BTreeSet::new();
    let mut out: Vec<ScanRoot> = Vec::new();

    'root: for root in roots {
        let key = windows::normalize_for_compare(&root.path);
        if !seen.insert(key) {
            continue;
        }

        for existing in &out {
            if windows::is_probably_same_or_child(&existing.path, &root.path) {
                // Avoid duplicate traversal. The classifier will still detect special descendants.
                continue 'root;
            }
        }
        out.push(root);
    }

    out
}

#[allow(dead_code)]
fn is_parent(parent: &Path, child: &Path) -> bool {
    windows::is_probably_same_or_child(parent, child)
}
