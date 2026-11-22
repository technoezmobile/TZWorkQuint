# 09. Templates, Fonts & Portability

TZWorkQuint is engineered for complete offline portability and zero-friction cross-platform document exchange. Whether opening documents authored in Microsoft Office 365, AutoCAD 2024, or Adobe Acrobat, or sharing your files with colleagues on other operating systems, TZWorkQuint guarantees visual fidelity, automated template bootstrapping, and intelligent cloud font discovery.

---

## 1. Built-in Starters & Starter Decks

When launching an editor without specifying a file, or choosing **File › New**, TZWorkQuint presents a curated collection of production-ready templates:

### SlideSpark Starter Decks
SlideSpark provides six meticulously designed presentation decks built on clean master slide layouts and modern 16:9 typography:

1. **Executive Pitch:** 12-slide investor deck with problem/solution, TAM calculation, unit economics, and team overview.
2. **Technical Architecture:** Dark-mode blueprint deck featuring sequence flow diagrams, API specs, and service topology cards.
3. **Financial Quarterly Review:** Data-dense corporate deck with pre-formatted revenue tables, KPI metric callouts, and waterfall charts.
4. **Product Roadmap & Strategy:** Gantt-style quarter timelines, feature prioritization matrices, and release candidate schedules.
5. **Modern Portfolio:** Clean visual showcase with asymmetric grid layouts, client testimonial quotes, and project case study cards.
6. **Education & Workshop:** High-contrast instructional deck with agenda tracks, exercise prompts, and summary takeaway frames.

### Wordloom Starters
- **Executive Briefing:** Clean two-page memo with metadata header, executive summary box, action item table, and sign-off block.
- **Technical Specification:** Formal engineering document with auto-numbered heading hierarchy, code block styles, and figure reference tables.
- **Business Correspondence:** Standard professional letter conforming to international business formatting standards.

### CellForge Starters
- **SaaS Financial Operating Model:** Complete 3-statement forecast with MRR waterfalls, cohort churn analysis, and automated dynamic charts.
- **Project Budget & Variance Tracker:** Multi-department expense ledger with pre-configured PivotTables and conditional formatting variance alerts.
- **Sales Pipeline & Commission Calculator:** Lead stage tracking with win-probability weighting and automated SUMIFS regional totals.

### Draftline Starters
- **ISO Metric Standard (A1/A3):** Standard engineering border with title block attributes, metric projection symbol, and pre-set dimension styles.
- **Architectural Plan (Metric):** Layered starter with pre-configured wall, door, window, dimension, and electrical annotation layers.

---

## 2. User Templates & Custom Formats

You can create, organize, and reuse your own organizational templates across all editors:

```
┌────────────────────────────────────────────────────────────────────────┐
│                      Template Formats & Directories                    │
├──────────────┬─────────────────┬───────────────────────────────────────┤
│ Application  │ Template Format │ Storage Directory                     │
├──────────────┼─────────────────┼───────────────────────────────────────┤
│ Wordloom     │ .dotx / .dot    │ %APPDATA%\tz.wordloom\templates\      │
│ CellForge    │ .xltx / .xlt    │ %APPDATA%\tz.cellforge\templates\     │
│ SlideSpark   │ .potx / .pot    │ %APPDATA%\tz.slidespark\templates\    │
│ Draftline    │ .dwt / .dxf     │ %APPDATA%\tz.draftline\templates\     │
└──────────────┴─────────────────┴───────────────────────────────────────┘
```

### Creating a Template
1. Create and style your document with desired boilerplate text, master slides, formulas, or CAD layers.
2. Click **File** › **Save As Template...**
3. Enter a template name. The file is saved automatically into the application template repository.
4. On future launches, your custom template appears in the **File › New › Custom Templates** gallery.

---

## 3. Font Handling & Microsoft CloudFonts Discovery

### The Microsoft Aptos Font Family
Starting in 2024, Microsoft Office standardized on **Aptos** (formerly Bierstadt) as its default typeface across Word, Excel, and PowerPoint. To guarantee pixel-perfect typographic parity when opening `.docx`, `.xlsx`, and `.pptx` documents without requiring manual font installation, TZWorkQuint includes an intelligent font discovery engine:

1. **Automatic CloudFonts Scanning:** On startup, TZWorkQuint scans the Windows local user cloud font cache:
   ```
   %LOCALAPPDATA%\Microsoft\FontCache\4\CloudFonts\
   ```
2. **Discovered Variants:** When present, TZWorkQuint automatically registers:
   - `Aptos` (Regular, Bold, Italic, Bold Italic)
   - `Aptos Display` (for prominent presentation headlines)
   - `Aptos Mono` (for code listings and technical data tables)
   - `Aptos Serif` (for formal editorial correspondence)
   - `Aptos Narrow` (for dense spreadsheet tables)
3. **Zero Configuration:** If Office 365 or Office 2024 is installed on your machine, Aptos renders natively in TZWorkQuint with identical line heights, kerning, and advance widths as Microsoft Office.

### Bundled Open-Source Fallback Fonts
If Microsoft CloudFonts are not present on the host system (such as on clean Windows installations or offline air-gapped workstations), TZWorkQuint loads embedded SIL Open Font License (OFL) typefaces:
- **Inter:** Clean, modern neo-grotesque interface font engineered for computer screens.
- **Liberation Sans & Liberation Serif:** Exact metric-compatible substitutes for Arial and Times New Roman.
- **JetBrains Mono & Fira Code:** Monospaced typefaces with programming ligatures for formula editing and code blocks.

### Non-Destructive Font Substitution Rules
When opening a document that references a font not installed on the system:
- TZWorkQuint displays a subtle notification in the status bar: `Font 'Segoe UI Variable' substituted with 'Inter' (Display Only)`.
- **Non-Destructive Storage:** The original font name is preserved intact in the underlying document XML (`w:rFonts`, `a:rPr`, `dxf:style`).
- When saving and re-opening the file in Microsoft Office or AutoCAD on another workstation, the original font remains fully assigned and unaltered.

