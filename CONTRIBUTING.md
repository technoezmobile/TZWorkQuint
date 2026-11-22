# Contributing to TZWorkQuint

Thank you for your interest in contributing to **TZWorkQuint**! We welcome bug fixes, documentation corrections, accessibility enhancements, performance optimizations, and platform packaging help.

---

## 1. Codebase Architecture

The suite is structured as a collection of modular Rust workspaces:

| Component Folder | Application | Binary Name | Focus |
| :--- | :--- | :--- | :--- |
| `tz-workquint` | Suite Launcher / Hub | `tz-workquint` | Desktop shell, process manager, settings, recent files |
| `tz-cellforge` | Spreadsheet Studio | `tz-cellforge` | High-performance formula engine, XLSX compatibility |
| `tz-wordloom` | Word Processor | `tz-wordloom` | Page typography, layout ruler, DOCX compatibility |
| `tz-slidespark` | Presentation Studio | `tz-slidespark` | Slide layout, presenter notes, PPTX compatibility |
| `tz-pdflex` | PDF Markup Studio | `tz-pdflex` | Vector PDF rendering, annotations, highlights |
| `tz-draftline` | 2D CAD Studio | `tz-draftline` | CAD viewport, CLI, DXF geometry modeling |

Each component contains its own `crates/` (e.g. `ui-egui`, `engine`, `ai`, `storage`) and `apps/` directories.

---

## 2. Setting Up Your Development Environment

### Prerequisites
1. **Rust Toolchain:** Install stable Rust via [rustup](https://rustup.rs):
   ```sh
   rustup default stable
   ```
2. **Platform C/C++ Toolchain:**
   - **Windows:** Visual Studio 2022 Build Tools (MSVC) or `llvm-mingw`.
   - **macOS:** Xcode Command Line Tools (`xcode-select --install`).
   - **Linux (Ubuntu/Debian):**
     ```sh
     sudo apt-get update
     sudo apt-get install -y build-essential libfontconfig1-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev
     ```

---

## 3. Building and Running From Source

You can build and test individual components independently:

### CellForge (Spreadsheets)
```sh
cd tz-cellforge
cargo test --locked
cargo run --bin tz-cellforge
```

### Wordloom (Word Processor)
```sh
cd tz-wordloom
cargo test --locked
cargo run --bin tz-wordloom
```

### Suite Launcher
```sh
cd tz-workquint
cargo test --locked
cargo run --bin tz-workquint
```

---

## 4. Testing with a Disposable Profile

When testing UI state, window resizing, or assistant settings, **never pollute your real personal profile**. Run with a temporary isolated profile directory:

```powershell
# PowerShell
$env:TZ_PORTABLE_DATA_DIR = "target/dev_profile"
cargo run --bin tz-cellforge
```

```bash
# Bash / Zsh
export TZ_PORTABLE_DATA_DIR="target/dev_profile"
cargo run --bin tz-cellforge
```

---

## 5. Submitting Pull Requests

1. **Focused Changes:** Keep pull requests focused on a single bug fix or feature.
2. **Synthetic Data Only:** Never submit test fixtures containing personal information, real customer names, or copyrighted media. Always use synthetic documents.
3. **Preserve Upstream Attribution:** Each component directory contains its own `ATTRIBUTION.md` and license notices. Do not alter existing copyright notices.
4. **Run Checks Before Opening PR:**
   ```sh
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```

