# Changelog

All notable changes to `termfactor` will be documented in this file.

## [Unreleased]

## [0.6.69] - 2026-04-11

### Changed

- Reworked the project into a conventional Rust layout with `src/main.rs` and focused modules for CLI parsing, errors, system detection, platform input, UI flow, leaderboard logic, save paths, and embedded bash generation.
- Moved version reporting to Cargo package metadata so `termfactor --version` stays in sync with `Cargo.toml`.
- Switched leaderboard saves to platform-appropriate app-data directories while still loading legacy local score files when present.
- Centralized the terminal test catalog so the Rust runner and embedded bash fallback share the same checks and score thresholds.
- Added CI coverage for Linux, macOS, and Windows, plus `shellcheck` for the generated bash script.
- Locked CI Cargo commands to `Cargo.lock` so validation uses the checked-in dependency set.
- Added an explicit Rust `1.82` CI job so the declared MSRV is verified by the test suite instead of assumed.
- Added `clippy::pedantic` and `clippy::nursery` coverage to the Rust `1.82` CI job so the zero-warning standard is checked at the declared MSRV too.
- Raised the Unix `nix` dependency to `0.29.*` with the modern termios API and added `unicode-width` for display-correct leaderboard alignment.

### Fixed

- Fixed leaderboard corruption that could truncate the first character of terminal and OS names and create duplicate same-score entries.
- Fixed leaderboard rendering so rows stay compact on screen without fake blank lines between entries.
- Fixed leaderboard storage to use deterministic tab-separated rows for future saves while still parsing legacy spacing-based files.
- Fixed leaderboard saves to reject tabs and line breaks in persisted terminal and OS fields instead of writing rows that would be dropped on reload.
- Fixed leaderboard writes to flush before reporting success.
- Fixed same-family leaderboard replacement to prefer a better score first, then newer OS and data quality as tie-breakers.
- Fixed raw key handling on Windows so Windows Terminal VT input does not cause arrow keys to be treated as `Esc`, and `Ctrl+C` can be handled without leaving the console in a broken mode.
- Fixed Unix key handling so multi-byte escape sequences like arrow keys are consumed cleanly instead of leaving stray bytes in the shell input buffer after a quit attempt.
- Fixed Apple Terminal containment so unsupported underline probes do not bleed formatting into later prompts.
- Fixed embedded bash confirmation handling for `Esc`, `Ctrl+C`, and `Ctrl+D` so quit behavior matches the Rust path more closely.
- Fixed embedded bash reset handling so confirmation responses emit real ANSI resets instead of literal escape text.
- Fixed embedded bash fastfetch parsing to preserve additional `:` content in terminal and OS strings and keep field matching case-insensitive like the Rust path.
- Fixed generated bash output so `shellcheck` stays clean across unused generated variables and shell-quoted terminal payloads.
- Fixed leaderboard replacement writes to use a temp file plus rename, and hardened temp-file naming against concurrent write collisions.
- Fixed configured shell detection so `COMSPEC` is only considered on Windows, and fixed the legacy spacing parser to accept rows whose terminal padding ends exactly at the old width boundary.
- Fixed Windows save-path selection to prefer `LOCALAPPDATA` over roaming `APPDATA`, and made tab-separated leaderboard parsing tolerate a single trailing separator while still rejecting malformed extra fields.
- Fixed leaderboard updates to reject missing terminal or OS identity fields instead of writing blank entries that vanish on the next load.
- Fixed startup validation so missing terminal or OS identity fails before the interactive suite begins.
- Fixed embedded bash temp-script creation to use unique files per invocation instead of reusing a per-process filename.
- Fixed embedded bash escape-key handling so it remains compatible with macOS's default Bash 3.2 while still draining escape sequences on newer Bash versions.
- Fixed shell-version extraction so Rust and embedded bash both preserve versions with more than three numeric components and avoid trailing-dot versions.
- Fixed Unix escape-sequence draining to restore the blocking raw-termios settings before returning.
- Fixed leaderboard migration so an empty preferred file does not shadow an existing legacy scoreboard.
- Fixed leaderboard storage validation to reject control characters that could corrupt terminal rendering.
- Fixed closed-stdin prompt handling so unexpected EOF exits through the normal quit path.
- Fixed help text so the usage header matches the supported option set.

### Tests

- Added regression coverage for leaderboard dedupe, legacy score parsing, legacy migration loading, tab-separated score parsing, whitespace preservation, long legacy terminal names, exact-width legacy padding, invalid storage-field rejection, round-trip write/load behavior, temp-file uniqueness, extra-field rejection, and display-width padding.
- Added generated bash regression coverage for `Esc` quit, EOF quit, prompt redraw behavior, real reset escape emission, case-insensitive fastfetch field extraction, and Rust-vs-bash feature-tier boundary agreement.
- Added regression coverage for empty leaderboard identity rejection and unique embedded bash temp-script creation.
- Added regression coverage for empty preferred-file migration, control-character rejection, Bash 3.2-compatible escape timeout generation, early bash identity validation, and extended shell-version extraction.
- Enforced `cargo test`, `cargo build`, and `clippy::pedantic` plus `clippy::nursery` on native and Windows GNU targets during validation.
