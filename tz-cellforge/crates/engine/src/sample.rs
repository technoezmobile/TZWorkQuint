//! Sample workbooks built in code (original content), for the start screen, demos, README
//! screenshots and tests.

use std::sync::Arc;

use gridcraft_model::{Sheet, Workbook};
use serde_json::json;

use crate::{DocState, Session};

pub const SAMPLES: &[(&str, &str)] = &[
    ("budget", "Personal Monthly Budget"),
    ("billing", "Billing Statement"),
    ("bp_tracker", "Blood Pressure Tracker"),
    ("expense_report", "Expense Report"),
    ("loan_amortization", "Loan Amortization"),
    ("sales", "Sales Report"),
    ("time_card", "Time Card"),
    ("grades", "Gradebook"),
];

pub fn title(name: &str) -> String {
    let norm = match name {
        "household_budget" | "household" => "budget",
        "quarterly_sales" | "dashboard" => "sales",
        other => other,
    };
    SAMPLES.iter().find(|(n, _)| *n == norm).map(|(_, t)| t.to_string()).unwrap_or_else(|| "Sample".into())
}

/// Builds a sample by replaying commands (so samples also exercise the command layer).
pub fn build(name: &str) -> Option<Workbook> {
    let norm = match name {
        "household_budget" | "household" => "budget",
        "quarterly_sales" | "dashboard" => "sales",
        other => other,
    };
    let mut s = Session::new();
    let mut wb = Workbook::new();
    if let Some(sh) = wb.sheet_mut(0) {
        sh.name = match norm {
            "billing" => "Billing Statement".into(),
            "bp_tracker" => "BP Log".into(),
            "expense_report" => "Expense Report".into(),
            "loan_amortization" => "Amortization".into(),
            "budget" => "Monthly Budget".into(),
            "sales" => "Sales Report".into(),
            "time_card" => "Time Card".into(),
            "grades" => "Gradebook".into(),
            _ => return None,
        };
    }
    s.add_document(DocState::new(wb, None, title(norm), crate::Trust::Local));
    let ok = match norm {
        "billing" => billing(&mut s),
        "bp_tracker" => bp_tracker(&mut s),
        "expense_report" => expense_report(&mut s),
        "loan_amortization" => loan_amortization(&mut s),
        "budget" => budget(&mut s),
        "sales" => sales(&mut s),
        "time_card" => time_card(&mut s),
        "grades" => grades(&mut s),
        _ => false,
    };
    if !ok {
        return None;
    }
    let mut wb = (*s.doc().ok()?.wb).clone();
    wb.set_active_sheet(0);
    if let Some(sh) = wb.sheet_mut(0) {
        sh.view_active = gridcraft_core::CellRef::default();
    }
    Some(wb)
}

fn run(s: &mut Session, id: &str, p: serde_json::Value) -> bool {
    match s.execute(id, p) {
        Ok(_) => true,
        Err(e) => {
            log::warn!("sample: {id} failed: {e}");
            false
        }
    }
}

