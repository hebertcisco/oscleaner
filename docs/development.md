# Development guide

## Architecture at a glance

- `main.rs`: Orchestrates the interactive flow end to end.
- `cli.rs`: Renders the banner/prompts and parses commands/flags (via `clap`) including category shortcuts, `--all`, `--category`, `--dry-run`/`-n`, `--yes`/`-Y`, and `--safe`.
- `context.rs`: Builds `ScanContext` with OS detection, home/temp paths, and search roots.
- `categories.rs`: Maps cleanup categories to detectors with platform guards.
- `detectors.rs`: Platform-aware finders using `walkdir` to locate caches and build artifacts.
- `scanner.rs`: Runs detectors, deduplicates paths, computes sizes, and produces summaries.
- `cleanup.rs`: Deletes or previews items with progress bars and a final report.
- `safe.rs`: Safe mode logic — safe category whitelist, protected path definitions, age/size filtering, and log writing.
- `fs_utils.rs`: Helpers for directory search, size calculation, and shortening long paths.
- `types.rs`: Shared enums and structs used across the pipeline.

## CLI flow

1. Parse `CliOptions` via `clap` (commands: `list`, `scan`, `clean`; flags: `--all`, `--category`, category shortcuts, `--dry-run`/`-n`, `--yes`/`-Y`, `--safe`, `--max-size`, `--min-age`).
2. If the `list` command is used, print category ids/platforms/descriptions and exit.
3. Build `ScanContext` (OS, home/temp/app data, search roots) and show the banner.
4. Choose categories to scan based on CLI selections (or interactive selection when no selection was provided).
5. Run detectors and size each hit (files with zero bytes are skipped); show summaries sorted by total size.
6. **Safe mode** (when `--safe` is active): filter findings to remove items inside protected paths and items younger than the minimum age. Abort if total size exceeds the configured limit.
7. For cleaning, auto-select categories when provided via CLI or `--yes`/`--all`/`--safe`, otherwise prompt; print the potential reclaim.
8. Apply dry-run or cleanup respecting `--dry-run` and `--yes`, then show the per-item progress and final report.
9. **Safe mode**: append a detailed log entry to `~/.oscleaner/safe_run.log` with processed items, skipped items, and results.

### Safe mode pipeline

When `--safe` is passed, the clean flow adds three filtering stages between scan and deletion:

1. **Category restriction** — `resolve_category_ids` intersects the requested categories with the safe whitelist defined in `safe::SAFE_CATEGORY_IDS` (15 regenerable categories only).
2. **Finding filter** — `safe::filter_safe` removes findings that are inside protected paths (`~/Documents`, `~/.ssh`, `/System`, etc.) or whose last modification time is within the minimum age window (default: 2 days).
3. **Size limit check** — `safe::check_size_limit` aborts the run if the remaining total exceeds the configured maximum (default: 20 GB).

The `--max-size <GB>` and `--min-age <DAYS>` flags allow overriding the defaults and can also be used outside of safe mode.

## Search and detection rules

- Search roots (in order): current working directory, `~/Projects`, `~/projects`, `~/code`, `~/src`, `~/dev` when present, then the home directory. Roots are deduplicated.
- Depth limits keep scans quick:
  - `node_modules` up to 5 levels deep; `target` up to 4.
  - Android and React Native build artifacts: walk up to depth 6 and match `build`/`Pods` under expected parents.
- Browser caches and OS caches use explicit paths per platform (e.g., Xcode DerivedData on macOS, `%TEMP%` on Windows).
- The scanner deduplicates paths across categories and only records items with a size greater than zero.

## Extending cleanup coverage

- Add a detector function in `detectors.rs` that returns `Vec<PathBuf>` for the new target.
- Register it in `categories::build_categories` with a unique `id`, descriptive `name`, and `Platform` guard.
- Update documentation and, when possible, add focused tests that exercise detection or size/sorting logic.

## Testing and tooling

- Run the suite locally with `cargo test --locked --all-features`.
- Manual smoke test: `cargo run --release -- --dry-run` to exercise the interactive flow without deleting files.
- Standard Rust tooling (`cargo fmt`, `cargo clippy`) is recommended before opening a PR, even though not enforced here.

## CI and releases

- `.github/workflows/ci.yml` builds and tests on pushes and pull requests to `main` and `develop`.
- `.github/workflows/release.yml` builds release binaries for Linux, Windows, and macOS on tags (or manual dispatch), packages artifacts (zip/tar.gz/dmg), and publishes a GitHub Release. Optional steps publish to WinGet, Chocolatey, and Homebrew when credentials are configured.
