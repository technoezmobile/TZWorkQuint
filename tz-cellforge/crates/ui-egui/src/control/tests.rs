use super::*;

fn sample() -> Vec<Json> {
    vec![
        json!({"id": "home.borders", "label": "Borders", "menu": ["Home", "Font"]}),
        json!({"id": "home.bold", "label": "Bold", "menu": ["Home", "Font"]}),
        json!({"id": "ui.ribbonTab", "label": "Ribbon Tab", "enabled": true, "ui": true}),
    ]
}

fn ids(v: &[Json]) -> Vec<&str> {
    v.iter().filter_map(|c| c.get("id").and_then(Json::as_str)).collect()
}

#[test]
fn search_filters_case_insensitively() {
    assert_eq!(ids(&filter_commands(sample(), Some("border"))), ["home.borders"]);
    assert_eq!(filter_commands(sample(), Some("BORDER")), filter_commands(sample(), Some("border")));
    assert_eq!(ids(&filter_commands(sample(), Some("ribbon"))), ["ui.ribbonTab"]);
    assert_eq!(ids(&filter_commands(sample(), Some("font"))), ["home.borders", "home.bold"]);
    assert!(filter_commands(sample(), Some("catalog_no_match_7391")).is_empty());
    assert_eq!(filter_commands(sample(), None).len(), 3);
}

fn test_app() -> SheetApp {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    SheetApp::new(session, crate::Services::default())
}

#[test]
/// Checks that oversized click/drag counts are refused without changing already queued batches, that queued input counts toward the budget, and that normal gestures still queue fully.
fn oversized_pointer_requests_are_refused_and_leave_the_queue_alone() {
    let mut app = test_app();
    let p = egui::pos2(20.0, 20.0);
    let m = egui::Modifiers::NONE;
    let btn = egui::PointerButton::Primary;
    assert!(push_click(&mut app, p, btn, 2, m).is_ok());
    assert_eq!(app.synthetic.iter().map(Vec::len).sum::<usize>(), 5);
    assert!(push_click(&mut app, p, btn, u32::MAX, m).is_err());
    assert!(push_drag(&mut app, p, p, usize::MAX, m).is_err());
    assert!(push_drag(&mut app, p, p, MAX_QUEUED_SYNTHETIC_EVENTS, m).is_err());
    assert_eq!(app.synthetic.len(), 2);
    assert_eq!(app.synthetic.iter().map(Vec::len).sum::<usize>(), 5);
    // Already queued input counts toward the budget.
    assert!(push_click(&mut app, p, btn, (MAX_QUEUED_SYNTHETIC_EVENTS / 2) as u32, m).is_err());
    assert_eq!(app.synthetic.len(), 2);
    // Normal requests still queue in full.
    assert!(push_drag(&mut app, p, p + egui::vec2(40.0, 40.0), 3, m).is_ok());
    assert_eq!(app.synthetic.len(), 8);
}

#[test]
fn print_command_discovery_documents_settings_and_submission() {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, crate::Services::default());
    let Outcome::Done(reply) = handle(&mut app, &egui::Context::default(), "engine.commands", &json!({"search":"print"})) else {
        panic!("command discovery must return immediately");
    };
    assert_eq!(reply["ok"], true);
    let commands = reply["result"].as_array().unwrap();
    for (id, expected_docs) in [
        ("ui.printSettings", &["copies", "duplex", "refresh", "discovering", "status", "detail"][..]),
        ("ui.printSubmit", &["file.print", "pagination", "pending", "without opening a dialog"][..]),
    ] {
        let command = commands.iter().find(|command| command["id"] == id).unwrap();
        assert_eq!(command["ui"], true);
        let docs = command["params"].as_str().unwrap();
        for expected in expected_docs {
            assert!(docs.contains(expected), "{id}: missing {expected}");
        }
    }
    assert!(!app.printing.busy(), "discovery must not submit a job");
    assert!(app.backstage.is_none(), "discovery must not open settings");
}

#[test]
fn language_command_docs_list_every_language() {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, crate::Services::default());
    let Outcome::Done(reply) = handle(&mut app, &egui::Context::default(), "engine.commands", &json!({"search":"language"})) else {
        panic!("command discovery must return immediately");
    };
    let commands = reply["result"].as_array().unwrap();
    let docs = commands.iter().find(|c| c["id"] == "app.language.set").unwrap()["params"].as_str().unwrap();
    for l in crate::i18n::Language::ALL {
        assert!(docs.contains(&format!("\"{}\"", l.code())), "app.language.set docs miss {:?}: {docs}", l.code());
    }
}
/// One headless UI pass. A test has no renderer to apply the font-atlas delta, so dropping
/// it is intentional (0.37 panics when a `TexturesDelta` is dropped unapplied).
fn ui_pass(ctx: &egui::Context, raw: egui::RawInput, f: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
    let mut out = ctx.run_ui(raw, f);
    out.textures_delta.clear();
    out
}

/// Returns an egui context initialized by one blank pass at `size` with an 8192-pixel texture limit.
fn sized_context(size: egui::Vec2) -> egui::Context {
    let ctx = egui::Context::default();
    let raw =
        egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)), max_texture_side: Some(8192), ..Default::default() };
    let _ = ui_pass(&ctx, raw, |_| {});
    ctx
}

/// Returns blank raw input with a screen rect at `size` and an 8192-pixel texture limit.
fn raw_of(size: egui::Vec2) -> egui::RawInput {
    egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)), max_texture_side: Some(8192), ..Default::default() }
}

/// Extracts cloned `ViewportCommand::Screenshot` callbacks from a `FullOutput` for the context's viewport.
fn screenshot_callbacks(out: &egui::FullOutput, ctx: &egui::Context) -> Vec<egui::ScreenshotCallback> {
    out.viewport_output
        .get(&ctx.viewport_id())
        .map(|output| {
            output
                .commands
                .iter()
                .filter_map(|command| match command {
                    egui::ViewportCommand::Screenshot(callback) => Some(callback.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// Run three settled passes with `raw` and return the callbacks issued in them.
fn issue_with(ctx: &egui::Context, app: &mut SheetApp, raw: egui::RawInput) -> Vec<egui::ScreenshotCallback> {
    let mut callbacks = Vec::new();
    for _ in 0..3 {
        let out = ui_pass(ctx, raw.clone(), |ui| issue_screenshots(app, ui.ctx()));
        callbacks.extend(screenshot_callbacks(&out, ctx));
    }
    callbacks
}

/// Returns a per-process temporary PNG path after removing any stale file at that path.
fn temp_png(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("gridcraft-shot-{name}-{}.png", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
/// Checks rounded physical pixels, expected/retained byte counts, the texture-axis cap, the no-size case and overflow refusal.
fn capture_geometry_reports_checked_capped_pixels() {
    let geometry = CaptureGeometry {
        viewport_id: egui::ViewportId::ROOT,
        viewport: Some(egui::vec2(800.0, 600.0)),
        pixels_per_point: 2.0,
        zoom: 1.0,
        max_texture_side: 4096,
        minimized: None,
        occluded: None,
        visible: None,
    };
    assert_eq!(geometry.expected_size(), Some([1600, 1200]));
    assert_eq!(geometry.expected_bytes(), Some(1600 * 1200 * 4));
    assert_eq!(geometry.retained_bytes(), Some(1601 * 1201 * 4), "one pixel of rounding per axis is reserved too");
    let capped = CaptureGeometry { max_texture_side: 1024, ..geometry };
    assert_eq!(capped.expected_size(), Some([1024, 1024]), "the surface limit caps both axes");
    let empty = CaptureGeometry { viewport: Some(egui::Vec2::ZERO), ..geometry };
    assert_eq!(empty.expected_size(), None, "a minimized window has no capturable size");
    assert_eq!(empty.retained_bytes(), None);
    let huge = CaptureGeometry { viewport: Some(egui::vec2(f32::MAX, f32::MAX)), ..geometry };
    assert_eq!(huge.expected_size(), None, "overflow is refused, not truncated");
}

#[test]
/// Checks that one request beyond `SCREENSHOT_MAX_REQUESTS` is rejected with its token while earlier requests stay queued and do not reply early.
fn screenshot_admission_stops_on_the_count_budget_without_evicting() {
    let ctx = sized_context(egui::vec2(800.0, 600.0));
    let mut app = test_app();
    let mut replies = Vec::new();
    for _ in 0..SCREENSHOT_MAX_REQUESTS {
        let (tx, rx) = std::sync::mpsc::channel();
        admit(&mut app, &ctx, None, tx);
        replies.push(rx);
    }
    assert_eq!(app.shots.requests.len(), SCREENSHOT_MAX_REQUESTS);
    assert!(replies.iter_mut().all(|r| r.try_recv().is_err()), "admitted requests do not reply early");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    let reply = rx.try_recv().expect("the over-count request is rejected");
    assert_eq!(reply["ok"], false);
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("limit 4")), "{reply}");
    assert!(reply["request"].as_u64().is_some(), "a rejection carries its request token: {reply}");
    assert_eq!(app.shots.requests.len(), SCREENSHOT_MAX_REQUESTS, "an earlier request is never evicted");
}

#[test]
/// Three roughly 100 MB path captures hit the aggregate byte limit; the third is rejected and only two requests remain.
fn screenshot_admission_stops_on_aggregate_bytes_before_the_count() {
    let ctx = sized_context(egui::vec2(5000.0, 5000.0));
    let mut app = test_app();
    let path = temp_png("aggregate").to_string_lossy().into_owned();
    let mut rejected = None;
    for _ in 0..3 {
        let (tx, rx) = std::sync::mpsc::channel();
        // A path capture (not base64) isolates the retained-image aggregate budget.
        admit(&mut app, &ctx, Some(path.clone()), tx);
        if let Ok(reply) = rx.try_recv() {
            rejected = Some(reply);
        }
    }
    let reply = rejected.expect("three 100 MB captures exceed the 256 MiB aggregate budget");
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("aggregate limit")), "{reply}");
    assert_eq!(app.shots.requests.len(), 2);
}

#[test]
/// An unknown-size request reserves the placeholder, issues nothing while settling, then replies once on timeout and releases the reservation.
fn requests_without_a_size_stay_queued_and_time_out() {
    let ctx = sized_context(egui::Vec2::ZERO);
    let mut app = test_app();
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    assert!(rx.try_recv().is_err(), "an unknown size is queued, not refused with an overflow error");
    assert_eq!(app.shots.requests[0].bytes, UNKNOWN_CAPTURE_BYTES, "the largest possible image is reserved meanwhile");
    // Settling passes without a size neither issue nor fail the request.
    let callbacks = issue_with(&ctx, &mut app, raw_of(egui::Vec2::ZERO));
    assert!(callbacks.is_empty());
    assert!(!app.shots.requests[0].issued);
    assert!(rx.try_recv().is_err());
    app.shots.requests[0].deadline = 0.0;
    let _ = ui_pass(&ctx, raw_of(egui::Vec2::ZERO), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the size-less request times out once");
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("capturable size")), "{reply}");
    assert_eq!(app.shots.reserved(), 0, "the placeholder reservation is released");
}

/// Counts `RequestPaintWhileHidden` commands in a logic output.
fn hidden_paints(out: &egui::LogicOutput) -> usize {
    out.viewport_commands.values().flatten().filter(|c| matches!(c, egui::ViewportCommand::RequestPaintWhileHidden)).count()
}

#[test]
/// A queued capture requests a hidden paint; once issued, the screenshot command supplies the paint itself.
fn a_queued_capture_requests_hidden_paint_until_it_is_issued() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let (tx, _rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    let queued = ctx.run_logic(&raw_of(egui::vec2(64.0, 48.0)), |ctx| app.logic(ctx));
    assert_eq!(hidden_paints(&queued), 1, "a queued capture needs a real `ui` pass");

    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);
    let issued = ctx.run_logic(&raw_of(egui::vec2(64.0, 48.0)), |ctx| app.logic(ctx));
    assert_eq!(hidden_paints(&issued), 0, "the issued Screenshot command paints the hidden window by itself");
}

#[test]
/// A minimized request stays queued without requesting hidden paints, because it cannot issue until restored.
fn a_minimized_capture_does_not_force_hidden_frames_while_it_waits() {
    let ctx = egui::Context::default();
    let mut app = test_app();
    let mut raw = raw_of(egui::vec2(800.0, 600.0));
    if let Some(v) = raw.viewports.get_mut(&egui::ViewportId::ROOT) {
        v.inner_rect = None;
        v.minimized = Some(true);
    }
    app.raw_input_hook(&ctx, &mut raw);
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    assert!(rx.try_recv().is_err());
    let waited = ctx.run_logic(&raw, |ctx| app.logic(ctx));
    assert_eq!(hidden_paints(&waited), 0, "a minimized request cannot issue until it is restored");
    app.shots.cancel_all();
}

#[test]
/// The real app harness issues exactly one callback, replies once with the token and dimensions, and releases the reservation.
fn one_admitted_capture_issues_one_callback_and_replies_once() {
    use egui_kittest::kittest::Queryable as _;

    let mut app = test_app();
    app.use_prepared_fonts();
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(900.0, 700.0))
        .with_context_setup(|ctx| {
            ctx.set_fonts(crate::theme::pinned_font_definitions());
            crate::theme::apply(ctx, false);
        })
        // This test completes the callback itself. Keep the real app/AccessKit tree, but
        // do not let kittest's automatic screenshot handling initialize a GPU renderer.
        .renderer(egui_kittest::LazyRenderer::Uninitialized { textures_delta: Default::default(), builder: None })
        .build_ui_state(
            |ui, app: &mut SheetApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
    h.run_steps(4);
    assert!(h.query_by_label("Name Box").is_some(), "the capture fixture draws the real app, not an empty tree");
    h.check_accessibility();

    let ctx = h.ctx.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    admit(h.state_mut(), &ctx, None, tx);
    let token = h.state().shots.requests[0].token;
    h.state_mut().shots.requests[0].ready_at = 0.0;
    let mut callbacks = Vec::new();
    for _ in 0..3 {
        h.step();
        callbacks.extend(screenshot_callbacks(h.output(), &ctx));
    }
    assert_eq!(callbacks.len(), 1, "the settled request is issued once");
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([900, 700], egui::Color32::RED)));
    h.step(); // the real logic drains the callback and the UI repaints
    h.check_accessibility();
    assert!(h.query_by_label("Name Box").is_some(), "the app tree survives callback drain and repaint");
    let reply = rx.try_recv().expect("the drain replies once");
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["width"], 900);
    assert_eq!(reply["result"]["height"], 700);
    assert_eq!(reply["request"].as_u64(), Some(token), "the reply names its request");
    assert!(rx.try_recv().is_err(), "a completed slot is taken exactly once");
    assert_eq!(h.state().shots.reserved(), 0, "the reservation is released after the reply");
}

