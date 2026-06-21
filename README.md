# HoardHound

**HoardHound** is a Windows-first CLI that sniffs out forgotten user-space storage hogs: games, launchers, mods, AppData bloat, media captures, downloads, caches, and per-user app installs.

It is intentionally **not** a generic whole-drive analyzer. It does not start by crawling `C:\`. It builds a targeted list of likely user-owned scan roots and explains why each result was included.

## Current status

This repository is an early v1 implementation scaffold with real scanning logic, targeted root discovery, read-only reporting, JSON/Markdown exports, and `open <ID>` support. The development environment that generated this source did not include the Rust toolchain, so compile verification should be done locally on Windows.

## Install / build

```powershell
winget install Rustlang.Rustup
rustup default stable
cargo build --release
.\target\release\hoardhound.exe --help
```

## Core usage

```powershell
# Default targeted scan: user folders, AppData, known game roots, known mod roots
hoardhound scan

# Find installed games and mod storage
hoardhound scan --profile games --top 50

# Focus on AppData bloat
hoardhound scan --profile appdata --min-size 500MB

# Preview scan roots before scanning
hoardhound roots --profile default

# Use ASCII-only terminal output for legacy consoles or logs
hoardhound --plain scan

# Export a shareable report
hoardhound scan --json report.json --markdown report.md

# Open a result from the last scan
hoardhound open 7
```

## Profiles

| Profile     | Purpose                                                                                                                       |
|-------------|-------------------------------------------------------------------------------------------------------------------------------|
| `default`   | Balanced scan: visible user folders, AppData, game libraries, launcher roots, mod folders.                                    |
| `games`     | Steam, Epic, XboxGames, GOG, EA, Ubisoft, Riot, Battle.net-style folders, Minecraft, Roblox, itch.io, mods.                   |
| `appdata`   | AppData Local, Roaming, LocalLow. Useful for Electron apps, launcher data, caches, packages, and forgotten per-user installs. |
| `downloads` | Downloads/Desktop/Documents cleanup suspects.                                                                                 |
| `media`     | Videos/Pictures/Music/Desktop. Useful for captures, recordings, exports, screenshots.                                         |
| `full-user` | Entire current user profile, still avoiding Windows/system traps and reparse points.                                          |
| `paranoid`  | Extra conservative: visible user folders and Saved Games only.                                                                |

## Safety model

HoardHound v1 is read-only.

Default safety choices:

- No delete command.
- Does not follow symlinks or Windows reparse points by default.
- Does not broadly crawl `C:\Windows`, `C:\Program Files`, `C:\Program Files (x86)`, `C:\ProgramData`, `System Volume Information`, `$Recycle.Bin`, or `Recovery`.
- Reads launcher manifests only to discover exact install locations.
- Allows exact known game/library roots under Program Files, but does not recursively scan all of Program Files.
- Every result includes a reason and conservative advice.

## Design goal

A normal disk analyzer answers:

> What are the biggest folders on this drive?

HoardHound answers:

> What forgotten games, launchers, app data, mods, caches, installers, recordings, and user-owned app folders can I inspect to recover space?

## Example output

```text
🐺 HoardHound targeted storage report
Profile: default | Roots: 12 | Scanned: 612.5 GB | Files: 340921 | Dirs: 42713 | Time: 18.42s

ID        SIZE  CATEGORY    CONF   FOLDER
──────────────────────────────────────────────────────────────────────────────
1      142.8 GB  Games       95%   Baldurs Gate 3
      C:\Program Files (x86)\Steam\steamapps\common\Baldurs Gate 3
      why: matched a known game library/install path; root: Discovered from Steam libraryfolders.vdf...
      do: Review through the relevant launcher first when possible...
```

## Roadmap

See [`docs/ROADMAP.md`](docs/ROADMAP.md) and [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
