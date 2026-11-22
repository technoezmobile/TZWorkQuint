# 00. Build, Environment & Launch Paths

## 1. System Environment & Execution Identification

The **TZWorkQuint 3.0.0** suite was audited and validated in the following runtime environment:

- **Audit Date:** November 20, 2026
- **Host OS:** Windows 10 / Windows 11 64-bit (x86_64)
- **Windows Display Language:** English (United States) / `en-US`
- **Native Framework:** Pure native Rust 1.84+ with `eframe`, `egui` 0.37, `wgpu`, and `winit`. Zero webviews, zero CEF, zero Electron.
- **Package Version:** 3.0.0
- **Vendor & Attribution:** TZLabs (https://tzlabs.in) and upstream contributors.

---

## 2. Installed & Portable Executable Inventory

### Primary Suite Release Binaries

All binaries reside in the portable distribution folder `dist\TZWorkQuint-3.0.0-win64\bin` and are installed to `{autopf}\TZWorkQuint\bin` by the Inno Setup installer. Authenticode status is `NotSigned` (self-built Windows x86_64 binaries); file integrity is verified and guaranteed independently through cryptographic SHA-256 checksums:

| Application | Executable File | File Size | SHA-256 Checksum |
|---|---|---|---|
| **TZ Wordloom** | `bin/tz-wordloom.exe` | 158,141,952 bytes | `d1d3ec51ac0ac0579d24c7c1f010e1144f03ea5af2a4b3ccc97fe81a90a746f3` |
| **TZ CellForge** | `bin/tz-cellforge.exe` | 102,364,160 bytes | `05fa8d609194062bdc9e62d1807e301a40b698302709c7777d1fd5e6d4e44fc6` |
| **TZ SlideSpark** | `bin/tz-slidespark.exe` | 131,703,296 bytes | `410b2df2a65b86e9e80c0a0416414d0effa6058328f05802b5cdb0ae1ce2fbc2` |
| **TZ PDFlex** | `bin/tz-pdflex.exe` | 129,675,776 bytes | `d8fd3ba2c9e53b6c9a44fab36d6b5ff98217cb0a4ca158071301092cdb030f12` |
| **TZ Draftline** | `bin/tz-draftline.exe` | 108,011,520 bytes | `9b9973c292a9a9b2db75d70d4ece8efd8f0e946bb1974b3952ed125aaed41260` |
| **TZWorkQuint Hub** | `bin/tz-workquint.exe` | 27,030,016 bytes | `1a4e5b7bca77439d1fb7f5aeed2f90c8364d54be5b6a036d046ccab061e4b883` |

### Package Deliverables

1. **Single-File Setup Installer:**
   - Path: `dist\TZWorkQuint-3.0.0-Setup.exe`
   - Size: `218,151,445 bytes`
   - SHA-256: `810bf7bfd231579e2a6042a8fe367610e985e865cb171c1918cf0c8f19a3ddaf`
   - Standard Installation Target: `%LOCALAPPDATA%\Programs\TZWorkQuint` (per-user, no UAC prompt needed) or `C:\Program Files\TZWorkQuint` (machine-wide).

2. **Portable ZIP Archive:**
   - Path: `dist\TZWorkQuint-3.0.0-win64.zip`
   - Size: `316,733,676 bytes`
   - SHA-256: `feb7c7e2f683418df2add8d6de05ff8b42d6230d503405fe3d87e7305b607a8c`
   - Requires no installation: unpack to any directory or USB drive and launch `bin\tz-workquint.exe`.

---

## 3. Launch Paths & Sibling Discovery

### Hub Sibling Launch Table

When you click an application card in the **TZWorkQuint Hub**, the Hub detects installed sibling editors in its own `bin` directory using relative resolution. The exact executable mapping is:

| Hub Card Title | Launched Executable Path | Working Directory | Command-line Args |
|---|---|---|---|
| **TZ CellForge** | `bin\tz-cellforge.exe` | `{app}\bin` | (None or document path) |
| **TZ Wordloom** | `bin\tz-wordloom.exe` | `{app}\bin` | (None or document path) |
| **TZ PDFlex** | `bin\tz-pdflex.exe` | `{app}\bin` | (None or document path) |
| **TZ Draftline** | `bin\tz-draftline.exe` | `{app}\bin` | (None or document path) |
| **TZ SlideSpark** | `bin\tz-slidespark.exe` | `{app}\bin` | (None or document path) |

### Start Menu & Desktop Shortcuts

The Inno Setup installer automatically establishes the following Start Menu programs group and desktop shortcuts:

- **Start Menu Folder:** `%APPDATA%\Microsoft\Windows\Start Menu\Programs\TZWorkQuint\`
  - `TZWorkQuint` -> `{app}\bin\tz-workquint.exe`
  - `TZ CellForge` -> `{app}\bin\tz-cellforge.exe`
  - `TZ Wordloom` -> `{app}\bin\tz-wordloom.exe`
  - `TZ PDFlex` -> `{app}\bin\tz-pdflex.exe`
  - `TZ Draftline` -> `{app}\bin\tz-draftline.exe`
  - `TZ SlideSpark` -> `{app}\bin\tz-slidespark.exe`
  - `TZWorkQuint User Guide` -> `{app}\docs\user-guide\index.html`
  - `Uninstall TZWorkQuint` -> `{app}\unins000.exe`
- **Desktop Shortcut:** `%USERPROFILE%\Desktop\TZWorkQuint.lnk` -> `{app}\bin\tz-workquint.exe`

### File Associations & Shell Integration

The installer registers Windows Capabilities and ProgIDs in `HKCU\Software\Classes` and `HKCU\Software\RegisteredApplications`:

- **`.xlsx`**: `TZLabs.CellForge.1` (`"{app}\bin\tz-cellforge.exe" "%1"`)
- **`.docx`**: `TZLabs.Wordloom.1` (`"{app}\bin\tz-wordloom.exe" "%1"`)
- **`.pdf`**: `TZLabs.PDFlex.1` (`"{app}\bin\tz-pdflex.exe" "%1"`)
- **`.dxf`**: `TZLabs.Draftline.1` (`"{app}\bin\tz-draftline.exe" "%1"`)
- **`.pptx`**: `TZLabs.SlideSpark.1` (`"{app}\bin\tz-slidespark.exe" "%1"`)

---

## 4. Multi-Instance Coexistence & Storage Isolation

The suite is engineered for complete operational coexistence. You can run multiple different editors simultaneously (e.g. Wordloom and CellForge side-by-side) with zero lock collisions:

1. **Storage Isolation Directory Structure:**
   - **Hub Settings:** `%APPDATA%\tz.workquint.launcher\data\app.ron`
   - **Wordloom Profile:** `%APPDATA%\TZ Wordloom\settings.ron` (Autosave: `autosave\`)
   - **CellForge Profile:** `%APPDATA%\TZ CellForge\settings.ron` (Recovery: `recovery\`)
   - **SlideSpark Profile:** `%APPDATA%\TZ SlideSpark\settings.ron` (Recovery: `TZ SlideSpark\Recovery\`)
   - **PDFlex Profile:** `%APPDATA%\TZ PDFlex\settings.ron` (Revisions: `revisions\`)
   - **Draftline Profile:** `%APPDATA%\TZ Draftline\settings.ron` (Autosave: `autosave\`)

2. **Single-Instance Handoff Protocol:**
   - Each editor bounds its single-instance socket to a dedicated localhost port and challenge token.
   - If an editor is already running, opening a document file from Explorer immediately hands the file path off to the running instance and focuses the existing window without spawning redundant background processes.