#[test]
/// Completing callbacks in reverse order replies on each matching token/path and writes each request's own pixels.
fn out_of_order_completions_reply_on_their_own_request() {
    let ctx = sized_context(egui::vec2(8.0, 8.0));
    let mut app = test_app();
    let p1 = temp_png("order-1");
    let p2 = temp_png("order-2");
    let (tx1, rx1) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(p1.to_string_lossy().into_owned()), tx1);
    let token1 = app.shots.requests[0].token;
    let (tx2, rx2) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(p2.to_string_lossy().into_owned()), tx2);
    let token2 = app.shots.requests[1].token;
    for request in &mut app.shots.requests {
        request.ready_at = 0.0;
    }
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 2, "one callback per admitted request");
    // Complete the second request first: slots, not callback order, carry the result.
    callbacks[1].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([8, 8], egui::Color32::BLUE)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let second = rx2.try_recv().expect("the second request gets its own reply");
    assert_eq!(second["ok"], true, "{second}");
    assert_eq!(second["request"].as_u64(), Some(token2));
    assert_eq!(second["result"]["path"].as_str(), Some(p2.to_string_lossy().as_ref()));
    assert!(!p1.exists(), "the unfinished request has not written anything");
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([8, 8], egui::Color32::RED)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let first = rx1.try_recv().expect("the first request gets its own reply");
    assert_eq!(first["request"].as_u64(), Some(token1));
    assert_eq!(first["result"]["path"].as_str(), Some(p1.to_string_lossy().as_ref()));
    assert_ne!(std::fs::read(&p1).unwrap(), std::fs::read(&p2).unwrap(), "each reply carries its own pixels");
    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// An expired request replies once with a timeout, writes no file and releases its reservation.
fn a_timed_out_capture_replies_once_and_never_writes() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let path = temp_png("timeout");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
    app.shots.requests[0].deadline = 0.0;
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the timeout replies");
    assert_eq!(reply["ok"], false);
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("timeout")), "{reply}");
    assert!(!path.exists(), "a timed-out capture never writes a file");
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    assert!(rx.try_recv().is_err(), "a timeout is reported once");
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// A valid image that completes after its deadline gets one timeout reply and is not written.
fn a_capture_completed_after_its_deadline_writes_nothing() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let path = temp_png("late");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
    let token = app.shots.requests[0].token;
    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);
    // The deadline passes while the callback is outstanding; the image arrives afterwards.
    app.shots.requests[0].deadline = 0.0;
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([64, 48], egui::Color32::RED)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the late completion gets one timeout reply");
    assert_eq!(reply["ok"], false, "{reply}");
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("timeout")), "{reply}");
    assert_eq!(reply["request"].as_u64(), Some(token));
    assert!(!path.exists(), "a capture that completed after its deadline never writes a file");
    assert!(rx.try_recv().is_err(), "the late image is reported once");
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// Cancelling a request does not reply; a later callback for it is ignored and writes nothing.
fn a_callback_after_the_request_is_removed_is_inert() {
    let ctx = sized_context(egui::vec2(32.0, 24.0));
    let mut app = test_app();
    let path = temp_png("removed");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);
    app.shots.cancel_all();
    assert!(rx.try_recv().is_err(), "cancelling a request does not reply");
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([32, 24], egui::Color32::RED)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    assert!(rx.try_recv().is_err(), "a callback for a removed request is inert");
    assert!(!path.exists());
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// The first issuance adopts a new geometry, updates the epoch and reservation, spends no retry, and the capture then succeeds.
fn a_resize_between_admission_and_issuance_does_not_cost_a_retry() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    app.shots.requests[0].ready_at = 0.0;
    let admitted_epoch = app.shots.requests[0].epoch;
    let resized = raw_of(egui::vec2(32.0, 24.0));
    let callbacks = issue_with(&ctx, &mut app, resized.clone());
    assert_eq!(callbacks.len(), 1);
    let request = &app.shots.requests[0];
    assert_eq!(request.retries, 0, "the first issuance adopts the current geometry instead of spending a retry");
    assert_eq!(request.geometry.viewport, Some(egui::vec2(32.0, 24.0)));
    assert_eq!(request.expected, Some([32, 24]));
    assert_eq!(request.bytes, 33 * 25 * 4);
    assert!(request.epoch > admitted_epoch, "issuance records the new epoch together with the geometry");
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([32, 24], egui::Color32::RED)));
    let _ = ui_pass(&ctx, resized, |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the once-issued capture replies");
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// The reservation is the largest accepted rounded image (11×9×4 bytes for a 10×8 viewport), and that image is accepted.
fn the_retained_reservation_is_the_slot_bound() {
    let ctx = sized_context(egui::vec2(10.0, 8.0));
    let mut app = test_app();
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    // validate_image accepts one pixel of rounding per axis: an 11x9 image is the largest.
    assert_eq!(app.shots.requests[0].bytes, 11 * 9 * 4);
    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([11, 9], egui::Color32::RED)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the rounded image is accepted");
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["width"], 11);
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// Oversized images retry within the bounded budget, never write, then report the per-image limit, while the valid peer is written.
fn an_image_beyond_the_reservation_is_released_without_writing() {
    let ctx = sized_context(egui::vec2(10.0, 8.0));
    let mut app = test_app();
    let p1 = temp_png("over-1");
    let p2 = temp_png("over-2");
    let (tx1, rx1) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(p1.to_string_lossy().into_owned()), tx1);
    let (tx2, rx2) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(p2.to_string_lossy().into_owned()), tx2);
    for request in &mut app.shots.requests {
        request.ready_at = 0.0;
    }
    assert_eq!(app.shots.reserved(), 2 * 11 * 9 * 4);
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 2);
    // The slot refuses an image over the reservation instead of retaining bytes beyond it.
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([12, 9], egui::Color32::RED)));
    // The other request completes with valid pixels.
    callbacks[1].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([10, 8], egui::Color32::BLUE)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    // A larger image can be a genuine resize, so it receives the same bounded retries as
    // a smaller one. Each oversized callback is released and never written while retrying.
    for attempt in 1..=SCREENSHOT_MAX_RETRIES {
        assert_eq!(app.shots.requests[0].retries, attempt);
        assert!(rx1.try_recv().is_err(), "retry {attempt} has not replied early");
        assert!(!p1.exists(), "oversized pixels are never written on retry {attempt}");
        let callbacks = issue_with(&ctx, &mut app, raw_of(egui::vec2(10.0, 8.0)));
        assert_eq!(callbacks.len(), 1, "the other valid request is already complete");
        callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([12, 9], egui::Color32::RED)));
        let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    }
    let refused = rx1.try_recv().expect("the oversized image is reported after bounded retries");
    assert_eq!(refused["ok"], false, "{refused}");
    assert!(refused["error"].as_str().is_some_and(|e| e.contains("per-image limit")), "{refused}");
    assert!(!p1.exists());
    let accepted = rx2.try_recv().expect("the valid image is written");
    assert_eq!(accepted["ok"], true, "{accepted}");
    let _ = std::fs::remove_file(&p2);
    assert_eq!(app.shots.reserved(), 0, "both reservations are released");
}

