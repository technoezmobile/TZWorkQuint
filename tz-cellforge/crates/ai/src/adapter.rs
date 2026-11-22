//! Workbook range adapter for cell extraction, subrange slicing, and preview diffs.

use std::collections::BTreeMap;
use gridcraft_core::{CellRef, RangeRef};
use serde_json::Value;

/// Strict bounds to protect against memory exhaustion, runaway iteration, and excessive allocations.
pub const MAX_SELECTION_CELLS: u64 = 5_000;
pub const MAX_READ_CELLS: u64 = 5_000;
pub const MAX_PROPOSAL_CELLS: u64 = 1_000;

/// Snapshot data for an individual workbook cell.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RangeCellData {
    /// Evaluated / display value (number, text, bool, or null)
    pub value: Value,
    /// Formula string (e.g. "=SUM(B1:B5)") if present
    pub formula: Option<String>,
}

impl RangeCellData {
    pub fn new(value: Value, formula: Option<String>) -> Self {
        Self { value, formula }
    }

    pub fn from_value(value: Value) -> Self {
        Self { value, formula: None }
    }

    pub fn from_formula(formula: impl Into<String>, evaluated_value: Value) -> Self {
        Self {
            value: evaluated_value,
            formula: Some(formula.into()),
        }
    }

    /// Returns the cell representation honoring the `formulas` flag.
    pub fn to_value(&self, formulas: bool) -> Value {
        if formulas {
            if let Some(f) = &self.formula {
                Value::String(f.clone())
            } else {
                self.value.clone()
            }
        } else {
            self.value.clone()
        }
    }
}

/// In-memory snapshot adapter for workbook ranges.
///
/// Enforces:
/// 1. Subrange reads return only the requested cells in proper row-major order.
/// 2. Values vs formulas option is honored.
/// 3. Preview reads return existing contents of approved write targets (including totals rows).
/// 4. When context sharing is disabled or outside selection, cells are kept out of provider context.
#[derive(Clone, Debug)]
pub struct WorkbookRangeAdapter {
    pub sheet_name: String,
    pub authorized_selection: RangeRef,
    pub include_selection_context: bool,
    pub totals_destination: Option<RangeRef>,
    pub cells: BTreeMap<(u32, u32), RangeCellData>,
}

impl WorkbookRangeAdapter {
    pub fn new(
        sheet_name: impl Into<String>,
        authorized_selection: RangeRef,
        include_selection_context: bool,
        totals_destination: Option<RangeRef>,
    ) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            authorized_selection,
            include_selection_context,
            totals_destination,
            cells: BTreeMap::new(),
        }
    }

    /// Inserts or updates cell data at (row, col).
    pub fn set_cell(&mut self, row: u32, col: u32, data: RangeCellData) {
        self.cells.insert((row, col), data);
    }

    /// Inserts cell data by A1 string (e.g. "B6").
    pub fn set_cell_a1(&mut self, a1: &str, data: RangeCellData) -> Result<(), String> {
        let cr = CellRef::parse(a1)
            .ok_or_else(|| format!("Invalid cell reference '{a1}'"))?;
        self.set_cell(cr.row, cr.col, data);
        Ok(())
    }

    /// Reads an individual cell, defaulting to `Value::Null` if not present.
    pub fn read_cell(&self, row: u32, col: u32, formulas: bool) -> Value {
        self.cells
            .get(&(row, col))
            .map(|c| c.to_value(formulas))
            .unwrap_or(Value::Null)
    }

    /// Returns the approved write scope (selection unioned with totals destination if present).
    pub fn approved_write_scope(&self) -> RangeRef {
        if let Some(t) = &self.totals_destination {
            self.authorized_selection.union(t)
        } else {
            self.authorized_selection
        }
    }

    /// Checks if a proposed write target is fully contained within the approved write scope.
    pub fn is_authorized_write_target(&self, target_range: &RangeRef) -> bool {
        self.approved_write_scope().contains_range(target_range)
    }

    /// Performs an authorized range read for the provider context.
    ///
    /// - Checks that the requested range is within bounded limits (MAX_READ_CELLS).
    /// - Strictly checks that `target_range` is contained within `authorized_selection`.
    /// - Checks that `include_selection_context` is true; if false, returns empty cells to protect privacy.
    /// - Returns only the requested subrange cells, in row-major order, honoring `formulas`.
    pub fn read_authorized_range(&self, target_range: &RangeRef, formulas: bool) -> Result<Vec<Vec<Value>>, String> {
        if target_range.is_full_cols() || target_range.is_full_rows() || target_range.count() > MAX_READ_CELLS {
            return Err(format!(
                "Read range '{}' exceeds maximum allowed read limit ({} cells)",
                target_range.a1(),
                MAX_READ_CELLS
            ));
        }

        if !self.authorized_selection.contains_range(target_range) {
            return Err(format!(
                "Read access denied: range '{}' extends outside permitted user selection '{}'",
                target_range.a1(),
                self.authorized_selection.a1()
            ));
        }

        if !self.include_selection_context {
            // User did not authorize sharing cell data with provider
            let empty_row = vec![Value::Null; target_range.width() as usize];
            return Ok(vec![empty_row; target_range.height() as usize]);
        }

        let mut rows = Vec::with_capacity(target_range.height() as usize);
        for r in target_range.start.row..=target_range.end.row {
            let mut cols = Vec::with_capacity(target_range.width() as usize);
            for c in target_range.start.col..=target_range.end.col {
                cols.push(self.read_cell(r, c, formulas));
            }
            rows.push(cols);
        }
        Ok(rows)
    }

    /// Reads cells for proposal preview diffs.
    ///
    /// Unlike provider reads, this is allowed to read approved write targets (including the totals row)
    /// to show existing before/after content to the user in the UI diff card.
    pub fn read_preview_range(&self, target_range: &RangeRef) -> Result<Vec<Vec<Value>>, String> {
        if target_range.is_full_cols() || target_range.is_full_rows() || target_range.count() > MAX_PROPOSAL_CELLS {
            return Err(format!(
                "Preview range '{}' exceeds maximum proposal limit ({} cells)",
                target_range.a1(),
                MAX_PROPOSAL_CELLS
            ));
        }

        if !self.is_authorized_write_target(target_range) {
            return Err(format!(
                "Preview read denied: range '{}' is outside approved write scope '{}'",
                target_range.a1(),
                self.approved_write_scope().a1()
            ));
        }

        let mut rows = Vec::with_capacity(target_range.height() as usize);
        for r in target_range.start.row..=target_range.end.row {
            let mut cols = Vec::with_capacity(target_range.width() as usize);
            for c in target_range.start.col..=target_range.end.col {
                cols.push(self.read_cell(r, c, true));
            }
            rows.push(cols);
        }
        Ok(rows)
    }
}

