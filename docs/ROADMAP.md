# HoardHound roadmap

## v0.1 - source scaffold

- [x] Cargo project layout
- [x] CLI profiles
- [x] Targeted root discovery
- [x] Steam `libraryfolders.vdf` parsing
- [x] Epic manifest install-location parsing
- [x] Safe recursive scanner
- [x] Category heuristics
- [x] Terminal report
- [x] JSON/Markdown export
- [x] Last-scan cache
- [x] `open <ID>` command

## v0.2 - compile hardening on Windows

- [x] Run `cargo fmt`, `cargo clippy`, and `cargo test` on Windows.
- [ ] Fix path/ANSI issues specific to Windows Terminal, PowerShell, and legacy consoles.
- [x] Add integration tests with synthetic Windows-like directory trees.
- [ ] Add `--threads` and per-drive scan scheduling.
- [x] Add `--plain`/ASCII output mode separate from `--no-color`.
- [ ] Include signed release builds through GitHub Actions.

## v0.3 - launcher intelligence

- [ ] Battle.net config discovery.
- [ ] Ubisoft Connect config discovery.
- [ ] EA app/Origin config discovery.
- [ ] GOG Galaxy database discovery.
- [ ] Xbox app package metadata hints.
- [ ] Better Microsoft Store package display names.

## v0.4 - smarter reclaimability

- [ ] Detect duplicate installers/archives.
- [ ] Identify stale folders by last modified time.
- [ ] Split results into `uninstall through launcher`, `review cache`, `review media`, `developer cache`, `unknown`.
- [ ] Add `risk` and `confidence` scoring separately.
- [ ] Add per-folder evidence snippets.

## v0.5 - UX polish

- [ ] Interactive TUI mode.
- [ ] Incremental live results while scanning.
- [ ] Rich tree drill-down.
- [ ] Compare reports before/after cleanup.
- [ ] Config file for custom roots and exclusions.
- [ ] Shell completions.

## v1.0 - stable release

- [ ] MSIX/winget/chocolatey distribution.
- [ ] Signed binaries.
- [ ] Snapshot-tested output.
- [ ] Windows documentation site.
- [ ] Conservative cleanup assistant that opens owning launcher/app rather than deleting files.