#[test]
/// Two `ui` calls in one pass count as one settling frame, and a pass that requests a discard is not counted; only its rerun counts.
fn settling_counts_one_real_pass_and_skips_a_discarded_one() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let (tx, _rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    // Two `ui` calls in one pass are one settling frame.
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| {
        issue_screenshots(&mut app, ui.ctx());
        issue_screenshots(&mut app, ui.ctx());
    });
    assert_eq!(app.shots.requests[0].frames, 1);
    // A pass that already asked to be discarded does not count, even though `run_ui` repeats it.
    let mut asked = false;
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| {
        if !asked {
            asked = true;
            ui.ctx().request_discard("settling test");
        }
        issue_screenshots(&mut app, ui.ctx());
    });
    assert_eq!(app.shots.requests[0].frames, 2, "only the rerun (real) pass counted");
}

#[test]
/// Issuance survives a discard requested after it, the image is written once, and a duplicate callback completion is inert.
fn a_discard_requested_after_issuance_still_issues_and_writes_once() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let path = temp_png("discard-after-issue");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
    app.shots.requests[0].ready_at = 0.0;
    let mut asked = false;
    let mut callbacks = Vec::new();
    for _ in 0..3 {
        let out = ui_pass(&ctx, egui::RawInput::default(), |ui| {
            issue_screenshots(&mut app, ui.ctx());
            if !asked && app.shots.requests.first().is_some_and(|r| r.issued) {
                asked = true;
                ui.ctx().request_discard("discard after screenshot issuance");
            }
        });
        callbacks.extend(screenshot_callbacks(&out, &ctx));
    }
    assert!(asked, "the test requested a discard after issuance");
    assert_eq!(callbacks.len(), 1, "issuance happens exactly once across the discard repeat");
    let image = std::sync::Arc::new(egui::ColorImage::filled([64, 48], egui::Color32::RED));
    callbacks[0].clone().complete(image.clone());
    callbacks[0].clone().complete(image);
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the discarded pass still delivers one reply");
    assert_eq!(reply["ok"], true, "{reply}");
    assert!(path.exists(), "the image is written exactly once");
    assert!(rx.try_recv().is_err(), "a second completion of the same callback is inert");
    let _ = std::fs::remove_file(&path);
}

#[test]
/// A grow-resize spends one retry, reissues at the new size and then accepts the new image.
fn an_image_larger_than_the_reservation_retries_like_a_smaller_one() {
    let ctx = sized_context(egui::vec2(10.0, 8.0));
    let mut app = test_app();
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);

    // The surface grew by two pixels while the image was outstanding. The slot's reservation
    // refuses the larger image before `validate_image`; drain must retry at the new geometry
    // instead of failing because the resize went in this direction.
    let mut grown = raw_of(egui::vec2(20.0, 16.0));
    app.raw_input_hook(&ctx, &mut grown);
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([12, 9], egui::Color32::RED)));
    let _ = ctx.run_logic(&grown, |ctx| app.logic(ctx));
    let request = &app.shots.requests[0];
    assert_eq!(request.retries, 1, "the slot's per-image refusal spends one retry, not a failure");
    assert!(!request.issued);
    assert_eq!(request.geometry.viewport, Some(egui::vec2(20.0, 16.0)));
    assert!(rx.try_recv().is_err(), "a retried capture does not reply yet");

    let callbacks = issue_with(&ctx, &mut app, raw_of(egui::vec2(20.0, 16.0)));
    assert_eq!(callbacks.len(), 1);
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([20, 16], egui::Color32::BLUE)));
    let _ = ui_pass(&ctx, raw_of(egui::vec2(20.0, 16.0)), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the retried capture replies");
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// Minimize retries reserve unknown-size bytes; only two fit the aggregate budget and stay queued (the rest are reported), and the survivors write after restore.
fn a_minimize_retry_without_a_size_still_checks_the_aggregate_budget() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let mut paths = Vec::new();
    let mut receivers = Vec::new();
    for n in 0..SCREENSHOT_MAX_REQUESTS {
        let path = temp_png(&format!("minimize-retry-{n}"));
        let (tx, rx) = std::sync::mpsc::channel();
        // A path capture isolates the retained-image aggregate budget.
        admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
        paths.push(path);
        receivers.push(rx);
    }
    for request in &mut app.shots.requests {
        request.ready_at = 0.0;
    }
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), SCREENSHOT_MAX_REQUESTS, "four path captures fit every budget");
    assert_eq!(app.shots.reserved(), SCREENSHOT_MAX_REQUESTS * 65 * 49 * 4);

    // The window is minimized with every capture outstanding. The drain retries each one
    // against the new surface generation, which has no capturable size, so every retry
    // reserves UNKNOWN_CAPTURE_BYTES. Only two unknown-size reservations fit the aggregate
    // limit; the rest must be reported instead of inflating `reserved()` over the budget.
    let mut minimized = raw_of(egui::vec2(64.0, 48.0));
    if let Some(viewport) = minimized.viewports.get_mut(&egui::ViewportId::ROOT) {
        viewport.inner_rect = None;
        viewport.minimized = Some(true);
    }
    app.raw_input_hook(&ctx, &mut minimized);
    let image = std::sync::Arc::new(egui::ColorImage::filled([64, 48], egui::Color32::RED));
    for callback in &callbacks {
        callback.clone().complete(image.clone());
    }
    let _ = ctx.run_logic(&minimized, |ctx| app.logic(ctx));

    assert!(
        app.shots.reserved() <= SCREENSHOT_MAX_AGGREGATE_BYTES,
        "size-less retries reserved {} bytes, over the {SCREENSHOT_MAX_AGGREGATE_BYTES}-byte aggregate limit",
        app.shots.reserved()
    );
    let rejected: Vec<Json> = receivers.iter_mut().filter_map(|rx| rx.try_recv().ok()).collect();
    assert_eq!(rejected.len(), 2, "two unknown-size retries no longer fit the aggregate budget");
    assert!(rejected.iter().all(|reply| reply["error"].as_str().is_some_and(|e| e.contains("aggregate limit"))), "{rejected:?}");
    assert_eq!(app.shots.requests.len(), 2, "the two fitting retries stay queued");
    assert!(app.shots.requests.iter().all(|r| !r.issued && r.expected.is_none() && r.bytes == UNKNOWN_CAPTURE_BYTES));
    assert!(receivers.iter_mut().all(|rx| rx.try_recv().is_err()), "the surviving retries wait for the restore");

    // Restoring re-issues only the surviving requests at the real size; both fit because the
    // over-reserved retries were rejected rather than kept.
    let mut restored = raw_of(egui::vec2(64.0, 48.0));
    if let Some(viewport) = restored.viewports.get_mut(&egui::ViewportId::ROOT) {
        viewport.inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(64.0, 48.0)));
        viewport.minimized = Some(false);
        viewport.occluded = Some(false);
    }
    app.raw_input_hook(&ctx, &mut restored);
    let callbacks = issue_with(&ctx, &mut app, restored.clone());
    assert_eq!(callbacks.len(), 2, "the surviving requests issue on restore");
    for callback in &callbacks {
        callback.clone().complete(image.clone());
    }
    let _ = ctx.run_logic(&restored, |ctx| app.logic(ctx));

    let mut delivered = 0;
    for rx in &mut receivers {
        if let Ok(reply) = rx.try_recv() {
            assert_eq!(reply["ok"], true, "{reply}");
            delivered += 1;
        }
    }
    assert_eq!(delivered, 2, "the two surviving captures reply once each");
    let written = paths.iter().filter(|path| path.exists()).count();
    assert_eq!(written, 2, "only the surviving captures write their image");
    for path in &paths {
        let _ = std::fs::remove_file(path);
    }
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// Fresh hidden viewport geometry invalidates the old image, spends one retry, and the reissued image succeeds.
fn a_hidden_resize_rejects_an_image_issued_before_it_and_reissues() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([64, 48], egui::Color32::RED)));

    // A hidden tick carries the resize in `RawInput::viewports`; the last UI pass still says
    // 64x48, so only the fresh viewport info lets the drain see the change.
    let hidden = || {
        let mut raw = raw_of(egui::vec2(64.0, 48.0));
        if let Some(viewport) = raw.viewports.get_mut(&egui::ViewportId::ROOT) {
            viewport.inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(32.0, 400.0)));
            viewport.native_pixels_per_point = Some(1.0);
        }
        raw
    };
    let _ = ctx.run_logic(&hidden(), |ctx| app.logic(ctx));
    let request = &app.shots.requests[0];
    assert!(!request.issued, "the image captured before the hidden resize is not accepted");
    assert_eq!(request.retries, 1);
    assert_eq!(request.geometry.viewport, Some(egui::vec2(32.0, 400.0)));
    assert!(rx.try_recv().is_err(), "a retried capture does not reply yet");

    // The next real pass issues against the new size and its image is accepted.
    let resized = raw_of(egui::vec2(32.0, 24.0));
    let callbacks = issue_with(&ctx, &mut app, resized.clone());
    assert_eq!(callbacks.len(), 1, "the retry is issued once against the current geometry");
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([32, 24], egui::Color32::BLUE)));
    let _ = ui_pass(&ctx, resized, |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the retried capture replies");
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// Draining four completed captures replies to all of them and permits a new admission.
fn a_drained_capture_frees_its_slot_for_the_next_admission() {
    let ctx = sized_context(egui::vec2(8.0, 8.0));
    let mut app = test_app();
    let mut receivers = Vec::new();
    for _ in 0..SCREENSHOT_MAX_REQUESTS {
        let (tx, rx) = std::sync::mpsc::channel();
        admit(&mut app, &ctx, None, tx);
        receivers.push(rx);
    }
    for request in &mut app.shots.requests {
        request.ready_at = 0.0;
    }
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    for callback in &callbacks {
        callback.clone().complete(std::sync::Arc::new(egui::ColorImage::filled([8, 8], egui::Color32::GREEN)));
    }
    assert_eq!(app.shots.requests.len(), SCREENSHOT_MAX_REQUESTS);
    // Draining before admitting is what `logic` does: completed requests release first.
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    assert_eq!(app.shots.requests.len(), 0);
    assert!(receivers.iter_mut().all(|rx| rx.try_recv().is_ok()));
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    assert!(rx.try_recv().is_err(), "a new request is admitted once the drained slots are free");
    assert_eq!(app.shots.requests.len(), 1);
}

#[test]
/// Uses `screen_rect` when `inner_rect` is absent, applies native scale times zoom, and advances the epoch on a hidden resize.
fn raw_input_geometry_uses_screen_rect_when_inner_rect_is_absent() {
    // Wayland always omits `inner_rect`; the fresh `RawInput::screen_rect` is the only size.
    let ctx = sized_context(egui::vec2(800.0, 600.0));
    let mut app = test_app();
    let mut raw = raw_of(egui::vec2(1280.0, 720.0));
    if let Some(v) = raw.viewports.get_mut(&egui::ViewportId::ROOT) {
        v.inner_rect = None;
        v.native_pixels_per_point = Some(2.0);
        v.minimized = Some(false);
        v.occluded = Some(false);
    }
    app.raw_input_hook(&ctx, &mut raw);
    let geo = app.capture_geometry.expect("the raw frame records its geometry");
    assert_eq!(geo.viewport, Some(egui::vec2(1280.0, 720.0)));
    assert_eq!(geo.pixels_per_point, 2.0, "native scale times zoom");
    assert_eq!(geo.expected_size(), Some([2560, 1440]));

    let epoch = app.capture_epoch;
    // A hidden resize lands on a logic-only tick: no UI pass ran, but the raw frame is fresh.
    let mut hidden = raw_of(egui::vec2(640.0, 480.0));
    if let Some(v) = hidden.viewports.get_mut(&egui::ViewportId::ROOT) {
        v.inner_rect = None;
        v.native_pixels_per_point = Some(2.0);
        v.minimized = Some(false);
        v.occluded = Some(true);
    }
    app.raw_input_hook(&ctx, &mut hidden);
    let _ = ctx.run_logic(&hidden, |ctx| app.logic(ctx));
    assert!(app.capture_epoch > epoch, "a hidden resize is a new epoch");
    assert_eq!(app.capture_geometry.expect("geometry").viewport, Some(egui::vec2(640.0, 480.0)));
}

#[test]
/// Minimized geometry has no size and reserves unknown bytes; restoring at the same size still changes the epoch.
fn minimized_geometry_has_no_size_and_restore_bumps_the_epoch() {
    let ctx = egui::Context::default();
    let mut app = test_app();
    let mut minimized = raw_of(egui::vec2(800.0, 600.0));
    if let Some(v) = minimized.viewports.get_mut(&egui::ViewportId::ROOT) {
        v.inner_rect = None;
        v.minimized = Some(true);
    }
    app.raw_input_hook(&ctx, &mut minimized);
    let geo = app.capture_geometry.expect("recorded");
    assert_eq!(geo.viewport, None, "a minimized window has no capturable size");
    assert_eq!(geo.visible, Some(false));

    // Admission while minimized reserves the unknown-size maximum and waits.
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    assert_eq!(app.shots.requests[0].bytes, UNKNOWN_CAPTURE_BYTES);
    assert!(rx.try_recv().is_err());
    let epoch = app.capture_epoch;

    // Restoring at the same logical size is still a new surface generation.
    let mut restored = raw_of(egui::vec2(800.0, 600.0));
    if let Some(v) = restored.viewports.get_mut(&egui::ViewportId::ROOT) {
        v.inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)));
        v.minimized = Some(false);
        v.occluded = Some(false);
    }
    app.raw_input_hook(&ctx, &mut restored);
    assert!(app.capture_epoch > epoch, "minimize/restore at the same size changes the epoch");
    assert_eq!(app.capture_geometry.expect("geometry").viewport, Some(egui::vec2(800.0, 600.0)));
    app.shots.cancel_all();
}

