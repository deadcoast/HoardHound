# Repository Guidelines

## Project Structure & Module Organization

HoardHound is a Rust 2021 Windows-first CLI. Source lives in `src/`, with `main.rs` wiring commands into focused modules: `cli.rs` for the Clap interface, `roots.rs` for targeted scan-root discovery, `scanner.rs` for read-only traversal and ranking, `filters.rs` for safety exclusions, `classify.rs` for category heuristics, `report.rs` for terminal/JSON/Markdown output, and `windows.rs` for Windows helpers. Project design notes are in `docs/`, especially `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, and `docs/RESEARCH.md`. There is no separate assets directory.

## Build, Test, and Development Commands

- `cargo build`: compile the debug build during development.
- `cargo run -- scan --profile games --top 50`: run the CLI locally with a representative scan.
- `cargo test --all-features`: run all Rust tests, including inline module tests.
- `cargo fmt --check`: verify formatting matches `rustfmt`.
- `cargo clippy --all-targets --all-features -- -D warnings`: run lint checks at the same strictness as CI.
- `cargo build --release`: build the optimized `target/release/hoardhound.exe`.

## Coding Style & Naming Conventions

Use standard `rustfmt` formatting and Rust idioms. Prefer small modules with explicit responsibilities, descriptive `snake_case` functions, `PascalCase` types/enums, and `SCREAMING_SNAKE_CASE` constants. Keep Windows-specific behavior isolated in `windows.rs` or clearly named helpers. Error paths should use `anyhow::Context` where extra path or argument detail helps users diagnose failures.

## Testing Guidelines

Tests currently live inline in modules under `#[cfg(test)] mod tests`, with descriptive `#[test]` names such as `parses_common_sizes`. Add focused unit tests near parsing, formatting, classification, and safety-filter logic. For scanner changes, prefer tests that use temporary directories via `tempfile` and avoid relying on machine-specific Windows folders.

## Commit & Pull Request Guidelines

Git history currently contains only `first commit`, so there is no established project-specific convention. Use concise imperative subjects, for example `Add launcher root filtering tests`. PRs should describe the user-facing behavior, list validation commands run, link related issues, and include sample CLI output or screenshots when terminal output changes.

## Safety & Configuration Tips

Preserve the v1 safety model: no delete command, no broad system crawling, no symlink or junction following by default, and clear reasons for every reported result. Treat Windows path handling and launcher manifest parsing as high-risk areas that need explicit tests.
