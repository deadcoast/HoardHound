use hoardhound::config::ScanConfig;
use hoardhound::model::{Category, RootSource, ScanProfile, ScanRoot};
use hoardhound::scanner;
use std::fs::{self, File};
use tempfile::tempdir;

#[test]
fn scans_synthetic_windows_game_tree() {
    let temp = tempdir().unwrap();
    let game_root = temp
        .path()
        .join("SteamLibrary")
        .join("steamapps")
        .join("common")
        .join("Synthetic Game");
    fs::create_dir_all(&game_root).unwrap();
    File::create(game_root.join("payload.bin"))
        .unwrap()
        .set_len(2 * 1024 * 1024)
        .unwrap();

    let config = ScanConfig::conservative_default(ScanProfile::Games, 10, 1024 * 1024);
    let roots = vec![ScanRoot {
        path: game_root.clone(),
        label: "Synthetic Game".to_string(),
        category_hint: Category::Games,
        source: RootSource::Manual,
        reason: "synthetic integration root".to_string(),
        trusted_user_space: true,
    }];

    let report = scanner::scan(&config, &roots).unwrap();

    assert_eq!(report.stats.roots_scanned, 1);
    assert_eq!(report.stats.files_scanned, 1);
    assert_eq!(report.stats.bytes_seen, 2 * 1024 * 1024);
    assert_eq!(report.entries.len(), 1);

    let entry = &report.entries[0];
    assert_eq!(entry.id, 1);
    assert_eq!(entry.path, game_root);
    assert_eq!(entry.display_name, "Synthetic Game");
    assert_eq!(entry.category, Category::Games);
    assert!(entry.reason.contains("known game library/install path"));
}