#[test]
/// Switching documents before issuance cancels the request once with a document-changed error and writes nothing.
fn a_document_switch_before_issuance_cancels_without_writing() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let path = temp_png("doc-queued");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
    app.shots.requests[0].ready_at = 0.0;
    let admitted = app.shots.requests[0].document;
    assert!(app.run("file.new", json!({})).is_ok());
    assert_ne!(app.session.active().map(|d| d.uid), admitted, "the test switched documents");

    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert!(callbacks.is_empty(), "a capture for another document is never issued");
    let reply = rx.try_recv().expect("the queued request is cancelled once");
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("document changed")), "{reply}");
    assert!(!path.exists(), "no pixels for the old document are written to its path");
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// Switching documents after issuance rejects the completed image once and writes nothing.
fn a_document_switch_after_issuance_rejects_the_completed_image() {
    let ctx = sized_context(egui::vec2(64.0, 48.0));
    let mut app = test_app();
    let path = temp_png("doc-issued");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);
    assert!(app.run("file.new", json!({})).is_ok());
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([64, 48], egui::Color32::RED)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the switched document is reported once");
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("document changed")), "{reply}");
    assert!(!path.exists(), "a completion after a document switch never writes");
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// A request bound to another viewport is rejected once with a viewport error and writes nothing.
fn a_completion_for_another_viewport_is_rejected() {
    let ctx = sized_context(egui::vec2(32.0, 24.0));
    let mut app = test_app();
    let path = temp_png("viewport");
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx);
    app.shots.requests[0].ready_at = 0.0;
    let callbacks = issue_with(&ctx, &mut app, egui::RawInput::default());
    assert_eq!(callbacks.len(), 1);
    // A slot/request bound to a different viewport must not fill this one.
    app.shots.requests[0].viewport_id = egui::ViewportId::from_hash_of("other-window");
    callbacks[0].clone().complete(std::sync::Arc::new(egui::ColorImage::filled([32, 24], egui::Color32::RED)));
    let _ = ui_pass(&ctx, egui::RawInput::default(), |ui| drain_screenshots(&mut app, ui.ctx()));
    let reply = rx.try_recv().expect("the viewport mismatch is reported once");
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("another viewport")), "{reply}");
    assert!(!path.exists());
    assert_eq!(app.shots.reserved(), 0);
}

#[test]
/// Accepts one-pixel rounding on a single axis, but rejects opposite-axis deviation, aspect distortion and more than one pixel of error.
fn validate_image_rejects_distortion_but_allows_one_pixel_rounding() {
    let image = |w: usize, h: usize| egui::ColorImage::filled([w, h], egui::Color32::RED);
    assert!(validate_image(&image(100, 100), [100, 100]).is_ok());
    assert!(validate_image(&image(101, 100), [100, 100]).is_ok(), "one pixel of rounding on one axis is legitimate");
    assert!(validate_image(&image(100, 101), [100, 100]).is_ok());
    assert!(validate_image(&image(101, 99), [100, 100]).is_err(), "independent per-axis tolerance must not accept a distorted image");
    assert!(validate_image(&image(99, 101), [100, 100]).is_err());
    assert!(validate_image(&image(102, 100), [100, 100]).is_err(), "more than one pixel per axis is a different surface");
}

#[test]
/// Checks the encoding working-set threshold, the base64 reply cap, and the retained-plus-encoding peak budget.
fn encoding_working_set_is_bounded_for_admitted_captures() {
    assert!(encoding_working_bytes([8191, 2048], false).expect("checked") <= SCREENSHOT_MAX_ENCODING_BYTES);
    assert!(encoding_working_bytes([8192, 8192], false).expect("checked") > SCREENSHOT_MAX_ENCODING_BYTES);
    assert!(encoding_working_bytes([4000, 3000], true).expect("checked") <= SCREENSHOT_MAX_ENCODING_BYTES);
    assert!(encoding_working_bytes([5000, 4000], true).expect("checked") > SCREENSHOT_MAX_ENCODING_BYTES);
    // The base64 reply itself is capped too, before the string exists.
    let png = png_worst_bytes(5000 * 4000 * 4, 4000).expect("checked");
    assert!(base64_len(png).expect("checked") > SCREENSHOT_MAX_BASE64_BYTES);
    // Retained requests plus one encoding working set are held to a hard peak.
    assert!(check_peak_budget(0, [64, 48], false).is_ok());
    assert!(check_peak_budget(SCREENSHOT_MAX_AGGREGATE_BYTES, [9000, 4096], false).is_err());
}

#[test]
/// An oversized base64 capture is refused with nothing reserved, while the same capture as a path is admitted.
fn base64_capture_over_the_encoding_budget_is_rejected_before_admission() {
    let ctx = sized_context(egui::vec2(5000.0, 4000.0));
    let mut app = test_app();
    let (tx, rx) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, None, tx);
    let reply = rx.try_recv().expect("an oversized base64 capture is refused at admission");
    assert_eq!(reply["ok"], false, "{reply}");
    assert!(reply["error"].as_str().is_some_and(|e| e.contains("encoding") || e.contains("base64")), "the error names the encoding budget: {reply}");
    assert!(app.shots.requests.is_empty(), "a refused request reserves nothing");

    // The same capture as a file is admitted within the per-image and encoding budgets.
    let path = temp_png("encoding-path");
    let (tx2, rx2) = std::sync::mpsc::channel();
    admit(&mut app, &ctx, Some(path.to_string_lossy().into_owned()), tx2);
    assert!(rx2.try_recv().is_err(), "the path form is admitted");
    assert_eq!(app.shots.requests.len(), 1);
    app.shots.cancel_all();
}

