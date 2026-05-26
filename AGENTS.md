# AGENTS.md

## Cursor Cloud specific instructions

### Project overview

`mensa-rs` is a Rust CLI (and optional GUI) application that fetches and displays the daily cafeteria menu for Mensa am Schloss (Mannheim, Germany) from `api.stw-ma.de`.

### Development commands

| Action | Command |
|--------|---------|
| Build CLI | `cargo build` |
| Build GUI | `cargo build --features gui` |
| Run CLI | `cargo run --bin mensa` |
| Run CLI (English) | `cargo run --bin mensa -- --lang en` |
| Run CLI (no cache) | `cargo run --bin mensa -- --no-cache` |
| Lint | `cargo clippy` |
| Format check | `cargo fmt --check` |
| Test | `cargo test` |

### Notes

- The Rust toolchain must be **stable 1.88+** (dependencies require edition2024 and newer features). The VM update script handles `rustup update stable`.
- Clippy is configured with very strict lints in `Cargo.toml` (pedantic, correctness, suspicious, complexity, style, nursery all denied). All new code must pass `cargo clippy` cleanly.
- `cargo fmt --check` currently fails on the existing GUI code (`src/bin/mensa_gui.rs`). This is a pre-existing state in the repo.
- There are no automated tests in the codebase (0 test functions).
- The CLI requires network access to `https://api.stw-ma.de/tl1/menuplan` for fetching menu data. Results are cached per day under `~/.cache/mensa/`.
- The GUI binary (`mensa-gui`) requires the `gui` feature flag and a display server (X11/Wayland).
- There is no database, Docker, or additional service dependency.
