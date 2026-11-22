# TZWorkQuint 3.0.0 — First Public Suite Release

We are excited to announce the first public release of **TZWorkQuint 3.0.0**, bringing together five native, high-performance desktop productivity applications and an independent suite launcher.

Every application runs natively in its own detached operating system process with dedicated window lifecycle management and native GPU rendering.

---

## What's Included in 3.0.0

### Five Specialized Document & Technical Editors
- **TZ Wordloom (`#1971C2`):** Word processor featuring a dedicated typography ruler, font styling, margin controls, and standard `.docx` interoperability.
- **TZ CellForge (`#10B981`):** High-speed spreadsheet studio with a reactive calculation engine, formula bar, multi-sheet workbook support, and `.xlsx` interoperability.
- **TZ SlideSpark (`#7950F2`):** Presentation designer with a slide navigator, layout presets, presenter notes, and smooth presentation delivery.
- **TZ PDFlex (`#E03131`):** PDF reading and annotation workstation with high-fidelity vector rendering, sticky notes, text highlighting, and freehand drawing.
- **TZ Draftline (`#D97706`):** Precision 2D CAD studio with an interactive command line interface, coordinate snapping, and standard `.dxf` drafting tools.
- **TZWorkQuint Hub (`#2563EB`):** Suite launcher and recent project tracker that lets you open documents or launch any editor into an independent window.

### Privacy-Preserving AI Gateway
- **Direct Local & Cloud Inference:** Connect directly to local runners (Ollama, vLLM, LM Studio, LocalAI) without internet connectivity, or connect to OpenAI, Gemini, or Anthropic using private API keys.
- **Strict Data Boundary:** Zero background document telemetry. The assistant only inspects cell ranges or passages you explicitly select and only transmits when you click **Send**.
- **Proposal Review Workflow:** Edit suggestions are returned as interactive cards with clear **Apply** and **Discard** buttons.
- **Deterministic Mock Mode:** Test proposals and assistant workflows completely offline without an LLM backend.

### Native Glass & Visual Refinements
- **Seamless Panel Boundaries:** Completely removed unsightly black frames and panel inset artifacts around the assistant dock.
- **Ergonomic Controls:** Restrained app-colored buttons with separate hit targets for voice input and message sending.
- **Liquid Glass Materials:** Subtle backdrop diffusion on supported compositors with automatic fallback to high-contrast opaque surfaces when reduced transparency is enabled.

---

## Windows x64 Downloads

| Asset | Size | SHA-256 Checksum |
| :--- | :---: | :--- |
| **`TZWorkQuint-3.0.0-Setup.exe`** | 115 MB | `f9e8c3d4eff749dec1a11b641b3722ca57a2cac6210bf1c914d3ac0af62b1eaf` |
| **`TZWorkQuint-3.0.0-win64.msi`** | 144 MB | `ae3825604c439900b55a6a4f850f291b6a6975a5fe7211e422b18d144aca224c` |
| **`TZWorkQuint-3.0.0-win64.zip`** | 184 MB | `01e02efbbfa7821ffde21442ebc8ebdf83bfe4df3fb7d5bc730bef4c169e5158` |

All packages include the complete offline User Manual at `docs/user-guide/index.html`.

For verification:
```sh
sha256sum -c SHA256SUMS.txt
```

---

## Upstream & Licenses
TZWorkQuint is released under open-source licenses. Upstream notices and source attributions are included in [THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt) and component directories.

Developed by [TZLabs](https://tzlabs.in).

