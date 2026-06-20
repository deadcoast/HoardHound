# targeted Windows “space hunter” CLI

Your tool’s job is not:

> “What are the biggest folders on C:?”

Your tool’s job is:

> “What forgotten games, launchers, caches, user-installed apps, mod folders, and personal data are eating space in places a normal user actually controls?”

That distinction matters a lot.

## Language choice

Use **Rust**.

Python can work for a prototype, but for this specific tool Rust is the better long-term choice:

| Language    | Verdict                                                                                                   |
| ----------- | --------------------------------------------------------------------------------------------------------- |
| **Python**  | Good prototype, slower directory traversal, annoying packaging on Windows unless bundled carefully        |
| **Rust**    | Best fit: fast, single `.exe`, safe filesystem walking, beautiful CLI ecosystem                           |
| **C++**     | Fast, but more painful than necessary for this kind of tool                                               |
| **C#/.NET** | Also valid on Windows, especially if you want native Windows APIs, but Rust is cleaner for a portable CLI |

Python is not “too slow” in an absolute sense. The bottleneck is often disk I/O, not CPU. But Python becomes annoying when scanning hundreds of thousands of files, doing parallel traversal, handling permission errors cleanly, and shipping a polished CLI as one executable.

Rust gives you:

```text
fast recursive scanning
single executable
good Windows path handling
parallel directory walking
nice terminal UI libraries
low memory overhead
strong error handling
```

For a QoL CLI tool, Rust is the right call.

---

## Core design

I would build it around **search profiles**, not “scan everything.”

Example command:

```powershell
spacehunter scan --profile user-apps
```

or:

```powershell
spacehunter scan --games
spacehunter scan --launchers
spacehunter scan --downloads
spacehunter scan --all-user
```

The tool should make its intent obvious.

---

## Search locations

Do **not** start with `C:\`.

Start with targeted roots.

## High-value user-space folders

Scan these by default:

```text
%USERPROFILE%\Desktop
%USERPROFILE%\Downloads
%USERPROFILE%\Documents
%USERPROFILE%\Videos
%USERPROFILE%\Pictures
%USERPROFILE%\Music
%USERPROFILE%\Saved Games
%USERPROFILE%\AppData\Local
%USERPROFILE%\AppData\Roaming
%USERPROFILE%\AppData\LocalLow
```

These catch:

```text
mods
save backups
game launchers
Electron apps
installers
cache folders
portable apps
screenshots/videos
old extracted archives
mod managers
emulators
game config/data folders
```

## Game / launcher-specific folders

Also scan known launcher roots if they exist:

```text
C:\XboxGames
C:\Program Files (x86)\Steam\steamapps\common
C:\Program Files\Steam\steamapps\common
C:\Program Files\Epic Games
C:\Program Files (x86)\Epic Games
C:\Program Files\EA Games
C:\Program Files\Ubisoft
C:\Program Files (x86)\Ubisoft
C:\Program Files (x86)\Battle.net
C:\Program Files\GOG Galaxy\Games
C:\GOG Games
```

But be careful: `Program Files` is semi-system territory. I would not recursively scan all of it by default.

Instead, only scan **known game/library subfolders**.

Good:

```text
C:\Program Files (x86)\Steam\steamapps\common
```

Bad:

```text
C:\Program Files
```

## Detect Steam libraries properly

Steam often installs games outside the default path.

Read:

```text
Steam\steamapps\libraryfolders.vdf
```

Then scan each:

```text
<library>\steamapps\common
```

This is exactly the kind of feature that makes the tool useful instead of generic.

## Detect Epic manifests

Epic usually stores manifests in:

```text
C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests
```

Those manifests can tell you install locations. You do not need to scan all of `ProgramData`; just read Epic’s manifest folder and then scan the install paths it references.

## Detect Xbox / Microsoft Store games

Useful paths:

```text
C:\XboxGames
%LOCALAPPDATA%\Packages
```

Be careful with Microsoft Store packages because names are ugly and permissions can get weird. Treat them as “user app packages,” not normal folders.

---

## What to exclude

Hard-exclude these unless the user explicitly asks for full-drive mode:

```text
C:\Windows
C:\ProgramData
C:\Recovery
C:\System Volume Information
C:\$Recycle.Bin
C:\Program Files
C:\Program Files (x86)
C:\Users\Default
C:\Users\Public
C:\Users\All Users
```

But with exceptions.

For example:

```text
C:\Program Files (x86)\Steam\steamapps\common
C:\Program Files\Epic Games
```

So the rule should be:

> Exclude broad system roots, but allow specific known app/game roots.

---

## The scanning model

Do not report every folder. That becomes useless.

You want a tree aggregation model:

```text
root
  child folder
    child folder
```

Calculate folder sizes bottom-up, then show only meaningful candidates.

Example output:

```text
Largest user-space folders

1.  186.4 GB  Steam Library
    C:\Program Files (x86)\Steam\steamapps\common
    contains: Baldurs Gate 3, Cyberpunk 2077, Total War WARHAMMER III

2.   92.1 GB  AppData Local
    C:\Users\Ryan\AppData\Local
    largest children:
      31.2 GB  Temp
      18.8 GB  NVIDIA
      12.4 GB  UnrealEngine

3.   64.7 GB  Downloads
    C:\Users\Ryan\Downloads
    likely cleanup: installers, archives, videos

4.   51.3 GB  Mod Organizer
    C:\Users\Ryan\AppData\Local\ModOrganizer
    likely game mods
