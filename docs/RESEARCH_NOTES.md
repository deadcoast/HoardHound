# Research notes and product decisions

## Why Rust

Rust gives the project a native single executable, strong filesystem/error handling, good parallelism, and a mature CLI ecosystem. Python is fine for prototyping but less ideal for a polished Windows utility that will traverse hundreds of thousands of filesystem entries and ship as one small executable.

## Why not whole-drive MFT scanning first

Tools like WizTree are extremely fast because they use NTFS metadata. HoardHound has a different job: targeted user-space discovery. A whole-drive backend can come later as an optional expert mode, but it should not drive the product shape.

## Modern Windows storage patterns to target

- Steam libraries and `steamapps/common`.
- Epic Launcher manifests and install locations.
- Xbox app `C:\XboxGames`.
- GOG, EA, Ubisoft, Riot, Battle.net-style game folders.
- Minecraft, Roblox, itch.io, CurseForge.
- Vortex, Mod Organizer, Nexus Mod Manager, r2modman, Thunderstore.
- AppData Local/Roaming/LocalLow.
- AppData caches: shader caches, GPU caches, Code Cache, DerivedDataCache, temp/log/crash dump folders.
- Per-user installs in `AppData\Local\Programs`.
- Store packages in `AppData\Local\Packages`.
- Developer dependency/build caches: Gradle, Cargo, NuGet, npm/pnpm/yarn, pip, Docker/WSL artifacts.
- Downloads/Desktop/Documents installers, archives, extracted folders, recordings.
- Videos/Pictures capture/export folders.

## User trust features

- Explain every result.
- Show root reason.
- Show confidence.
- Show conservative advice.
- Do not delete.
- Do not silently scan system directories.
- Make `roots` preview easy.