fn billing(s: &mut Session) -> bool {
    let mut ok = run(s, "range.setValues", json!({"range": "B2", "values": [["Billing Statement"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B2", "name": "Title"}));
    ok &= run(s, "range.setValues", json!({"range": "B3", "values": [["TZLabs Software & Engineering Services · Official Invoice"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B3", "name": "Explanatory Text"}));

    let meta = json!([
        ["Statement #:", "INV-2026-088", "", "Customer ID:", "CUST-409"],
        ["Date:", "2026-11-01", "", "Due Date:", "2026-12-01"],
        ["Billed To:", "Apex Digital Systems Ltd.", "", "Terms:", "Net 30"]
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B5", "values": meta}));
    ok &= run(s, "home.bold", json!({"range": "B5:B7", "on": true}));
    ok &= run(s, "home.bold", json!({"range": "E5:E7", "on": true}));

    let rows = json!([
        ["Item", "Description", "Hours / Qty", "Unit Rate", "Total"],
        ["1", "Native Windows UI Architecture", 32, 125.0],
        ["2", "Spreadsheet Calculation Engine Optimization", 48, 135.0],
        ["3", "Automated Regression Test Harness", 20, 110.0],
        ["4", "Technical Documentation & User Manual", 16, 95.0],
        ["5", "Security Audit & Redaction Verification", 12, 150.0],
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B9", "values": rows}));
    for r in 10..=14 {
        ok &= run(s, "cell.set", json!({"cell": format!("F{r}"), "input": format!("=D{r}*E{r}")}));
    }
    ok &= run(s, "insert.table", json!({"range": "B9:F14", "style": "TableStyleMedium2", "name": "InvoiceLines"}));

    ok &= run(s, "range.setValues", json!({"range": "E15", "values": [["Subtotal:"]]}));
    ok &= run(s, "cell.set", json!({"cell": "F15", "input": "=SUM(F10:F14)"}));
    ok &= run(s, "range.setValues", json!({"range": "E16", "values": [["Tax Rate:"]]}));
    ok &= run(s, "cell.set", json!({"cell": "F16", "input": "0.08"}));
    ok &= run(s, "range.setValues", json!({"range": "E17", "values": [["Tax Amount:"]]}));
    ok &= run(s, "cell.set", json!({"cell": "F17", "input": "=F15*F16"}));
    ok &= run(s, "range.setValues", json!({"range": "E18", "values": [["Total Due:"]]}));
    ok &= run(s, "cell.set", json!({"cell": "F18", "input": "=F15+F17"}));

    ok &= run(s, "home.cellStyle", json!({"range": "E18:F18", "name": "Total"}));
    ok &= run(s, "home.bold", json!({"range": "E15:E18", "on": true}));
    ok &= run(s, "home.numberFormat", json!({"range": "E10:F15", "code": "\"$\"#,##0.00"}));
    ok &= run(s, "home.numberFormat", json!({"range": "F16", "code": "0.0%"}));
    ok &= run(s, "home.numberFormat", json!({"range": "F17:F18", "code": "\"$\"#,##0.00"}));
    ok &= run(s, "home.columnWidth", json!({"cols": "B:B", "width": 60}));
    ok &= run(s, "home.columnWidth", json!({"cols": "C:C", "width": 240}));
    ok &= run(s, "home.columnWidth", json!({"cols": "D:F", "width": 110}));
    ok &= run(s, "selection.set", json!({"cell": "B2"}));
    ok
}

fn bp_tracker(s: &mut Session) -> bool {
    let mut ok = run(s, "range.setValues", json!({"range": "B2", "values": [["Blood Pressure Tracker"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B2", "name": "Title"}));
    ok &= run(s, "range.setValues", json!({"range": "B3", "values": [["Daily systolic, diastolic and pulse monitoring · Thresholds per AHA guidelines"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B3", "name": "Explanatory Text"}));

    let rows = json!([
        ["Date", "Time", "Systolic", "Diastolic", "Pulse", "Category"],
        ["2026-11-01", "08:00", 118, 76, 68],
        ["2026-11-01", "20:00", 124, 78, 72],
        ["2026-11-02", "08:15", 116, 74, 65],
        ["2026-11-02", "20:30", 132, 84, 76],
        ["2026-11-03", "08:00", 121, 78, 70],
        ["2026-11-03", "19:45", 136, 88, 79],
        ["2026-11-04", "08:30", 119, 75, 66],
        ["2026-11-04", "20:15", 128, 82, 73],
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B5", "values": rows}));
    for r in 6..=13 {
        ok &= run(s, "cell.set", json!({"cell": format!("G{r}"), "input": format!("=IFS(D{r}>=140,\"Stage 2\",D{r}>=130,\"Stage 1\",D{r}>=120,\"Elevated\",TRUE,\"Normal\")")}));
    }
    ok &= run(s, "insert.table", json!({"range": "B5:G13", "style": "TableStyleLight9", "name": "BPLog"}));

    let stats = json!([
        ["Metric", "Value"],
        ["Average Systolic", "=AVERAGE(D6:D13)"],
        ["Average Diastolic", "=AVERAGE(E6:E13)"],
        ["Average Pulse", "=AVERAGE(F6:F13)"],
        ["Max Systolic", "=MAX(D6:D13)"],
        ["Min Systolic", "=MIN(D6:D13)"],
    ]);
    ok &= run(s, "range.setValues", json!({"range": "I5", "values": stats}));
    ok &= run(s, "home.cellStyle", json!({"range": "I5:J5", "name": "Heading 3"}));
    ok &= run(s, "home.bold", json!({"range": "I6:I10", "on": true}));
    ok &= run(s, "home.numberFormat", json!({"range": "J6:J10", "code": "0.0"}));
    ok &= run(s, "home.columnWidth", json!({"cols": "B:C", "width": 90}));
    ok &= run(s, "home.columnWidth", json!({"cols": "D:F", "width": 80}));
    ok &= run(s, "home.columnWidth", json!({"cols": "G:G", "width": 100}));
    ok &= run(s, "home.columnWidth", json!({"cols": "I:I", "width": 140}));
    ok &= run(s, "home.columnWidth", json!({"cols": "J:J", "width": 80}));

    ok &= run(s, "selection.set", json!({"range": "C5:E13"}));
    ok &= run(s, "insert.chart", json!({"type": "line", "title": "Blood Pressure Trends", "at": "I12", "width": 460, "height": 260}));
    ok &= run(s, "selection.set", json!({"cell": "B2"}));
    ok
}

fn expense_report(s: &mut Session) -> bool {
    let mut ok = run(s, "range.setValues", json!({"range": "B2", "values": [["Expense Report"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B2", "name": "Title"}));
    ok &= run(s, "range.setValues", json!({"range": "B3", "values": [["Business travel and expense reimbursement log"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B3", "name": "Explanatory Text"}));

    let meta = json!([
        ["Employee:", "Elena Rostova", "", "Department:", "Engineering"],
        ["Period:", "October 2026", "", "Manager:", "Marcus Vance"],
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B5", "values": meta}));
    ok &= run(s, "home.bold", json!({"range": "B5:B6", "on": true}));
    ok &= run(s, "home.bold", json!({"range": "E5:E6", "on": true}));

    let rows = json!([
        ["Date", "Category", "Description", "Amount"],
        ["2026-10-12", "Travel", "Flight to Architecture Summit", 540.20],
        ["2026-10-12", "Lodging", "Hotel Downtown (3 nights)", 675.00],
        ["2026-10-13", "Meals", "Working Dinner with Partners", 148.50],
        ["2026-10-14", "Transport", "Airport Shuttle & Rideshare", 62.30],
        ["2026-10-15", "Supplies", "Workshop Presentation Materials", 45.00],
        ["2026-10-16", "Meals", "Client Lunch Meeting", 86.40],
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B8", "values": rows}));
    ok &= run(s, "insert.table", json!({"range": "B8:E14", "style": "TableStyleMedium6", "name": "Expenses"}));
    ok &= run(s, "home.numberFormat", json!({"range": "E9:E14", "code": "\"$\"#,##0.00"}));

    ok &= run(s, "range.setValues", json!({"range": "D15", "values": [["Total Claim:"]]}));
    ok &= run(s, "cell.set", json!({"cell": "E15", "input": "=SUM(E9:E14)"}));
    ok &= run(s, "range.setValues", json!({"range": "D16", "values": [["Advance Received:"]]}));
    ok &= run(s, "cell.set", json!({"cell": "E16", "input": "500.00"}));
    ok &= run(s, "range.setValues", json!({"range": "D17", "values": [["Net Reimbursement:"]]}));
    ok &= run(s, "cell.set", json!({"cell": "E17", "input": "=E15-E16"}));

    ok &= run(s, "home.cellStyle", json!({"range": "D17:E17", "name": "Total"}));
    ok &= run(s, "home.bold", json!({"range": "D15:D17", "on": true}));
    ok &= run(s, "home.numberFormat", json!({"range": "E15:E17", "code": "\"$\"#,##0.00"}));

    ok &= run(s, "home.columnWidth", json!({"cols": "B:B", "width": 95}));
    ok &= run(s, "home.columnWidth", json!({"cols": "C:C", "width": 110}));
    ok &= run(s, "home.columnWidth", json!({"cols": "D:D", "width": 240}));
    ok &= run(s, "home.columnWidth", json!({"cols": "E:E", "width": 120}));

    ok &= run(s, "selection.set", json!({"range": "C8:E14"}));
    ok &= run(s, "insert.chart", json!({"type": "column", "title": "Expenses by Category", "at": "G8", "width": 460, "height": 270}));
    ok &= run(s, "selection.set", json!({"cell": "B2"}));
    ok
}

fn loan_amortization(s: &mut Session) -> bool {
    let mut ok = run(s, "range.setValues", json!({"range": "B2", "values": [["Loan Amortization Schedule"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B2", "name": "Title"}));
    ok &= run(s, "range.setValues", json!({"range": "B3", "values": [["Enter loan parameters below · Payment and interest schedule update automatically"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B3", "name": "Explanatory Text"}));

    let params = json!([
        ["Loan Parameters", ""],
        ["Loan Amount", 200000.0],
        ["Annual Interest Rate", 0.06],
        ["Loan Period (Years)", 5],
        ["Payments Per Year", 12],
        ["Total Payments", "=C8*C9"],
        ["Monthly Payment", "=PMT(C7/C9,C10,-C6)"]
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B5", "values": params}));
    ok &= run(s, "home.cellStyle", json!({"range": "B5:C5", "name": "Heading 3"}));
    ok &= run(s, "home.bold", json!({"range": "B6:B11", "on": true}));
    ok &= run(s, "home.numberFormat", json!({"range": "C6", "code": "\"$\"#,##0.00"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C7", "code": "0.00%"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C8:C10", "code": "#,##0"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C11", "code": "\"$\"#,##0.00"}));
    ok &= run(s, "home.cellStyle", json!({"range": "B11:C11", "name": "Accent1"}));

    let summary = json!([
        ["Loan Summary", ""],
        ["Total Payments Amount", "=C11*C10"],
        ["Total Interest", "=F6-C6"]
    ]);
    ok &= run(s, "range.setValues", json!({"range": "E5", "values": summary}));
    ok &= run(s, "home.cellStyle", json!({"range": "E5:F5", "name": "Heading 3"}));
    ok &= run(s, "home.bold", json!({"range": "E6:E7", "on": true}));
    ok &= run(s, "home.numberFormat", json!({"range": "F6:F7", "code": "\"$\"#,##0.00"}));

    let headers = json!([
        ["Pmt #", "Beginning Balance", "Payment", "Principal", "Interest", "Ending Balance"]
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B13", "values": headers}));
    ok &= run(s, "home.cellStyle", json!({"range": "B13:G13", "name": "Heading 3"}));

    ok &= run(s, "cell.set", json!({"cell": "B14", "input": "1"}));
    ok &= run(s, "cell.set", json!({"cell": "C14", "input": "=C6"}));
    ok &= run(s, "cell.set", json!({"cell": "D14", "input": "=$C$11"}));
    ok &= run(s, "cell.set", json!({"cell": "F14", "input": "=C14*($C$7/$C$9)"}));
    ok &= run(s, "cell.set", json!({"cell": "E14", "input": "=D14-F14"}));
    ok &= run(s, "cell.set", json!({"cell": "G14", "input": "=C14-E14"}));

    for pmt in 2..=12 {
        let r = 13 + pmt;
        let prev = r - 1;
        ok &= run(s, "cell.set", json!({"cell": format!("B{r}"), "input": format!("{pmt}")}));
        ok &= run(s, "cell.set", json!({"cell": format!("C{r}"), "input": format!("=G{prev}")}));
        ok &= run(s, "cell.set", json!({"cell": format!("D{r}"), "input": "=$C$11"}));
        ok &= run(s, "cell.set", json!({"cell": format!("F{r}"), "input": format!("=C{r}*($C$7/$C$9)")}));
        ok &= run(s, "cell.set", json!({"cell": format!("E{r}"), "input": format!("=D{r}-F{r}")}));
        ok &= run(s, "cell.set", json!({"cell": format!("G{r}"), "input": format!("=C{r}-E{r}")}));
    }
    ok &= run(s, "insert.table", json!({"range": "B13:G25", "style": "TableStyleLight1", "name": "Schedule"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C14:G25", "code": "\"$\"#,##0.00"}));
    ok &= run(s, "home.columnWidth", json!({"cols": "B:B", "width": 55}));
    ok &= run(s, "home.columnWidth", json!({"cols": "C:G", "width": 125}));
    ok &= run(s, "selection.set", json!({"cell": "C7"}));
    ok
}

fn time_card(s: &mut Session) -> bool {
    let mut ok = run(s, "range.setValues", json!({"range": "B2", "values": [["Weekly Time Card"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B2", "name": "Title"}));
    ok &= run(s, "range.setValues", json!({"range": "B3", "values": [["Weekly hourly tracking · Regular and overtime calculations"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B3", "name": "Explanatory Text"}));

    let meta = json!([
        ["Employee:", "Devon Reed", "", "Hourly Rate:", 38.50],
        ["Week Ending:", "2026-10-31", "", "Department:", "Design & Prototyping"],
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B5", "values": meta}));
    ok &= run(s, "home.bold", json!({"range": "B5:B6", "on": true}));
    ok &= run(s, "home.bold", json!({"range": "E5:E6", "on": true}));
    ok &= run(s, "home.numberFormat", json!({"range": "F5", "code": "\"$\"#,##0.00"}));

    let rows = json!([
        ["Day", "Time In", "Lunch Out", "Lunch In", "Time Out", "Daily Hours"],
        ["Monday", 0.35417, 0.50000, 0.52083, 0.70833],
        ["Tuesday", 0.35417, 0.50000, 0.52083, 0.70833],
        ["Wednesday", 0.33333, 0.50000, 0.52083, 0.72917],
        ["Thursday", 0.35417, 0.50000, 0.52083, 0.70833],
        ["Friday", 0.35417, 0.50000, 0.52083, 0.75000],
    ]);
    ok &= run(s, "range.setValues", json!({"range": "B8", "values": rows}));
    for r in 9..=13 {
        ok &= run(s, "cell.set", json!({"cell": format!("G{r}"), "input": format!("=((D{r}-C{r})+(F{r}-E{r}))*24")}));
    }
    ok &= run(s, "insert.table", json!({"range": "B8:G13", "style": "TableStyleMedium7", "name": "TimeSheet"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C9:F13", "code": "h:mm AM/PM"}));
    ok &= run(s, "home.numberFormat", json!({"range": "G9:G13", "code": "0.00"}));

    let summary = json!([
        ["Total Regular Hours:", "=MIN(G14, 40)"],
        ["Total Overtime Hours:", "=MAX(0, G14-40)"],
        ["Total Weekly Hours:", "=SUM(G9:G13)"],
        ["Regular Pay:", "=G15*F5"],
        ["Overtime Pay (1.5x):", "=G16*(F5*1.5)"],
        ["Gross Pay Due:", "=G18+G19"]
    ]);
    ok &= run(s, "range.setValues", json!({"range": "F15", "values": summary}));
    ok &= run(s, "home.bold", json!({"range": "F15:F20", "on": true}));
    ok &= run(s, "home.numberFormat", json!({"range": "G15:G17", "code": "0.00"}));
    ok &= run(s, "home.numberFormat", json!({"range": "G18:G20", "code": "\"$\"#,##0.00"}));
    ok &= run(s, "home.cellStyle", json!({"range": "F20:G20", "name": "Total"}));

    ok &= run(s, "home.columnWidth", json!({"cols": "B:B", "width": 100}));
    ok &= run(s, "home.columnWidth", json!({"cols": "C:F", "width": 90}));
    ok &= run(s, "home.columnWidth", json!({"cols": "G:G", "width": 100}));
    ok &= run(s, "home.columnWidth", json!({"cols": "F:F", "width": 170}));
    ok &= run(s, "selection.set", json!({"cell": "B2"}));
    ok
}

fn sales(s: &mut Session) -> bool {
    let rows = json!([
        ["Region", "Q1", "Q2", "Q3", "Q4", "Total", "Growth", "Trend"],
        ["North", 48200, 51340, 56010, 61870],
        ["South", 39150, 37980, 42400, 45120],
        ["East", 55300, 58910, 57230, 63480],
        ["West", 61020, 64400, 70150, 76390],
        ["Central", 28740, 31260, 33980, 36510],
        ["Online", 72410, 81630, 95220, 112840],
    ]);
    let mut ok = run(s, "range.setValues", json!({"range": "B4", "values": rows}));
    ok &= run(s, "range.setValues", json!({"range": "B2", "values": [["Sales Report — Quarterly Performance"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B2", "name": "Title"}));
    ok &= run(s, "range.setValues", json!({"range": "B3", "values": [["Fiscal year 2026 · Regional breakdown and performance trends"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B3", "name": "Explanatory Text"}));
    for r in 5..=10 {
        ok &= run(s, "cell.set", json!({"cell": format!("G{r}"), "input": format!("=SUM(C{r}:F{r})")}));
        ok &= run(s, "cell.set", json!({"cell": format!("H{r}"), "input": format!("=F{r}/C{r}-1")}));
        ok &= run(s, "insert.sparkline", json!({"range": format!("C{r}:F{r}"), "location": format!("I{r}"), "type": "line", "markers": true}));
    }
    ok &= run(s, "insert.table", json!({"range": "B4:I10", "style": "TableStyleMedium2", "name": "Sales"}));
    ok &= run(s, "table.totalRow", json!({"table": "Sales", "on": true}));
    for col in ["Q1", "Q2", "Q3", "Q4", "Total"] {
        ok &= run(s, "table.totalFunction", json!({"table": "Sales", "column": col, "function": "sum"}));
    }
    ok &= run(s, "table.totalFunction", json!({"table": "Sales", "column": "Growth", "function": "average"}));
    ok &= run(s, "table.totalFunction", json!({"table": "Sales", "column": "Trend", "function": "none"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C5:G11", "code": "\"$\"#,##0"}));
    ok &= run(s, "home.numberFormat", json!({"range": "H5:H11", "code": "0.0%"}));
    ok &= run(s, "home.conditionalFormat", json!({"range": "G5:G10", "rule": {"type": "dataBar", "color": "#5B9BD5"}}));
    ok &= run(s, "home.conditionalFormat", json!({"range": "H5:H10", "rule": {"type": "iconSet", "set": "3Arrows"}}));
    ok &= run(s, "home.columnWidth", json!({"cols": "A:A", "width": 20}));
    ok &= run(s, "home.columnWidth", json!({"cols": "B:B", "width": 132}));
    ok &= run(s, "home.columnWidth", json!({"cols": "C:G", "width": 84}));
    ok &= run(s, "home.columnWidth", json!({"cols": "H:I", "width": 76}));
    ok &= run(s, "selection.set", json!({"range": "B4:F10"}));
    ok &= run(s, "insert.chart", json!({"type": "column", "title": "Revenue by Region", "at": "K3", "width": 460, "height": 270}));
    ok &= run(s, "selection.set", json!({"range": "B4:B10"}));
    ok &=
        run(s, "insert.chart", json!({"range": "B4:B10", "type": "pie", "title": "Share of Annual Total", "at": "K19", "width": 460, "height": 260}));
    if let Ok(d) = s.doc_mut() {
        let wb = Arc::make_mut(&mut d.wb);
        if let Some(sh) = wb.sheet_mut(0)
            && let Some(c) = sh.charts.last_mut()
        {
            c.series = vec![gridcraft_model::Series {
                name: Some("Sales!$G$4".into()),
                categories: Some("Sales!$B$5:$B$10".into()),
                values: "Sales!$G$5:$G$10".into(),
                bubble_sizes: None,
                color: None,
                secondary: false,
                kind: None,
                quartiles: Default::default(),
                smooth: false,
                trendlines: Vec::new(),
            }];
        }
    }
    ok &= run(s, "range.setValues", json!({"range": "B14", "values": [["Best region"], ["Average quarter"], ["Online share"], ["Year-over-year"]]}));
    ok &= run(s, "cell.set", json!({"cell": "C14", "input": "=INDEX(Sales[Region],MATCH(MAX(Sales[Total]),Sales[Total],0))"}));
    ok &= run(s, "cell.set", json!({"cell": "C15", "input": "=AVERAGE(C5:F10)"}));
    ok &= run(s, "cell.set", json!({"cell": "C16", "input": "=G10/SUM(G5:G10)"}));
    ok &= run(s, "cell.set", json!({"cell": "C17", "input": "=SUM(F5:F10)/SUM(C5:C10)-1"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C15", "code": "\"$\"#,##0"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C16:C17", "code": "0.0%"}));
    ok &= run(s, "home.bold", json!({"range": "B14:B17", "on": true}));
    ok &= run(s, "home.borders", json!({"range": "B14:C17", "preset": "outside"}));
    ok &= run(s, "home.fillColor", json!({"range": "B14:C17", "color": "#F2F7FC"}));
    ok &= run(s, "review.newNote", json!({"cell": "H4", "text": "Q4 vs Q1 growth for each region."}));
    ok &= run(s, "selection.set", json!({"cell": "B2"}));
    ok
}

fn budget(s: &mut Session) -> bool {
    let rows = json!([
        ["Category", "Budget", "Actual", "Difference", "% Used"],
        ["Rent", 1850, 1850],
        ["Groceries", 620, 684.35],
        ["Utilities", 240, 212.8],
        ["Transport", 180, 203.5],
        ["Internet & phone", 95, 95],
        ["Insurance", 210, 210],
        ["Dining out", 200, 265.9],
        ["Entertainment", 120, 88.25],
        ["Savings", 600, 600],
        ["Art supplies", 150, 172.4],
    ]);
    let mut ok = run(s, "range.setValues", json!({"range": "B2", "values": [["Personal Monthly Budget — October 2026"]]}));
    ok &= run(s, "home.cellStyle", json!({"range": "B2", "name": "Title"}));
    ok &= run(s, "range.setValues", json!({"range": "B4", "values": rows}));
    for r in 5..=14 {
        ok &= run(s, "cell.set", json!({"cell": format!("E{r}"), "input": format!("=C{r}-D{r}")}));
        ok &= run(s, "cell.set", json!({"cell": format!("F{r}"), "input": format!("=D{r}/C{r}")}));
    }
    ok &= run(s, "range.setValues", json!({"range": "B15", "values": [["Total"]]}));
    for c in ["C", "D", "E"] {
        ok &= run(s, "cell.set", json!({"cell": format!("{c}15"), "input": format!("=SUM({c}5:{c}14)")}));
    }
    ok &= run(s, "cell.set", json!({"cell": "F15", "input": "=D15/C15"}));
    ok &= run(s, "home.cellStyle", json!({"range": "B4:F4", "name": "Heading 3"}));
    ok &= run(s, "home.cellStyle", json!({"range": "B15:F15", "name": "Total"}));
    ok &= run(s, "home.numberFormat", json!({"range": "C5:E15", "format": "Accounting"}));
    ok &= run(s, "home.numberFormat", json!({"range": "F5:F15", "code": "0%"}));
    ok &= run(
        s,
        "home.conditionalFormat",
        json!({"range": "E5:E14", "rule": {"type": "cellIs", "operator": "less", "value": 0, "preset": "lightRedFillDarkRedText"}}),
    );
    ok &= run(s, "home.conditionalFormat", json!({"range": "F5:F14", "rule": {"type": "colorScale", "colors": ["#63BE7B", "#FFEB84", "#F8696B"]}}));
    ok &= run(s, "home.columnWidth", json!({"cols": "B:B", "width": 130}));
    ok &= run(s, "home.columnWidth", json!({"cols": "C:E", "width": 90}));
    ok &= run(s, "selection.set", json!({"range": "B4:D14"}));
    ok &= run(s, "insert.chart", json!({"type": "bar", "title": "Budget vs Actual", "at": "H3", "width": 440, "height": 300}));
    ok &= run(s, "view.freezePanes", json!({"cell": "A5"}));
    ok &= run(s, "selection.set", json!({"cell": "B2"}));
    ok
}

fn grades(s: &mut Session) -> bool {
    let rows = json!([
        ["Student", "Essay", "Midterm", "Project", "Final", "Average", "Grade"],
        ["Amara Okafor", 92, 88, 95, 91],
        ["Bruno Silva", 78, 82, 74, 80],
        ["Chen Wei", 85, 91, 89, 94],
        ["Dana Kowalski", 67, 72, 80, 70],
        ["Elif Yılmaz", 98, 95, 97, 99],
        ["Farid Haddad", 74, 69, 71, 77],
        ["Grace Mensah", 88, 84, 90, 86],
        ["Hiro Tanaka", 59, 64, 70, 62],
    ]);
    let mut ok = run(s, "range.setValues", json!({"range": "A1", "values": rows}));
    for r in 2..=9 {
        ok &= run(s, "cell.set", json!({"cell": format!("F{r}"), "input": format!("=AVERAGE(B{r}:E{r})")}));
        ok &= run(
            s,
            "cell.set",
            json!({"cell": format!("G{r}"), "input": format!("=IFS(F{r}>=90,\"A\",F{r}>=80,\"B\",F{r}>=70,\"C\",F{r}>=60,\"D\",TRUE,\"F\")")}),
        );
    }
    ok &= run(s, "home.numberFormat", json!({"range": "F2:F9", "code": "0.0"}));
    ok &= run(s, "insert.table", json!({"range": "A1:G9", "style": "TableStyleLight9", "name": "Grades"}));
    ok &= run(s, "home.columnWidth", json!({"cols": "A:A", "width": 120}));
    ok &= run(
        s,
        "home.conditionalFormat",
        json!({"range": "G2:G9", "rule": {"type": "cellIs", "operator": "equal", "value": "A", "preset": "greenFillDarkGreenText"}}),
    );
    ok &= run(s, "home.conditionalFormat", json!({"range": "F2:F9", "rule": {"type": "dataBar", "color": "#63BE7B"}}));
    ok
}

/// A blank sheet (for callers that need one without a session).
pub fn blank_sheet(name: &str) -> Sheet {
    Sheet::new(name)
}