```

The tool should think in terms of **actionable suspects**, not just directory sizes.

---

## Minimum useful features

## 1. Top folders

```powershell
spacehunter scan --top 30
```

Shows largest directories.

## 2. Group by category

```powershell
spacehunter scan --grouped
```

Categories:

```text
Games
Launchers
AppData
Downloads
Mods
Media
Caches
Unknown large folders
```

## 3. Explain why a folder was included

This is important.

Example:

```text
C:\Users\Ryan\AppData\Local\ModOrganizer
Reason: matched known mod-manager directory
```

or:

```text
C:\XboxGames
Reason: known Xbox app game install location
```

This prevents the user from feeling like the tool is randomly crawling the computer.

## 4. Never delete by default

First version should be read-only.

Maybe later:

```powershell
spacehunter open <id>
spacehunter reveal <id>
spacehunter export report.json
```

But do **not** start with deletion. Too dangerous.

## 5. Open folder in Explorer

This is a killer QoL feature:

```powershell
spacehunter open 4
```

Opens result #4 in File Explorer.

## 6. Export

```powershell
spacehunter scan --json report.json
spacehunter scan --markdown report.md
```

Useful for comparing before/after cleanup.

---

## Rust libraries I’d use

For a beautiful CLI:

```toml
clap = "4"
walkdir = "2"
ignore = "0.4"
rayon = "1"
comfy-table = "7"
indicatif = "0.17"
colored = "2"
humansize = "2"
serde = "1"
serde_json = "1"
dirs = "5"
```

Potentially:

```toml
ratatui = "0.29"
crossterm = "0.28"
```

Use `ratatui` only if you want a full terminal dashboard. For v1, I would avoid it. A clean table + progress bars is better.

---

## Architecture

I would structure it like this:

```text
spacehunter/
  src/
    main.rs
    cli.rs
    scanner.rs
    roots.rs
    filters.rs
    classify.rs
    report.rs
    windows.rs
```

## `roots.rs`

Finds scan roots:

```text
user folders
Steam libraries
Epic installs
XboxGames
GOG games
known launcher folders
```

## `filters.rs`

Decides what not to enter:

```text
skip Windows
skip system folders
skip permission traps
skip symlinks by default
```

## `scanner.rs`

Walks directories and calculates sizes.

## `classify.rs`

Labels results:

```text
Game Library
Launcher
Mod Manager
Downloads
Cache
Media
AppData
Unknown Large Folder
```

## `report.rs`

Prints beautiful results.

---

## Very important Windows rules

## Do not follow symlinks by default

Windows has junctions and symlinks that can create loops or duplicate counts.

Default:

```text
do not follow symlinks
do not follow junctions
```

Maybe add:

```powershell
--follow-links
```

but off by default.

## Permission errors are normal

Do not crash.

Show:

```text
Skipped 42 protected folders
```

Maybe with:

```powershell
--show-skipped
```

## Use long path support carefully

Windows paths can exceed normal limits. Rust handles a lot, but you may eventually need to normalize paths with `\\?\` prefixes. Not a v1 blocker, but keep it in mind.

---

## The killer feature: “targeted scan profiles”

I would design profiles like this:

```text
default
  User folders + known game libraries + known launcher install paths

games
  Steam, Epic, XboxGames, GOG, EA, Ubisoft, Battle.net

appdata
  AppData Local/Roaming/LocalLow, grouped carefully

downloads
  Downloads/Desktop/Documents

media
  Videos/Pictures/Music/Desktop

full-user
  Entire C:\Users\<name>, excluding system-ish traps

paranoid
  Dry run only, extra conservative, no Program Files exceptions
```

Example:

```powershell
spacehunter scan --profile games
```

Output:

```text
Game Storage Report

Total found: 624.2 GB

1. 142.8 GB  Cyberpunk 2077
2. 119.4 GB  Baldur's Gate 3
3. 103.2 GB  Total War WARHAMMER III
4.  88.7 GB  Starfield
5.  51.3 GB  Mod Organizer Skyrim Profile
```

That is way more useful than generic disk analyzer output.

---

## Would Python be too slow?

For a rough test tool? No.

For a polished Windows CLI? I would not choose Python.

The main issue is not just speed. It is:

```text
packaging
startup feel
Windows antivirus false positives from bundled Python apps
parallel traversal overhead
native path weirdness
distribution size
```

Rust gives you a fast native `.exe` that feels like a real utility.

So my recommendation:

```text
Prototype logic in Python only if you want to think fast.
Build the actual tool in Rust.
```

But honestly, you can start directly in Rust. This project is very Rust-friendly.

---

## Good v1 scope

Do not overbuild it.

Version 1 should do this:

```powershell
spacehunter scan
```

And produce:

```text
- top 30 largest targeted folders
- category labels
- reasons included
- skipped/protected count
- total scanned size
- open-in-explorer command hints
```

Commands:

```powershell
spacehunter scan
spacehunter scan --profile games
spacehunter scan --profile appdata
spacehunter scan --top 50
spacehunter open 7
spacehunter export --json report.json
```

That is already a genuinely useful tool.

---

## Name ideas

Since you want creative QoL design:

```text
SpaceHunter
GhostGig
DiskRaven
HoardHound
GameGrave
CacheCrow
ByteBurrow
ClutterFox
```

My strongest pick for the actual tool:

```text
HoardHound
```

It fits the purpose:

> sniff out forgotten piles of disk space.

Example:

```powershell
hoardhound scan --profile games
hoardhound open 3
```

Very good CLI name.

---

## Final recommendation

Build it in **Rust** as a targeted Windows user-space scanner.

The core philosophy should be:

```text
Never scan the whole system by default.
Never delete anything by default.
Prioritize user-owned storage, launchers, games, mods, AppData bloat, downloads, and forgotten installs.
Always explain why a folder was scanned.
Show actionable large folders, not raw filesystem noise.
```

That gives you a tool with an actual identity, instead of just another WinDirStat clone.