#[test]
/// Oversized text, repeated maximum texts, keys and moves are refused without changing the queue, and the byte/event limits are both enforced.
fn synthetic_text_key_and_move_producers_honour_the_queue_budget() {
    let ctx = egui::Context::default();
    let mut app = test_app();
    // One oversized text event is refused before it is queued.
    let huge = "x".repeat(MAX_SYNTHETIC_TEXT_BYTES + 1);
    let Outcome::Done(reply) = handle(&mut app, &ctx, "ui.text", &json!({"text": huge})) else { panic!("ui.text returns immediately") };
    assert_eq!(reply["ok"], false, "{reply}");
    assert!(app.synthetic.is_empty());

    // The byte budget stops repeated maximum-size texts; the queue is left unchanged.
    let max = "x".repeat(MAX_SYNTHETIC_TEXT_BYTES);
    let mut refused_at = None;
    for i in 0..64 {
        let Outcome::Done(reply) = handle(&mut app, &ctx, "ui.text", &json!({"text": max.clone()})) else { panic!("ui.text returns immediately") };
        if reply["ok"] == false {
            refused_at = Some((i, app.synthetic.len()));
            break;
        }
    }
    let (accepted, queued) = refused_at.expect("the synthetic byte budget is enforced");
    assert!(accepted > 0 && accepted < 64, "{accepted} texts fit");
    let Outcome::Done(reply) = handle(&mut app, &ctx, "ui.text", &json!({"text": max})) else { panic!("ui.text returns immediately") };
    assert_eq!(reply["ok"], false);
    assert_eq!(app.synthetic.len(), queued, "a refused text leaves the queue alone");

    // Event count: keys and moves use the same overall budget as pointer gestures.
    app.synthetic.clear();
    let pos = egui::pos2(1.0, 1.0);
    app.synthetic.push_back(vec![egui::Event::PointerMoved(pos); MAX_QUEUED_SYNTHETIC_EVENTS - 1]);
    let Outcome::Done(reply) = handle(&mut app, &ctx, "ui.key", &json!({"key":"enter"})) else { panic!("ui.key returns immediately") };
    assert_eq!(reply["ok"], false, "two key events do not fit in the last slot: {reply}");
    assert_eq!(app.synthetic.iter().map(Vec::len).sum::<usize>(), MAX_QUEUED_SYNTHETIC_EVENTS - 1);

    app.synthetic.clear();
    app.synthetic.push_back(vec![egui::Event::PointerMoved(pos); MAX_QUEUED_SYNTHETIC_EVENTS]);
    let Outcome::Done(reply) = handle(&mut app, &ctx, "ui.move", &json!({"x":2.0,"y":2.0})) else { panic!("ui.move returns immediately") };
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(app.synthetic.iter().map(Vec::len).sum::<usize>(), MAX_QUEUED_SYNTHETIC_EVENTS);
}

#[test]
fn python_ui_commands_are_listed() {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, crate::Services::default());
    let Outcome::Done(reply) = handle(&mut app, &egui::Context::default(), "engine.commands", &json!({"search": "python"})) else {
        panic!("command discovery must return immediately");
    };
    let commands = reply["result"].as_array().cloned().unwrap_or_default();
    let listed = [("ui.pythonMode", "{on?: bool} (toggles without it) → {on}; a refused start is an error")];
    for (id, docs) in listed {
        let command = commands.iter().find(|c| c["id"] == id).unwrap_or_else(|| panic!("engine.commands lists {id}"));
        assert_eq!(command["ui"], true, "{id}");
        assert_eq!(command["params"], docs, "{id}");
    }
}

/// Runs `command` through the control channel, as one line of a batch: no frame runs after it.
fn execute(app: &mut SheetApp, ctx: &egui::Context, command: &str, params: Json) {
    let Outcome::Done(reply) = handle(app, ctx, "engine.execute", &json!({"command": command, "params": params})) else {
        panic!("{command} answers at once");
    };
    assert_eq!(reply["ok"], true, "{command}: {reply}");
}

fn inspected(app: &mut SheetApp, ctx: &egui::Context) -> Json {
    let Outcome::Done(reply) = handle(app, ctx, "ui.inspect", &json!({})) else { panic!("ui.inspect answers at once") };
    reply["result"].clone()
}

#[test]
fn a_python_draft_on_code_hidden_in_the_same_batch_shows_no_code() {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    session.execute("python.set", json!({"cell": "B2", "code": "secret = 41\nsecret + 1"})).unwrap();
    session.execute("home.formatCells", json!({"range": "B2", "style": {"protection": {"hidden": true}}})).unwrap();
    session.execute("selection.set", json!({"cell": "B2"})).unwrap();
    let mut app = SheetApp::new(session, crate::Services::default());
    let ctx = egui::Context::default();
    execute(&mut app, &ctx, "ui.pythonMode", json!({"on": true}));
    let before = inspected(&mut app, &ctx);
    assert_eq!(before["editor"]["text"], "secret = 41\nsecret + 1", "unprotected, the draft shows its code");
    // Protected on the session itself, so no app command drops the draft first: `ui.inspect`
    // still leaves the code out, checked when it is called.
    app.session.execute("review.protectSheet", json!({})).unwrap();
    let after = inspected(&mut app, &ctx);
    assert!(app.editor.as_ref().is_some_and(|e| e.python), "the draft is still open");
    assert!(!after.to_string().contains("secret"), "{after}");
    assert_eq!(after["editor"]["text"], Json::Null);
    assert_eq!(after["editor"]["hidden"], true);
    // Out of Python mode the draft still holds the hidden cell's code.
    if let Some(e) = app.editor.as_mut() {
        e.leave_python();
    }
    assert!(app.editor.as_ref().is_some_and(|e| !e.python && e.text.contains("secret")), "an ordinary draft holds the code");
    let left = inspected(&mut app, &ctx);
    assert!(!left.to_string().contains("secret"), "{left}");
    assert_eq!(left["editor"]["hidden"], true);
}

/// Answers one control request that must answer at once with `ok`.
fn call(app: &mut SheetApp, ctx: &egui::Context, method: &str, params: Json) {
    let Outcome::Done(reply) = handle(app, ctx, method, &params) else { panic!("{method} answers at once") };
    assert_eq!(reply["ok"], true, "{method}: {reply}");
}

/// A Python draft of hidden code at B2 on Sheet1, protected after the draft opened, with an
/// unprotected sheet `Other` after it when `other` is set.
fn hidden_python_draft(other: bool) -> (SheetApp, egui::Context) {
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    session.execute("python.set", json!({"cell": "B2", "code": "secret = 41\nsecret + 1"})).unwrap();
    session.execute("home.formatCells", json!({"range": "B2", "style": {"protection": {"hidden": true}}})).unwrap();
    if other {
        session.execute("home.insertSheet", json!({"name": "Other", "before": 1})).unwrap();
        session.execute("sheet.activate", json!({"sheet": 0})).unwrap();
    }
    session.execute("selection.set", json!({"cell": "B2"})).unwrap();
    let mut app = SheetApp::new(session, crate::Services::default());
    let ctx = egui::Context::default();
    execute(&mut app, &ctx, "ui.pythonMode", json!({"on": true}));
    // On the session itself, so the draft stays open for `ui.inspect` to be checked against.
    app.session.execute("review.protectSheet", json!({})).unwrap();
    assert_eq!(inspected(&mut app, &ctx)["editor"]["hidden"], true, "hidden on its own sheet");
    (app, ctx)
}

#[test]
fn a_python_draft_shows_no_code_when_another_sheet_takes_its_index_in_the_same_batch() {
    let (mut app, ctx) = hidden_python_draft(false);
    // An unprotected sheet now sits at the draft's old index, in the same batch.
    app.session.execute("home.insertSheet", json!({"name": "New", "before": 0})).unwrap();
    let after = inspected(&mut app, &ctx);
    assert!(!after.to_string().contains("secret"), "{after}");
    assert_eq!(after["editor"]["text"], Json::Null);
    assert_eq!(after["editor"]["hidden"], true);
}

#[test]
fn a_python_draft_shows_no_code_once_its_sheet_is_deleted_in_the_same_batch() {
    let (mut app, ctx) = hidden_python_draft(true);
    // The draft's own sheet goes, and the unprotected `Other` takes its index.
    app.session.execute("home.deleteSheet", json!({"sheet": "Sheet1"})).unwrap();
    assert_eq!(app.session.active().map(|d| d.wb.sheets.len()), Some(1));
    let after = inspected(&mut app, &ctx);
    assert!(!after.to_string().contains("secret"), "{after}");
    assert_eq!(after["editor"]["hidden"], true);
    // The next app command (an undo that brings the protected sheet back) drops the draft, since
    // its code is hidden there; deleting a draft's sheet is covered by
    // `a_draft_whose_sheet_is_deleted_is_dropped_with_a_message`.
    execute(&mut app, &ctx, "edit.undo", json!({}));
    assert!(app.editor.is_none(), "the draft is dropped");
    assert_eq!(app.message.as_ref().map(|m| m.1.as_str()), Some(crate::app::DRAFT_CODE_HIDDEN));
    assert_eq!(inspected(&mut app, &ctx)["editor"], Json::Null);
}

/// Sheet `name`'s A1: its input text and whether it is a Python cell.
fn a1(app: &SheetApp, name: &str) -> Option<(String, bool)> {
    let d = app.session.active()?;
    let sh = d.wb.sheet(d.wb.sheet_index(name)?)?;
    sh.cell(gridcraft_engine::core::CellRef::new(0, 0)).map(|c| (c.input_text(), c.py.is_some()))
}

