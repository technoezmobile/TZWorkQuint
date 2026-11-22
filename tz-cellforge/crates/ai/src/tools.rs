//! Structured tool schemas and definitions for spreadsheet inspection, reading, and proposed editing.

use serde_json::json;
use crate::types::ToolDefinition;

/// Returns the standard set of structured tools exposed to the LLM.
pub fn get_spreadsheet_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "inspect_context".to_string(),
            description: "Inspect the active workbook title, sheet name, current selected range, and document revision. Call this first to understand what the user has currently highlighted.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "read_range".to_string(),
            description: "Read cell values and formulas from an explicitly bounded range (e.g. 'A1:C10'). Always keep the read range scoped to relevant data.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "range": {
                        "type": "string",
                        "description": "A1-style cell or range reference (e.g. 'A1:D10', 'B2', 'Sheet1!A1:C5')"
                    },
                    "sheet": {
                        "type": "string",
                        "description": "Optional worksheet name (defaults to active sheet)"
                    },
                    "formulas": {
                        "type": "boolean",
                        "description": "If true, returns raw formula text (e.g. '=SUM(A1:A5)') for formula cells"
                    }
                },
                "required": ["range"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "propose_edit".to_string(),
            description: "Propose writing values or formulas to a bounded spreadsheet range. The user will be presented with a before/after diff preview in the UI and must review and approve before any cells are modified. All edits apply as a single undoable transaction.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "range": {
                        "type": "string",
                        "description": "Target A1-style range to write into (e.g. 'B6:D6', 'E2:E10')"
                    },
                    "sheet": {
                        "type": "string",
                        "description": "Target worksheet name"
                    },
                    "values": {
                        "type": "array",
                        "description": "2D array of rows containing values or formulas starting with '=' (e.g. [['=SUM(B2:B5)', '=SUM(C2:C5)']])",
                        "items": {
                            "type": "array",
                            "items": {}
                        }
                    },
                    "explanation": {
                        "type": "string",
                        "description": "Clear explanation of what calculations or changes are being proposed"
                    }
                },
                "required": ["range", "values", "explanation"],
                "additionalProperties": false
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_valid_schema() {
        let tools = get_spreadsheet_tools();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "inspect_context");
        assert_eq!(tools[1].name, "read_range");
        assert_eq!(tools[2].name, "propose_edit");

        for t in tools {
            assert!(t.parameters.is_object());
            assert_eq!(t.parameters["type"], "object");
        }
    }
}
