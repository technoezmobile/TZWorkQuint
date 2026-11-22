//! Structured edit proposals, cell diffs, revision binding, and transaction packaging.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Status of an edit proposal during user review and execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Awaiting user review (shows Apply / Reject in task pane).
    Pending,
    /// User accepted and changes were committed in one atomic undoable transaction.
    Applied,
    /// User rejected the proposal; workbook was unchanged.
    Rejected,
    /// Document was modified before user could apply; rejected to prevent clobbering edits.
    Stale,
}

impl ProposalStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending Review",
            Self::Applied => "Applied",
            Self::Rejected => "Rejected",
            Self::Stale => "Stale (Document Modified)",
        }
    }
}

/// A diff item for a single cell.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CellDiff {
    pub cell_ref: String,
    pub old_display: String,
    pub new_display: String,
    pub is_formula: bool,
}

/// A structured proposal to modify a bounded range in the spreadsheet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditProposal {
    pub proposal_id: String,
    /// Workbook instance UID to ensure proposals are never applied to a different open document.
    pub workbook_uid: u64,
    /// Document revision number when context was captured.
    pub base_revision: u64,
    /// Target worksheet name.
    pub sheet_name: String,
    /// Target cell or range (e.g. "C6", "A1:D10").
    pub target_range: String,
    /// Human-readable explanation of the proposed changes.
    pub explanation: String,
    /// Individual cell changes for preview rendering.
    pub diffs: Vec<CellDiff>,
    /// 2-D array of values/formulas to pass to `range.setValues`.
    pub values_matrix: Vec<Vec<Value>>,
    /// Current proposal lifecycle state.
    pub status: ProposalStatus,
    /// Timestamp for ordering and expiration.
    pub created_at_ms: u64,
}

impl EditProposal {
    /// Checks whether the proposal is stale compared to the current document revision.
    pub fn is_stale(&self, current_doc_uid: u64, current_revision: u64) -> bool {
        self.workbook_uid != current_doc_uid || self.base_revision != current_revision
    }

    /// Validates that the proposal has well-formed values and does not exceed maximum cell bounds.
    pub fn validate_bounds(&self, max_cells: usize) -> Result<(), String> {
        if self.target_range.trim().is_empty() {
            return Err("Target range cannot be empty".to_string());
        }
        let total_cells: usize = self.values_matrix.iter().map(|r| r.len()).sum();
        if total_cells == 0 {
            return Err("Proposal contains no cell values to write".to_string());
        }
        if total_cells > max_cells {
            return Err(format!(
                "Proposal affects {} cells, exceeding maximum permitted batch limit of {} cells",
                total_cells, max_cells
            ));
        }
        Ok(())
    }

    /// Builds the parameters for the atomic `range.setValues` engine command.
    pub fn to_range_set_values_params(&self) -> Value {
        json!({
            "range": if self.sheet_name.is_empty() {
                self.target_range.clone()
            } else {
                format!("{}!{}", self.sheet_name, self.target_range)
            },
            "values": self.values_matrix,
            "parse": true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stale_detection() {
        let proposal = EditProposal {
            proposal_id: "prop-1".to_string(),
            workbook_uid: 100,
            base_revision: 5,
            sheet_name: "Sheet1".to_string(),
            target_range: "B6".to_string(),
            explanation: "Add total".to_string(),
            diffs: vec![CellDiff {
                cell_ref: "B6".to_string(),
                old_display: "".to_string(),
                new_display: "=SUM(B2:B5)".to_string(),
                is_formula: true,
            }],
            values_matrix: vec![vec![json!("=SUM(B2:B5)")]],
            status: ProposalStatus::Pending,
            created_at_ms: 1000,
        };

        // When document revision matches: NOT stale
        assert!(!proposal.is_stale(100, 5));

        // When revision changed (e.g. user typed in another cell): STALE
        assert!(proposal.is_stale(100, 6));

        // When workbook UID differs: STALE
        assert!(proposal.is_stale(101, 5));
    }

    #[test]
    fn test_bounds_validation() {
        let mut proposal = EditProposal {
            proposal_id: "prop-2".to_string(),
            workbook_uid: 100,
            base_revision: 5,
            sheet_name: "Sheet1".to_string(),
            target_range: "A1:B2".to_string(),
            explanation: "Test".to_string(),
            diffs: vec![],
            values_matrix: vec![vec![json!(1), json!(2)], vec![json!(3), json!(4)]],
            status: ProposalStatus::Pending,
            created_at_ms: 1000,
        };

        // 4 cells <= 10 limit: OK
        assert!(proposal.validate_bounds(10).is_ok());

        // 4 cells > 3 limit: ERR
        assert!(proposal.validate_bounds(3).is_err());

        // Empty values: ERR
        proposal.values_matrix = vec![];
        assert!(proposal.validate_bounds(10).is_err());
    }

    #[test]
    fn test_command_params() {
        let proposal = EditProposal {
            proposal_id: "prop-3".to_string(),
            workbook_uid: 100,
            base_revision: 1,
            sheet_name: "Data".to_string(),
            target_range: "C10".to_string(),
            explanation: "Average".to_string(),
            diffs: vec![],
            values_matrix: vec![vec![json!("=AVERAGE(C1:C9)")]],
            status: ProposalStatus::Pending,
            created_at_ms: 1000,
        };

        let params = proposal.to_range_set_values_params();
        assert_eq!(params["range"], "Data!C10");
        assert_eq!(params["values"][0][0], "=AVERAGE(C1:C9)");
        assert_eq!(params["parse"], true);
    }
}