#[test]
fn a_draft_is_committed_on_its_own_sheet_after_a_sheet_is_inserted_before_it() {
    // An ordinary draft and a Python draft on Sheet1!A1, then a sheet inserted before Sheet1.
    for python in [false, true] {
        let mut app = test_app();
        let ctx = egui::Context::default();
        if python {
            execute(&mut app, &ctx, "ui.pythonMode", json!({"on": true}));
            call(&mut app, &ctx, "ui.edit.text", json!({"text": "1 + 1"}));
        } else {
            call(&mut app, &ctx, "ui.edit.begin", json!({"text": "42"}));
        }
        execute(&mut app, &ctx, "home.insertSheet", json!({"name": "New", "before": 0}));
        call(&mut app, &ctx, "ui.edit.commit", json!({"move": "none"}));
        assert!(app.editor.is_none(), "python {python}");
        assert_eq!(a1(&app, "New"), None, "python {python}: nothing lands on the inserted sheet");
        let got = a1(&app, "Sheet1");
        if python {
            assert_eq!(got.map(|g| g.1), Some(true), "the code lands on the draft's own sheet");
        } else {
            assert_eq!(got, Some(("42".to_string(), false)), "the entry lands on the draft's own sheet");
        }
    }
}

#[test]
fn a_draft_whose_sheet_is_deleted_is_dropped_with_a_message() {
    for python in [false, true] {
        let mut app = test_app();
        let ctx = egui::Context::default();
        execute(&mut app, &ctx, "home.insertSheet", json!({"name": "Other", "before": 1}));
        execute(&mut app, &ctx, "sheet.activate", json!({"sheet": 0}));
        if python {
            execute(&mut app, &ctx, "ui.pythonMode", json!({"on": true}));
            call(&mut app, &ctx, "ui.edit.text", json!({"text": "1 + 1"}));
        } else {
            call(&mut app, &ctx, "ui.edit.begin", json!({"text": "42"}));
        }
        execute(&mut app, &ctx, "home.deleteSheet", json!({"sheet": "Sheet1"}));
        // Dropped once the command is done, not left drawn on the sheet in its place.
        assert!(app.editor.is_none(), "python {python}: the draft is dropped");
        assert_eq!(inspected(&mut app, &ctx)["editor"], Json::Null, "python {python}");
        assert_eq!(a1(&app, "Other"), None, "python {python}: nothing lands on the sheet in its place");
        assert_eq!(app.message.as_ref().map(|m| m.1.as_str()), Some(crate::app::DRAFT_SHEET_GONE), "python {python}");
    }
}

#[test]
fn an_open_draft_follows_its_sheet_when_a_sheet_is_inserted_before_it() {
    for python in [false, true] {
        let mut app = test_app();
        let ctx = egui::Context::default();
        if python {
            execute(&mut app, &ctx, "ui.pythonMode", json!({"on": true}));
        } else {
            call(&mut app, &ctx, "ui.edit.begin", json!({"text": "=SUM("}));
        }
        if let Some(ed) = app.editor.as_mut() {
            ed.point_anchor = Some((0, gridcraft_engine::core::CellRef::parse("C3").unwrap()));
        }
        execute(&mut app, &ctx, "home.insertSheet", json!({"name": "Before", "before": 0}));
        let ed = app.editor.as_ref().expect("the draft stays open");
        assert_eq!(ed.sheet, 1, "python {python}: the draft is on Sheet1, now at index 1, so it is drawn there");
        assert_eq!(ed.point_anchor, None, "python {python}: the anchor's sheet index no longer names its sheet");
        // A rename changes no index: nothing is let go.
        if let Some(ed) = app.editor.as_mut() {
            ed.point_anchor = Some((1, gridcraft_engine::core::CellRef::parse("C3").unwrap()));
        }
        execute(&mut app, &ctx, "sheet.rename", json!({"sheet": "Before", "name": "First"}));
        assert!(app.editor.as_ref().is_some_and(|e| e.sheet == 1 && e.point_anchor.is_some()), "python {python}");
    }
}

#[test]
fn a_draft_of_python_code_that_protection_hides_is_dropped_with_a_message() {
    // In Python mode, and after leaving it (an ordinary draft of a Python cell's code).
    for leave in [false, true] {
        let mut session = gridcraft_engine::Session::new();
        session.new_workbook();
        session.execute("python.set", json!({"cell": "B2", "code": "secret = 41"})).unwrap();
        session.execute("cell.set", json!({"cell": "C2", "input": "=1+1"})).unwrap();
        session.execute("home.formatCells", json!({"range": "B2:C2", "style": {"protection": {"hidden": true, "locked": false}}})).unwrap();
        session.execute("selection.set", json!({"cell": "B2"})).unwrap();
        let mut app = SheetApp::new(session, crate::Services::default());
        let ctx = egui::Context::default();
        execute(&mut app, &ctx, "ui.pythonMode", json!({"on": true}));
        if leave {
            execute(&mut app, &ctx, "ui.pythonMode", json!({"on": false}));
            assert!(app.editor.as_ref().is_some_and(|e| !e.python && e.text.contains("secret")), "an ordinary draft of the code");
        }
        execute(&mut app, &ctx, "review.protectSheet", json!({}));
        assert!(app.editor.is_none(), "leave {leave}: neither editor keeps hidden code on screen");
        assert_eq!(app.message.as_ref().map(|m| m.1.as_str()), Some(crate::app::DRAFT_CODE_HIDDEN), "leave {leave}");
        assert!(!inspected(&mut app, &ctx).to_string().contains("secret"));
    }
    // An ordinary formula draft on a hidden cell stays open, as on main.
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    session.execute("cell.set", json!({"cell": "C2", "input": "=1+1"})).unwrap();
    session.execute("home.formatCells", json!({"range": "C2", "style": {"protection": {"hidden": true, "locked": false}}})).unwrap();
    session.execute("selection.set", json!({"cell": "C2"})).unwrap();
    let mut app = SheetApp::new(session, crate::Services::default());
    let ctx = egui::Context::default();
    call(&mut app, &ctx, "ui.edit.begin", json!({"text": "=2+2"}));
    execute(&mut app, &ctx, "review.protectSheet", json!({}));
    assert!(app.editor.as_ref().is_some_and(|e| e.text == "=2+2"), "a formula draft is unchanged");
}

#[test]
fn a_draft_typed_over_a_hidden_python_cell_stays_open_through_commands() {
    // An unlocked, hidden Python cell on a protected sheet: typing replaces it, as in Excel. The
    // draft holds only what was typed, so a command neither drops it nor hides it from ui.inspect.
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    session.execute("python.set", json!({"cell": "B2", "code": "secret = 41"})).unwrap();
    session.execute("home.formatCells", json!({"range": "B2", "style": {"protection": {"hidden": true, "locked": false}}})).unwrap();
    session.execute("review.protectSheet", json!({})).unwrap();
    session.execute("selection.set", json!({"cell": "B2"})).unwrap();
    let mut app = SheetApp::new(session, crate::Services::default());
    let ctx = egui::Context::default();
    call(&mut app, &ctx, "ui.edit.begin", json!({"text": "7"}));
    execute(&mut app, &ctx, "home.insertSheet", json!({"name": "Other", "before": 1}));
    assert!(app.editor.as_ref().is_some_and(|e| e.text == "7"), "the typed entry stays open");
    assert!(app.message.is_none(), "{:?}", app.message);
    let after = inspected(&mut app, &ctx);
    assert_eq!(after["editor"]["text"], "7");
    assert_eq!(after["editor"]["hidden"], false);
    call(&mut app, &ctx, "ui.edit.commit", json!({"move": "none"}));
    let b2 = app.session.active().and_then(|d| d.wb.sheet(0)).and_then(|sh| sh.cell(gridcraft_engine::core::CellRef::new(1, 1)).cloned());
    assert_eq!(b2.map(|c| (c.input_text(), c.py.is_some())), Some(("7".to_string(), false)), "the typed entry replaces the Python cell");
}

#[test]
fn python_mode_does_not_start_on_a_draft_typed_inside_a_spill() {
    // Typing on a spill child is an ordinary entry; switching that draft to Python mode would let
    // Ctrl+Enter write code into a cell another cell's result spills over, which every other way
    // into Python mode refuses.
    let mut session = gridcraft_engine::Session::new();
    session.new_workbook();
    session.execute("cell.set", json!({"cell": "A1", "input": "=SEQUENCE(3)"})).unwrap();
    session.execute("selection.set", json!({"cell": "A2"})).unwrap();
    let mut app = SheetApp::new(session, crate::Services::default());
    let ctx = egui::Context::default();
    call(&mut app, &ctx, "ui.edit.begin", json!({"text": "5"}));
    let Outcome::Done(reply) = handle(&mut app, &ctx, "engine.execute", &json!({"command": "ui.pythonMode", "params": {"on": true}})) else {
        panic!("ui.pythonMode answers at once");
    };
    assert_eq!(reply["ok"], false, "the start is refused: {reply}");
    assert!(app.editor.as_ref().is_some_and(|e| !e.python && e.text == "5"), "the draft stays an ordinary entry");
    call(&mut app, &ctx, "ui.edit.commit", json!({"move": "none"}));
    let a2 = app.session.active().and_then(|d| d.wb.sheet(0)).and_then(|sh| sh.cell(gridcraft_engine::core::CellRef::new(1, 0)).cloned());
    assert_eq!(a2.map(|c| (c.input_text(), c.py.is_some())), Some(("5".to_string(), false)));
}

#[test]
fn test_apply_assistant_proposal_success_and_undo() {
    let mut app = test_app();
    let doc_rev = app.session.active().map(|d| d.revision).unwrap_or(0);
    let doc_uid = app.session.active().map(|d| d.uid).unwrap_or(0);

    let proposal = gridcraft_ai::EditProposal {
        proposal_id: "prop_unit_1".to_string(),
        workbook_uid: doc_uid,
        base_revision: doc_rev,
        sheet_name: "Sheet1".to_string(),
        target_range: "A1:B1".to_string(),
        explanation: "Test apply".to_string(),
        diffs: vec![],
        values_matrix: vec![vec![json!("Alpha"), json!(999)]],
        status: gridcraft_ai::ProposalStatus::Pending,
        created_at_ms: 0,
    };

    app.assistant.pending_proposal = Some(proposal);
    let res = crate::assistant::apply_assistant_proposal(&mut app);
    assert!(res.is_ok(), "apply should succeed: {:?}", res);
    assert!(app.assistant.pending_proposal.is_none(), "pending proposal must be cleared on success");
    assert!(app.assistant.last_apply_error.is_none());
    assert!(app.assistant.messages.iter().any(|m| m.text.contains("✓ Applied proposed changes")));

    // Verify cell values in SheetApp
    let a1_val = app.run("cell.get", json!({"cell": "A1"})).unwrap();
    assert_eq!(a1_val.get("value").and_then(Json::as_str), Some("Alpha"));

    // Verify undo
    app.run("edit.undo", json!({})).unwrap();
    let a1_after_undo = app.run("cell.get", json!({"cell": "A1"})).unwrap();
    assert!(a1_after_undo.get("value").is_none() || a1_after_undo.get("value").unwrap().is_null());
}

