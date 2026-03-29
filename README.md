# OSCleaner

[![CI](https://github.com/hebertcisco/oscleaner/actions/workflows/ci.yml/badge.svg)](https://github.com/hebertcisco/oscleaner/actions/workflows/ci.yml)

A cross-platform system cleanup CLI tool written in Rust to free up disk space by removing development and system artifacts. It works on both Windows and macOS.

## Core Functionality

-   **Interactive CLI:** Scan, preview, and selectively clean up system and development artifacts.
-   **Disk Space Preview:** Shows the amount of disk space that will be freed before cleaning.
-   **Dry Run Mode:** Preview the files to be deleted without actually deleting them.
-   **Progress and Summary:** Displays progress indicators and summary statistics after cleaning.
-   **Human-Readable Sizes:** Calculates and displays file sizes in a human-readable format (GB, MB, KB).
-   **Colorized Output:** Uses colorized terminal output for better readability.

## Cleanup Categories

`oscleaner` can detect and clean the following categories of files and directories:

### Development Artifacts (Cross-Platform)

-   **Node.js:** `node_modules` directories.
-   **Docker:** Unused images, containers, volumes, and build cache.
-   **Android:** `android/build` and `**/build/` folders in Android projects.
-   **React Native:** `ios/Pods`, `ios/build` (for iOS) and Android build artifacts.
-   **Gradle:** Caches located at `~/.gradle/caches`.
-   **Maven:** Caches located at `~/.m2/repository`.
-   **Rust:** `target/` directories in Cargo projects.
-   **Python:** `__pycache__` directories, `.pyc` files, and virtual environments.
-   **Browser Caches:** Chrome, Firefox, Safari, Edge, and Brave cache directories.

### System Caches (macOS)

-   **Xcode:** Derived data (`~/Library/Developer/Xcode/DerivedData`) and archives (`~/Library/Developer/Xcode/Archives`).
-   **CocoaPods:** Caches at `~/Library/Caches/CocoaPods`.
-   **User Caches:** General user cache files in `~/Library/Caches`.
-   **System Logs:** Log files in `~/Library/Logs` and `/Library/Logs`.
-   **Temporary Files:** Files in `/tmp` and `~/Library/Application Support/CrashReporter`.
-   **iOS Backups:** Old device backups in `~/Library/Application Support/MobileSync/Backup`.
-   **Homebrew:** Cache at `~/Library/Caches/Homebrew`.
-   **Email Attachments:** Downloaded email attachments cache.

### System Caches (Windows)

-   **User Temp:** Temporary files in `%TEMP%`.
-   **Windows Update:** Update cache in `C:\Windows\SoftwareDistribution\Download`.
-   **Thumbnail Cache:** `%LocalAppData%\Microsoft\Windows\Explorer`.
-   **Prefetch Files:** `C:\Windows\Prefetch`.
-   **Windows Error Reporting:** `%ProgramData%\Microsoft\Windows\WER`.

## Installation

### Windows (Chocolatey)

Install the published Chocolatey package (requires an elevated shell):

```powershell
choco install oscleaner
```

Update to the latest version later with:

```powershell
choco upgrade oscleaner
```

### macOS or Linux (Homebrew tap)

```bash
brew tap hebertcisco/homebrew-tap
brew install hebertcisco/homebrew-tap/oscleaner
```

### Cargo (from source)

You can also build and install `oscleaner` locally with Cargo (after cloning this repo):

```bash
cargo install --path .
```

## Usage

`oscleaner` still supports the interactive flow (run `oscleaner` with no args), but you can now drive it entirely via commands and flags.

### Quick examples

-   Interactive prompts: `oscleaner`
-   Scan everything and show a summary: `oscleaner scan`
-   Clean a single category: `oscleaner clean --docker` (or simply `oscleaner --docker`)
-   Target multiple categories and preview: `oscleaner clean --node-modules --cargo-targets --python-cache --dry-run`
-   Non-interactive clean of everything detected (dangerous): `oscleaner clean --all -Y`
-   List available category ids and platforms: `oscleaner list`

### Commands and global flags

-   `list` — print category ids, platform, and descriptions.
-   `scan` — scan selected categories and print a summary (no deletion).
-   `clean` — scan and remove selected categories.
-   Global flags:
    -   `--dry-run` / `-n` — preview deletions without removing files.
    -   `--yes` / `-Y` — skip confirmations (for automation; be careful).
    -   `--all` — include every category available on the current platform.
    -   `--category <id>` — target a category by id (repeatable).
    -   Category shortcuts (use any combination): `--node-modules`, `--docker`, `--xcode`, `--android-builds`, `--react-native-ios`, `--gradle-cache`, `--maven-cache`, `--cargo-targets`, `--python-cache`, `--cocoapods-cache`, `--mac-caches`, `--mac-logs`, `--mac-tmp`, `--ios-backups`, `--homebrew-cache`, `--mail-downloads`, `--windows-temp`, `--windows-update`, `--windows-thumbnail`, `--windows-prefetch`, `--windows-wer`, `--browser-caches`.

If you supply category flags without a subcommand, `oscleaner` will default to running `clean` with those selections.

### Safe mode (`--safe`)

Safe mode is designed for unattended execution (e.g. cron jobs). It applies multiple layers of protection so you can schedule automatic cleanups without worrying about losing important data.

```bash
# Basic safe cleanup
oscleaner --safe

# Preview what would be deleted (no actual deletion)
oscleaner --safe --dry-run

# Custom limits: max 10 GB, only items older than 7 days
oscleaner --safe --max-size 10 --min-age 7
```

**What safe mode does:**

1. **Restricts categories** to only regenerable caches and build artifacts (15 out of 32):
   `node_modules`, `cargo_targets`, `gradle_cache`, `maven_cache`, `php_vendor`, `ruby_vendor`, `python_cache`, `cocoapods_cache`, `android_builds`, `react_native_ios`, `xcode`, `homebrew_cache`, `browser_caches`, `snap_cache`, `flatpak_cache`.
2. **Skips recent files** — only deletes items last modified more than N days ago (default: 2). Override with `--min-age <DAYS>`.
3. **Enforces a size cap** — aborts if the total to be deleted exceeds N GB (default: 20). Override with `--max-size <GB>`.
4. **Protects sensitive paths** — items inside personal directories (`~/Documents`, `~/Desktop`, `~/Downloads`, `~/.ssh`, `~/.gnupg`, `~/.config`, etc.) and system directories (`/System`, `/usr`, `/etc`, `/var`, etc.) are never touched.
5. **Auto-confirms** — `--yes` is implied; no interactive prompts.
6. **Writes a log** — every run appends to `~/.oscleaner/safe_run.log` with a full record of what was processed, skipped, and any errors.

**Categories excluded from safe mode** (and why):

| Category | Reason |
|---|---|
| `docker` | Could remove running containers or images in use |
| `ios_backups` | Irreplaceable device backups |
| `mail_downloads` | May contain important attachments |
| `mac_caches` | Too broad — `~/Library/Caches` includes app state |
| `mac_logs`, `linux_logs`, `linux_journal` | System logs needed for debugging |
| `mac_tmp`, `linux_tmp`, `windows_temp` | May contain files in use by running processes |
| `windows_update` | System-critical update cache |
| `windows_prefetch` | System performance data |
| `windows_wer` | Error reports useful for debugging |
| `linux_coredumps` | Useful for post-mortem debugging |
| `linux_trash` | User may want to recover deleted files |
| `linux_cache` | Too broad — `~/.cache` includes app state |
| `windows_thumbnail` | System UI cache |

**Cron example:**

```bash
# Weekly cleanup every Sunday at 3 AM
0 3 * * 0 /usr/local/bin/oscleaner --safe

# Conservative: 10 GB limit, 7-day age, dry-run logged
0 3 * * 0 /usr/local/bin/oscleaner --safe --max-size 10 --min-age 7 --dry-run
```

## Building from Source

1.  Clone the repository:
    ```bash
    git clone https://github.com/hebertcisco/oscleaner.git
    cd oscleaner
    ```
2.  Build the project:
    ```bash
    cargo build --release
    ```
3.  The executable will be located at `target/release/oscleaner`.

## Contributing

Contributions are welcome! Please feel free to submit a pull request.

1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature/your-feature`).
3.  Make your changes.
4.  Commit your changes (`git commit -am 'Add some feature'`).
5.  Push to the branch (`git push origin feature/your-feature`).
6.  Create a new Pull Request.

## Release automation

Tagged releases trigger `.github/workflows/release.yml`, which builds binaries
for macOS (Apple Silicon and Intel), Linux, and Windows. Artifacts are uploaded
to GitHub Releases and then consumed by the downstream package managers:

- **Chocolatey and WinGet:** Publishing steps in the release workflow update
  the Windows packages when the respective API keys are configured.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
