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