#[test]
fn test_apply_assistant_proposal_failure_preserves_proposal() {
    let mut app = test_app();
    let doc_uid = app.session.active().map(|d| d.uid).unwrap_or(0);

    // 1. Stale proposal with invalid base revision 999
    let stale_proposal = gridcraft_ai::EditProposal {
        proposal_id: "prop_stale".to_string(),
        workbook_uid: doc_uid,
        base_revision: 999,
        sheet_name: "Sheet1".to_string(),
        target_range: "A1:B1".to_string(),
        explanation: "Stale apply".to_string(),
        diffs: vec![],
        values_matrix: vec![vec![json!("Test"), json!(123)]],
        status: gridcraft_ai::ProposalStatus::Pending,
        created_at_ms: 0,
    };

    app.assistant.pending_proposal = Some(stale_proposal);
    let res = crate::assistant::apply_assistant_proposal(&mut app);
    assert!(res.is_err(), "stale apply should fail");
    assert!(app.assistant.pending_proposal.is_some(), "pending proposal MUST be preserved on failure for safe retry/review");
    assert!(app.assistant.last_apply_error.is_some());
    assert!(app.assistant.last_apply_error.as_ref().unwrap().contains("Stale proposal"));

    // 2. Mismatched sheet proposal
    let sheet_mismatch_proposal = gridcraft_ai::EditProposal {
        proposal_id: "prop_wrong_sheet".to_string(),
        workbook_uid: doc_uid,
        base_revision: app.session.active().map(|d| d.revision).unwrap_or(0),
        sheet_name: "NonExistentSheet".to_string(),
        target_range: "A1:B1".to_string(),
        explanation: "Wrong sheet".to_string(),
        diffs: vec![],
        values_matrix: vec![vec![json!("Test"), json!(123)]],
        status: gridcraft_ai::ProposalStatus::Pending,
        created_at_ms: 0,
    };

    app.assistant.pending_proposal = Some(sheet_mismatch_proposal);
    let res_mismatch = crate::assistant::apply_assistant_proposal(&mut app);
    assert!(res_mismatch.is_err(), "sheet mismatch apply should fail");
    assert!(app.assistant.pending_proposal.is_some(), "pending proposal MUST be preserved on failure");
    assert!(app.assistant.last_apply_error.as_ref().unwrap().contains("not the active sheet"));
}

#[test]
fn test_reject_assistant_proposal() {
    let mut app = test_app();
    let proposal = gridcraft_ai::EditProposal {
        proposal_id: "prop_reject".to_string(),
        workbook_uid: 0,
        base_revision: 0,
        sheet_name: "Sheet1".to_string(),
        target_range: "A1".to_string(),
        explanation: "To reject".to_string(),
        diffs: vec![],
        values_matrix: vec![vec![json!("X")]],
        status: gridcraft_ai::ProposalStatus::Pending,
        created_at_ms: 0,
    };

    app.assistant.pending_proposal = Some(proposal);
    let res = crate::assistant::reject_assistant_proposal(&mut app);
    assert!(res.is_ok());
    assert!(app.assistant.pending_proposal.is_none());
    assert!(app.assistant.messages.iter().any(|m| m.text.contains("rejected")));
}

#[test]
fn test_cancellation_and_superseding_in_assistant_state() {
    let mut app = test_app();
    app.assistant.current_request_id = 5;

    let proposal = gridcraft_ai::EditProposal {
        proposal_id: "prop_old".to_string(),
        workbook_uid: 0,
        base_revision: 0,
        sheet_name: "Sheet1".to_string(),
        target_range: "A1".to_string(),
        explanation: "Old".to_string(),
        diffs: vec![],
        values_matrix: vec![vec![json!("Old")]],
        status: gridcraft_ai::ProposalStatus::Pending,
        created_at_ms: 0,
    };

    // 1. Superseded event from previous request_id = 4
    let (tx, rx) = std::sync::mpsc::channel();
    app.assistant.rx = Some(rx);
    tx.send(crate::assistant::WorkerEvent::ProposalGenerated { request_id: 4, proposal: proposal.clone() }).unwrap();
    app.assistant.poll_events();
    assert!(app.assistant.pending_proposal.is_none(), "superseded event from request 4 must not be accepted");

    // 2. Event for current request_id = 5 when cancelled
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    app.assistant.cancel_handle = Some(cancel);
    tx.send(crate::assistant::WorkerEvent::ProposalGenerated { request_id: 5, proposal }).unwrap();
    app.assistant.poll_events();
    assert!(app.assistant.pending_proposal.is_none(), "cancelled request must never leave an applicable proposal");
}

#[test]
fn test_assistant_layout_switching_preserves_state() {
    let mut app = test_app();
    app.assistant.init();

    let doc = app.session.active().unwrap();
    let doc_rev = doc.revision;
    let doc_uid = doc.uid;

    let proposal = gridcraft_ai::EditProposal {
        proposal_id: "prop_layout_test".to_string(),
        workbook_uid: doc_uid,
        base_revision: doc_rev,
        sheet_name: "Sheet1".to_string(),
        target_range: "A6:B6".to_string(),
        explanation: "Add totals in layout test".to_string(),
        diffs: vec![
            gridcraft_ai::CellDiff {
                cell_ref: "A6".to_string(),
                old_display: "Subtotal".to_string(),
                new_display: "Total".to_string(),
                is_formula: false,
            },
            gridcraft_ai::CellDiff {
                cell_ref: "B6".to_string(),
                old_display: "4500".to_string(),
                new_display: "=SUM(B1:B5)".to_string(),
                is_formula: true,
            },
        ],
        values_matrix: vec![vec![json!("Total"), json!("=SUM(B1:B5)")]],
        status: gridcraft_ai::ProposalStatus::Pending,
        created_at_ms: 0,
    };

    app.assistant.pending_proposal = Some(proposal.clone());
    app.assistant.composer_text = "Draft in progress".to_string();
    app.assistant.messages.push(gridcraft_ai::AssistantMessage {
        id: "msg_user_1".to_string(),
        role: "user".to_string(),
        text: "Please add totals".to_string(),
        tool_status: None,
        created_at_ms: 0,
    });

    // 1. Initial layout: Sidebar (default)
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::Sidebar);
    assert!(app.assistant.pending_proposal.is_some());
    assert_eq!(app.assistant.composer_text, "Draft in progress");

    // 2. Switch to Review Workspace
    app.assistant.set_layout(crate::assistant::AssistantLayout::ReviewWorkspace);
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::ReviewWorkspace);
    assert!(app.assistant.pending_proposal.is_some(), "Pending proposal must be preserved after switching to Review Workspace");
    assert_eq!(app.assistant.composer_text, "Draft in progress", "Composer text must be preserved");
    assert_eq!(app.assistant.messages.len(), 2);

    // 3. Switch to Command Bar
    app.assistant.set_layout(crate::assistant::AssistantLayout::CommandBar);
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::CommandBar);
    assert!(app.assistant.pending_proposal.is_some(), "Pending proposal must be preserved after switching to Command Bar");
    assert_eq!(app.assistant.composer_text, "Draft in progress");

    // 4. Switch back to Sidebar and apply proposal
    app.assistant.set_layout(crate::assistant::AssistantLayout::Sidebar);
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::Sidebar);
    let applied = crate::assistant::apply_assistant_proposal(&mut app).expect("Apply should succeed");
    assert_eq!(applied.target_range, "A6:B6");
    assert!(app.assistant.pending_proposal.is_none(), "Proposal must clear after successful apply");

    // 5. Single-step undo
    app.run_or_alert("edit.undo", json!({}));
}

#[test]
fn test_assistant_appearance_prefs_persistence_and_reset() {
    let mut app = test_app();
    app.assistant.init();

    // Set custom appearance settings
    app.assistant.appearance.layout = crate::assistant::AssistantLayout::ReviewWorkspace;
    app.assistant.appearance.panel_width = 440.0;
    app.assistant.appearance.tray_height = 310.0;
    app.assistant.appearance.density = crate::assistant::InterfaceDensity::Compact;
    app.assistant.appearance.theme = crate::assistant::ThemePreference::Dark;
    app.assistant.appearance.conversation_collapsed = true;
    app.assistant.appearance.tray_collapsed = true;

    // Use an isolated path for test persistence to prevent parallel test races
    let temp_path = std::env::temp_dir().join(format!("ai_appearance_test_{}.json", std::process::id()));
    crate::assistant::save_appearance_prefs_to(&temp_path, &app.assistant.appearance);

    // Reload from isolated test file
    let loaded = crate::assistant::load_appearance_prefs_from(&temp_path);
    let _ = std::fs::remove_file(&temp_path);

    assert_eq!(loaded.layout, crate::assistant::AssistantLayout::ReviewWorkspace);
    assert_eq!(loaded.panel_width, 440.0);
    assert_eq!(loaded.tray_height, 310.0);
    assert_eq!(loaded.density, crate::assistant::InterfaceDensity::Compact);
    assert_eq!(loaded.theme, crate::assistant::ThemePreference::Dark);
    assert!(loaded.conversation_collapsed);
    assert!(loaded.tray_collapsed);

    // Reset to defaults
    app.assistant.reset_appearance_defaults();
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::Sidebar);
    assert_eq!(app.assistant.appearance.panel_width, 320.0);
    assert_eq!(app.assistant.appearance.tray_height, 240.0);
    assert_eq!(app.assistant.appearance.density, crate::assistant::InterfaceDensity::Default);
    assert_eq!(app.assistant.appearance.theme, crate::assistant::ThemePreference::FollowAppTheme);
    assert!(!app.assistant.appearance.conversation_collapsed);
    assert!(!app.assistant.appearance.tray_collapsed);
}

