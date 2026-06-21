# HoardHound architecture

## Product philosophy

HoardHound is not a WinDirStat clone. It is a targeted discovery tool for reclaimable user-controlled storage. The scanner should avoid broad system crawling and produce actionable suspects with reasons.

## Main modules

- `lib.rs`: reusable library boundary for integration tests and the CLI binary.
- `main.rs`: thin binary entry point that parses CLI arguments and dispatches commands.
- `cli.rs`: clap command surface.
- `roots.rs`: targeted root discovery for user folders, AppData, launchers, game libraries, mod managers, and manifest-derived install locations.
- `scanner.rs`: read-only recursive size aggregation, candidate selection, ranking, and report generation.
- `filters.rs`: safety exclusions and reparse-point handling.
- `classify.rs`: path/category/confidence/advice heuristics.
- `report.rs`: terminal, JSON, and Markdown output.
- `vdf.rs`: Steam `libraryfolders.vdf` parser.
- `windows.rs`: Windows-oriented path helpers, last report path, reparse detection, and Explorer opening.

## Data flow

```text
CLI args
  -> ScanConfig
  -> root discovery
  -> safe recursive scan per root
  -> DirNode aggregation
  -> candidate extraction
  -> classification
  -> duplicate suppression
  -> terminal report + JSON cache
```

## Root discovery strategy

The app discovers roots from three classes:

1. Known folders: `Downloads`, `Desktop`, `Documents`, `Videos`, `Pictures`, `Music`, `Saved Games`, `AppData`.
2. Launcher/library paths: Steam, Epic, XboxGames, GOG, EA, Ubisoft, Riot, common Battle.net game folders.
3. Manifest/config-derived roots: Steam `libraryfolders.vdf`, Epic `.item` manifests.

Future versions should add registry discovery, Battle.net config parsing, Ubisoft/EA config parsing, GOG Galaxy database parsing, and Xbox package metadata.

## Scanner strategy

v1 uses recursive `std::fs::read_dir` traversal and parallelizes across scan roots with Rayon. This is simple and safe. It is intentionally not an MFT-based analyzer because the product goal is targeted user-space scanning, not whole-drive NTFS indexing.

Potential v2 performance improvements:

- optional NTFS/MFT backend for full-user profile only;
- streaming top-k aggregation to reduce memory;
- persistent cache keyed by directory mtime/inode equivalent;
- per-drive concurrency limits to avoid thrashing HDDs;
- adaptive depth-first/priority traversal that surfaces early partial results.

## Candidate selection

The scanner avoids reporting every folder. It reports:

- shallow large folders under targeted roots;
- special folders at any depth when path heuristics identify caches, game data, mod managers, or developer stores;
- exact manifest-derived install roots;
- known standalone game roots.

A duplicate suppression pass removes near-identical parent/child duplicates when a child is at least 85% of the already-selected parent and shares a category.

## Classification categories

- Games
- Launcher
- Mods
- AppData
- Cache
- Downloads
- Media
- Documents
- Developer
- User App
- Personal
- Unknown

Each result receives:

- category;
- confidence;
- reason;
- conservative action advice;
- largest child summaries.

## Safety invariants

- No delete command in v1.
- No symlink/junction following by default.
- No broad `C:\` scan by default.
- No broad Program Files scan.
- No broad ProgramData scan.
- Permission errors are counted, not fatal.
- Results explain root/source reasoning.
