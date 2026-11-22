# 10. Troubleshooting & System Recovery

TZWorkQuint is engineered for resilience, isolation, and predictability. Because the suite is compiled directly to native 64-bit machine code with zero external runtime dependencies (no Electron, no Chromium, no Node.js, and no Python runtimes), troubleshooting is straightforward and fully deterministic.

This guide details common diagnostic scenarios, operational solutions, and data recovery procedures.

---

## 1. Disabled or Greyed-Out Commands

In TZWorkQuint, buttons and ribbon tools are dynamically enabled or disabled based on document selection and application state. If a tool appears greyed out, verify the following prerequisites:

| Feature / Command | Required Application State | Why It May Be Disabled |
|---|---|---|
| **Table Design & Layout Tabs** | Text cursor must be positioned inside a table cell. | Disappears automatically when cursor is outside table boundaries. |
| **Formula Insert / Functions** | A spreadsheet cell or formula bar must have active focus. | Disabled when viewing charts or during full-row header selection. |
| **Ungroup / Break Group** | Selection must contain at least one composite group. | Disabled when selecting a single primitive shape or empty area. |
| **Apply Permanent Redaction** | At least one draft redaction rectangle or search mark must exist. | Disabled in PDFlex if no regions are marked for redaction. |
| **Track Changes / Accept / Reject** | Document must have tracked revisions active or pending changes. | Disabled if the document has no unresolved editorial revisions. |
| **Entity Trim / Extend** | At least two intersecting CAD entities must be selected. | Disabled in Draftline if boundary edges are parallel or non-intersecting. |

---

## 2. Missing Contextual Tabs & Hidden Panes

### Contextual Ribbon Tabs
Certain specialized toolsets only appear in the ribbon when an associated element is selected:
- **Table Tools (Design & Layout):** Appears when clicking inside any table.
- **Picture Format:** Appears when selecting an embedded image.
- **Chart Design & Format:** Appears when clicking a chart in CellForge or SlideSpark.
- **PivotTable Analyze:** Appears when navigating inside a PivotTable range.

*To dismiss a contextual tab, click anywhere on the general document background.*

### Restoring Hidden Docks and Panels
If you accidentally close a side panel:
- **TZ Assistant Dock:** Click the **`✨ TZ Assistant`** button in the title bar or press <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>A</kbd>.
- **Slide Thumbnail Sorter (SlideSpark):** Go to **View** › **Thumbnails** or press <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>T</kbd>.
- **Layer & Object Manager (Draftline):** Go to **View** › **Layers** or type `LAYER` in the command bar.
- **Bookmarks & Pages (PDFlex):** Click the **Sidebar** icon in the left toolbar or press <kbd>F4</kbd>.

---

## 3. Responsive UI at Compact Resolutions (1040 × 760)

TZWorkQuint is certified to run on display viewports down to **1040 × 760 px** (such as small laptops or half-screen snapped windows):

- **Ribbon Chunk Collapsing:** When window width falls below 1200 px, ribbon button groups automatically compress their text labels into compact icon stacks.
- **Horizontal Overflow Buttons:** If a ribbon tab exceeds window width, a subtle **`›` (More Tools)** overflow button appears on the right edge. Clicking it drops down a tidy popover containing the hidden commands.
- **Assistant Panel Clamping:** When opening the TZ Assistant dock at 1040 px width, the active canvas contracts smoothly to maintain a minimum readable width of 700 px without obscuring rulers or CAD input fields.

---

## 4. AI Gateway Connection Errors & Diagnostics

If TZ Assistant displays an error banner when sending prompts:

```
┌────────────────────────────────────────────────────────┐
│  ⚠️ AI Gateway: Connection Refused (port 11434)        │
│  Unable to connect to local Ollama server.             │
│  [ Open Gateway Settings ]    [ Retry Connection ]     │
└────────────────────────────────────────────────────────┘
```

### Diagnostic Solutions:
1. **Connection Refused (`ERR_CONNECTION_REFUSED`):**
   - If using **Ollama**: Verify the daemon is running by opening PowerShell and executing `curl http://localhost:11434`. If it fails, start Ollama from the Windows Start menu.
   - If using **LM Studio**: Ensure the local server is started on port `1234` and the "CORS" switch is enabled.
2. **Unauthorized (`HTTP 401 / 403`):**
   - The API key for Gemini or OpenAI is incorrect, expired, or has insufficient quota. Open **File › Options › TZ AI Gateway**, re-enter your secret key, and click **Test Connection**.
3. **Model Not Found (`HTTP 404`):**
   - The configured model name does not exist on the local engine. For Ollama, run `ollama list` in PowerShell to see pulled models, and update the model string in Gateway settings (e.g. `llama3.2:3b`).
4. **Timeout (`ERR_TIMEOUT`):**
   - High local CPU/GPU load or oversized prompt context. Reduce the context scope in the Assistant composer from **Full Document** to **Selection Only**.

---

## 5. Crash Recovery & Autosave Snapshots

TZWorkQuint runs an automated background snapshot worker every **5 minutes** (configurable in **Preferences › Save & Backup**).

### Recovery Procedure
In the event of an abrupt system power loss or forced process termination:
1. Re-launch the affected application (e.g. `tz-wordloom.exe`).
2. On startup, the application checks for snapshot journals in:
   ```
   %APPDATA%\<app>\recovery\
   ```
3. If an unsaved document journal is detected, a **Document Recovery Bar** appears across the top of the window:
   ```
   ┌────────────────────────────────────────────────────────────────────────┐
   │ ℹ️ Document Recovery: Unsaved changes from Nov 20, 16:42 recovered.     │
   │ [ Open Recovered Version ]    [ Discard ]    [ Compare Changes ]       │
   └────────────────────────────────────────────────────────────────────────┘
   ```
4. Click **Open Recovered Version** to review the restored document, then press <kbd>Ctrl</kbd>+<kbd>S</kbd> to save it to your permanent storage location.

### Manual Lockfile Cleanup
If an application was terminated abruptly while editing a network document, a temporary lockfile (`.~lock.filename.docx#`) may remain:
- If prompted that a file is "Locked by another session", verify no other instance of the editor is running in Task Manager.
- Delete the small hidden `.~lock.*` file in the same directory as your document to clear the lock.