#[test]
fn test_workspace_preset_profiles_initialization_and_validity() {
    let prefs = crate::workspace::WorkspacePreferences::default();
    assert_eq!(prefs.version, crate::workspace::PREFS_VERSION);
    assert_eq!(prefs.active_profile, "Classic");

    // All 4 required presets must exist
    assert!(prefs.profiles.contains_key("Classic"));
    assert!(prefs.profiles.contains_key("Focus"));
    assert!(prefs.profiles.contains_key("Data Analysis"));
    assert!(prefs.profiles.contains_key("AI Review"));

    let classic = prefs.profiles.get("Classic").unwrap();
    assert!(!classic.workspace.focus_mode);
    assert!(classic.workspace.show_task_pane);
    assert!(classic.workspace.show_formula_bar);
    assert_eq!(classic.assistant.layout, crate::assistant::AssistantLayout::Sidebar);

    let focus = prefs.profiles.get("Focus").unwrap();
    assert!(focus.workspace.focus_mode);
    assert!(!focus.workspace.show_task_pane);
    assert!(focus.ribbon.collapsed);
    assert_eq!(focus.assistant.layout, crate::assistant::AssistantLayout::CommandBar);

    let data = prefs.profiles.get("Data Analysis").unwrap();
    assert!(!data.workspace.focus_mode);
    assert!(data.workspace.show_task_pane);
    assert_eq!(data.spreadsheet.display_density, crate::assistant::InterfaceDensity::Compact);
    assert_eq!(data.assistant.layout, crate::assistant::AssistantLayout::Sidebar);

    let ai_rev = prefs.profiles.get("AI Review").unwrap();
    assert!(!ai_rev.workspace.focus_mode);
    assert_eq!(ai_rev.assistant.layout, crate::assistant::AssistantLayout::ReviewWorkspace);
    assert_eq!(ai_rev.assistant.preview_style, crate::workspace::ProposalPreviewStyle::SplitTable);
}

fn call_control(app: &mut SheetApp, command: &str, params: serde_json::Value) -> serde_json::Value {
    let Outcome::Done(reply) = handle(app, &egui::Context::default(), command, &params) else {
        panic!("command {command} returned async instead of Done");
    };
    if let Some(res) = reply.get("result") {
        let mut obj = res.clone();
        if let Some(map) = obj.as_object_mut() {
            if let Some(ok_val) = reply.get("ok") {
                map.entry("ok").or_insert_with(|| ok_val.clone());
            }
        }
        obj
    } else {
        reply
    }
}

#[test]
fn test_workspace_profile_switching_and_control_commands() {
    let mut app = test_app();

    // 1. Open customizer via control
    let res = call_control(&mut app, "workspace.customize", json!({ "open": true }));
    assert_eq!(res.get("open").and_then(Json::as_bool), Some(true));
    assert!(app.customizer.is_open);

    // 2. Switch to Focus preset
    let res = call_control(&mut app, "workspace.setProfile", json!({ "profile": "Focus" }));
    assert_eq!(res.get("active_profile").and_then(Json::as_str), Some("Focus"));
    assert_eq!(app.workspace_prefs.active_profile, "Focus");
    assert!(app.workspace_prefs.get_active().workspace.focus_mode);
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::CommandBar);

    // 3. Switch to AI Review preset
    let res = call_control(&mut app, "workspace.setProfile", json!({ "profile": "AI Review" }));
    assert_eq!(res.get("active_profile").and_then(Json::as_str), Some("AI Review"));
    assert_eq!(app.workspace_prefs.active_profile, "AI Review");
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::ReviewWorkspace);

    // 4. Switch to Data Analysis preset
    let res = call_control(&mut app, "workspace.setProfile", json!({ "profile": "Data Analysis" }));
    assert_eq!(res.get("active_profile").and_then(Json::as_str), Some("Data Analysis"));
    assert_eq!(app.workspace_prefs.active_profile, "Data Analysis");

    // 5. Reset to defaults
    let res = call_control(&mut app, "workspace.resetDefaults", json!({}));
    assert_eq!(res.get("active_profile").and_then(Json::as_str), Some("Classic"));
    assert_eq!(app.workspace_prefs.active_profile, "Classic");
    assert_eq!(app.assistant.appearance.layout, crate::assistant::AssistantLayout::Sidebar);

    // 6. Close customizer
    let res = call_control(&mut app, "workspace.customize", json!({ "open": false }));
    assert_eq!(res.get("open").and_then(Json::as_bool), Some(false));
    assert!(!app.customizer.is_open);
}

#[test]
fn test_workspace_profile_duplication_and_export_sanitization() {
    let mut app = test_app();

    // Duplicate Data Analysis into a custom profile
    let res = call_control(&mut app, "workspace.duplicateProfile", json!({
        "source": "Data Analysis",
        "new_name": "Quant Research Lab"
    }));
    assert_eq!(res.get("ok").and_then(Json::as_bool), Some(true));
    assert!(app.workspace_prefs.profiles.contains_key("Quant Research Lab"));

    // Export the profile
    let res = call_control(&mut app, "workspace.exportProfile", json!({
        "name": "Quant Research Lab"
    }));
    assert_eq!(res.get("ok").and_then(Json::as_bool), Some(true));
    let json_text = res.get("json").and_then(Json::as_str).expect("Export JSON must be present");

    // Verify sanitization: NO credentials, API keys, conversations, or workbook paths
    let lower_json = json_text.to_ascii_lowercase();
    assert!(!lower_json.contains("api_key"));
    assert!(!lower_json.contains("apikey"));
    assert!(!lower_json.contains("secret"));
    assert!(!lower_json.contains("token"));
    assert!(!lower_json.contains("credential"));
    assert!(!lower_json.contains("password"));
    assert!(!lower_json.contains("bearer"));

    // Roundtrip import
    let imported = crate::workspace::import_profile_json(json_text).expect("Import must succeed");
    assert_eq!(imported.name, "Quant Research Lab");
    assert!(!imported.is_builtin);
    assert_eq!(imported.spreadsheet.display_density, crate::assistant::InterfaceDensity::Compact);
}

#[test]
fn test_workspace_customization_undo_redo_isolation() {
    let mut app = test_app();
    app.open_customize_workspace();

    let initial_theme = app.customizer.draft.appearance.theme;
    app.customizer.push_undo();

    // Make a visual draft change
    app.customizer.draft.appearance.theme = crate::workspace::ThemeMode::Dark;
    assert_eq!(app.customizer.draft.appearance.theme, crate::workspace::ThemeMode::Dark);

    // Undo the customizer step
    let undone = app.customizer.undo();
    assert!(undone);
    assert_eq!(app.customizer.draft.appearance.theme, initial_theme);

    // Redo the customizer step
    let redone = app.customizer.redo();
    assert!(redone);
    assert_eq!(app.customizer.draft.appearance.theme, crate::workspace::ThemeMode::Dark);

    // Verify workbook session history is completely unaffected
    assert!(gridcraft_engine::cmd::can_undo(&app.session).is_err());
    assert!(gridcraft_engine::cmd::can_redo(&app.session).is_err());
}

#[test]
fn test_workspace_persistence_across_restart() {
    let mut prefs = crate::workspace::WorkspacePreferences::default();
    prefs.active_profile = "Data Analysis".to_string();

    let mut custom = crate::workspace::WorkspaceProfile::classic_preset();
    custom.name = "My Custom Profile".to_string();
    custom.is_builtin = false;
    custom.appearance.accent = crate::workspace::AccentChoice::Amber;
    prefs.profiles.insert("My Custom Profile".to_string(), custom);

    let temp_path = std::env::temp_dir().join(format!("tz_workspace_prefs_test_{}.json", std::process::id()));
    crate::workspace::save_workspace_preferences_to(&temp_path, &prefs);

    let loaded = crate::workspace::load_workspace_preferences_from(&temp_path);
    let _ = std::fs::remove_file(&temp_path);

    assert_eq!(loaded.version, crate::workspace::PREFS_VERSION);
    assert_eq!(loaded.active_profile, "Data Analysis");
    assert!(loaded.profiles.contains_key("My Custom Profile"));
    let loaded_custom = loaded.profiles.get("My Custom Profile").unwrap();
    assert_eq!(loaded_custom.appearance.accent, crate::workspace::AccentChoice::Amber);
}

#[test]
fn test_workspace_preset_switching_preserves_pending_assistant_proposal() {
    let mut app = test_app();
    app.assistant.init();

    let doc = app.session.active().unwrap();
    let doc_rev = doc.revision;
    let doc_uid = doc.uid;

    let proposal = gridcraft_ai::EditProposal {
        proposal_id: "prop_workspace_switch_test".to_string(),
        workbook_uid: doc_uid,
        base_revision: doc_rev,
        sheet_name: "Sheet1".to_string(),
        target_range: "A6:B6".to_string(),
        explanation: "Totals proposal during workspace preset switch".to_string(),
        diffs: vec![
            gridcraft_ai::CellDiff {
                cell_ref: "A6".to_string(),
                old_display: "Old Total".to_string(),
                new_display: "Total".to_string(),
                is_formula: false,
            },
            gridcraft_ai::CellDiff {
                cell_ref: "B6".to_string(),
                old_display: "".to_string(),
                new_display: "=SUM(B1:B5)".to_string(),
                is_formula: true,
            },
        ],
        values_matrix: vec![vec![json!("Total"), json!("=SUM(B1:B5)")]],
        status: gridcraft_ai::ProposalStatus::Pending,
        created_at_ms: 0,
    };

    app.assistant.pending_proposal = Some(proposal.clone());
    app.assistant.composer_text = "Prompt in flight".to_string();

    // Switch across multiple presets
    app.apply_workspace_profile("AI Review");
    assert!(app.assistant.pending_proposal.is_some());
    assert_eq!(app.assistant.pending_proposal.as_ref().unwrap().proposal_id, "prop_workspace_switch_test");
    assert_eq!(app.assistant.composer_text, "Prompt in flight");

    app.apply_workspace_profile("Focus");
    assert!(app.assistant.pending_proposal.is_some());
    assert_eq!(app.assistant.pending_proposal.as_ref().unwrap().proposal_id, "prop_workspace_switch_test");
    assert_eq!(app.assistant.composer_text, "Prompt in flight");

    app.apply_workspace_profile("Classic");
    assert!(app.assistant.pending_proposal.is_some());
    assert_eq!(app.assistant.pending_proposal.as_ref().unwrap().proposal_id, "prop_workspace_switch_test");
    assert_eq!(app.assistant.composer_text, "Prompt in flight");
}
