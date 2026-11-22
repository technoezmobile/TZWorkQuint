# Changelog

All notable changes to **TZWorkQuint** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [3.0.0] — 2026-11-22

### Added
- **Unified Productivity Suite:** First public release of TZWorkQuint bundling five native desktop applications with an independent hub launcher:
  - **TZWorkQuint Hub (`#2563EB`):** Decoupled process launcher, recent project tracker, global preferences, and suite assistant.
  - **TZ Wordloom (`#1971C2`):** Word processor with typography ruler, character/paragraph formatting, and `.docx` import/export.
  - **TZ CellForge (`#10B981`):** Spreadsheet studio with high-performance formula calculation engine, multi-sheet workbook navigation, and `.xlsx` import/export.
  - **TZ SlideSpark (`#7950F2`):** Presentation designer with slide navigator, layout templates, speaker notes, and presentation delivery.
  - **TZ PDFlex (`#E03131`):** PDF viewer and annotation workstation with text highlight, sticky notes, and freehand markup.
  - **TZ Draftline (`#D97706`):** 2D CAD drafting workbench with command line input, polar snapping, and standard `.dxf` geometry support.
- **Privacy-Preserving AI Gateway:**
  - Configurable endpoints for local runners (Ollama, vLLM, LM Studio, LocalAI) and cloud providers (OpenAI, Gemini, Anthropic).
  - Explicit user data boundary: only highlighted or selected ranges (e.g. `D4:D9`) are transmitted when pressing Send; document body is never uploaded in the background.
  - Integrated **Proposal Review Card** with one-click **Apply** and **Discard** actions.
  - Built-in **Deterministic Mock Mode** allowing full assistant UI verification without an active LLM backend or network connection.
- **Packaging & Multi-Platform Distribution:**
  - **Windows x64 & ARM64:** Inno Setup graphical installer (`Setup.exe`), WiX 3 enterprise installer (`.msi`), and standalone portable `.zip` bundle.
  - **macOS Apple Silicon & Intel:** Native application bundle (`TZWorkQuint.app`), signed dmg image generator (`generate_dmg.py`), and portable `.tar.gz`.
  - **Linux x86_64 & ARM64:** Standalone Debian packages (`.deb`) and portable `.tar.gz` with XDG desktop integration entries for all six applications.
  - **Automated CI/CD:** Complete GitHub Actions release workflow (`.github/workflows/release.yml`) building, validating, checksumming, and publishing release artifacts across all platform matrices.
- **Documentation:**
  - Comprehensive user guides for all five editors and the launcher under `docs/`.
  - Dedicated AI setup guide (`docs/ai-setup.md`) with synthetic test workflows.
  - Contributor guide (`CONTRIBUTING.md`) and GitHub issue forms (`bug_report.yml`, `feature_request.yml`).
  - Offline HTML User Manual (`docs/user-guide/index.html`) shipped inside every binary package.

### Changed
- **Native Visual Redesign:**
  - Rebuilt all editor assistant docks to eliminate unsightly black frame boundaries and panel inset artifacts.
  - Implemented continuous dock surface painting with consistent internal padding and subtle resize dividers.
  - Unified assistant composer controls with ergonomic 30×30 rounded hit targets and independent actions for voice input and message submission.
  - Introduced liquid glass backdrop diffusion on supported compositors with automatic fallback to high-contrast opaque surfaces when reduced transparency is requested.
  - Upgraded backstage dialogs (About, Open/New, Unsaved Changes confirmation) to follow approved typography scales and brand accent styling.

### Fixed
- Fixed process locking when launching editors from the hub by detaching child processes (`CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS`).
- Fixed assistant dock resize handles not responding near window borders.
- Fixed formula recalculation synchronization when applying assistant proposals to target cell ranges in CellForge.

