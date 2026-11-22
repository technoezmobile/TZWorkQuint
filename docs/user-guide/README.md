# TZWorkQuint 3.0.0 User Manual & Feature Reference

Welcome to **TZWorkQuint 3.0.0**, the professional desktop productivity and technical drafting suite built entirely in native Rust (egui/eframe/winit). TZWorkQuint delivers instant startup, optical liquid glass styling, cross-document interoperability with Microsoft Office and AutoCAD standards, local document sovereignty, and integrated offline/gateway AI assistance without webviews or external cloud telemetry.

---

## Suite Architecture & Application Directory

The suite consists of the centralized **Workspace Hub** and five focused native document and technical editors:

```
                                  ┌───────────────────────────────┐
                                  │   TZWorkQuint Hub (Launcher)  │
                                  │       tz-workquint.exe        │
                                  └───────────────┬───────────────┘
                     ┌────────────────────────────┼────────────────────────────┐
                     │                            │                            │
      ┌──────────────┴──────────────┐   ┌─────────┴─────────┐   ┌──────────────┴──────────────┐
      │         TZ Wordloom         │   │    TZ CellForge   │   │        TZ SlideSpark        │
      │       tz-wordloom.exe       │   │  tz-cellforge.exe │   │      tz-slidespark.exe      │
      │  Word Processing & Review   │   │  Sheets & Pivots  │   │   Presentations & Decks     │
      └─────────────────────────────┘   └───────────────────┘   └─────────────────────────────┘
                     │                            │                            │
                     └────────────────────────────┼────────────────────────────┘
                                                  │
                                 ┌────────────────┴────────────────┐
                                 │           TZ PDFlex             │
                                 │         tz-pdflex.exe           │
                                 │    PDF Forms, Edit, Redact      │
                                 └────────────────┬────────────────┘
                                                  │
                                 ┌────────────────┴────────────────┐
                                 │          TZ Draftline           │
                                 │       tz-draftline.exe          │
                                 │      2D CAD & Technical DXF     │
                                 └─────────────────────────────────┘
```

---

## User Manual Sitemap & Navigation

| Document | Topic & Scope | Key Sections |
|---|---|---|
| **[00. Build, Environment & Launch Paths](00_build_and_system.md)** | System requirements, verified builds, launch paths, file associations, and process coexistence. | Windows paths, binary hashes, sibling discovery, protocol handlers. |
| **[01. TZWorkQuint Hub Guide](01_hub_guide.md)** | Workspace launchpad, library view, application filtering, appearance controls, and global settings. | Card grid, compact list, search, sort, Pearl/Smoked themes, about modal. |
| **[02. TZ Wordloom Guide](02_wordloom_guide.md)** | Professional document composition, typography, styles, tables, tracked changes, and Word interoperability. | First five minutes, ribbon reference, styles, comments, review, Aptos fonts. |
| **[03. TZ CellForge Guide](03_cellforge_guide.md)** | Spreadsheet calculation, formulas, PivotTables, sorting/filtering, and Excel interoperability. | Quick start, formula bar, PivotTables, cell styling, charts, roundtrip. |
| **[04. TZ SlideSpark Guide](04_slidespark_guide.md)** | Slide presentations, starter decks, themes, animations, Presenter View, and PowerPoint interoperability. | Starter templates, compacted Design ribbon, layouts, presenter mode. |
| **[05. TZ PDFlex Guide](05_pdflex_guide.md)** | PDF inspection, annotation, forms, text editing, page organization, and permanent redaction. | Navigation, annotation bar, AcroForms, zero-leak redaction, OCR. |
| **[06. TZ Draftline Guide](06_draftline_guide.md)** | 2D technical CAD, entity geometry, layers, snaps, ViewCube, and command line autocomplete. | First drawing, CLI input, object snaps, layers, dimensions, DXF export. |
| **[07. Using TZ Assistant](07_using_tz_assistant.md)** | Standalone guide to AI-assisted editing, gateway setup, scoped context, and proposal workflows. | Gateway setup, docked dock, prompt scope, 5 editor worked examples. |
| **[08. Customization, Themes & Workspaces](08_customization_and_themes.md)** | Optical material controls (Pearl Glass, Smoked Glass, Glass Off), layouts, and presets. | Liquid glass calibration, specular highlights, reduced motion, persistence. |
| **[09. Templates, Fonts & Portability](09_templates_and_fonts.md)** | Built-in starter decks, templates, Aptos cloud font discovery, and cross-machine portability. | Starters, custom templates, CloudFonts discovery, font substitution. |
| **[10. Troubleshooting & System Recovery](10_troubleshooting.md)** | Error diagnostics, recovery files, gateway timeouts, clipped controls, and repair procedures. | Autosave recovery, gateway diagnostics, lockfiles, command overflow. |

---

## QA & Independent Inspection Records

Inspection evidence and verification artifacts are maintained under `artifacts/documentation-verification/`:
- **Coverage Ledger:** [coverage.csv](../../artifacts/documentation-verification/coverage.csv) — 2,035 inventoried commands and options with stable IDs, routes, and verified status.
- **Replay Checklist:** [replay-checklist.md](../../artifacts/documentation-verification/replay-checklist.md) — Step-by-step verification procedures by application.
- **Gaps Log:** [gaps-log.md](../../artifacts/documentation-verification/gaps-log.md) — Audited product behavior distinctions (Source vs UI vs Executed).
- **Handoff Note:** [handoff-note.md](../../artifacts/documentation-verification/handoff-note.md) — Summary for independent verification inspectors.

