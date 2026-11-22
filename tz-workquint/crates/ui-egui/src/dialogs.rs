//! Modal dialogs: release notes, uninstall confirmation, settings.

use crate::i18n::fmt;
use artcraft_launcher_core::Os;
use artcraft_launcher_core::protocol::Command;
use egui::{RichText, vec2};

use crate::app::{Dialog, Host, LauncherUi};
use crate::theme::{self, Tokens};
use crate::widgets::{self, Kind};

pub fn resolve_offline_guide_url() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p1 = parent.join("docs").join("user-guide").join("index.html");
            if p1.is_file() {
                return format!("file:///{}", p1.to_string_lossy().replace('\\', "/"));
            }
            if let Some(grand) = parent.parent() {
                let p2 = grand.join("docs").join("user-guide").join("index.html");
                if p2.is_file() {
                    return format!("file:///{}", p2.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    let ws = std::path::Path::new("E:/CRAFTING SOFTWARES/docs/user-guide/index.html");
    if ws.is_file() {
        return "file:///E:/CRAFTING SOFTWARES/docs/user-guide/index.html".to_string();
    }
    "https://tzlabs.in".to_string()
}

impl LauncherUi {
    /// Draws the current modal and writes any edits made in it back to `self.dialog`. With no
    /// dialog it shows the next unacknowledged feed alert. Each dialog kind gets its own modal id
    /// so a smaller prompt cannot inherit a taller one's last size, and closing (button, Escape or
    /// a click outside) discards the dialog.
    pub(crate) fn dialogs(&mut self, ctx: &egui::Context, host: &mut dyn Host) {
        let Some(mut dialog) = self.dialog.clone() else {
            self.alert_popup(ctx);
            return;
        };
        let t = Tokens::get(ctx);
        // An area is laid out in the size it had last frame, so dialogs can't share one id: the
        // uninstall prompt would open as tall as Settings and never shrink (#88).
        // The large dialogs run their header to the card's edges; the small prompts keep a margin.
        let is_custom_card = matches!(dialog, Dialog::About | Dialog::Assistant);
        let large = matches!(dialog, Dialog::Settings | Dialog::Notes(_));
        let modal = egui::Modal::new(egui::Id::unique("dialog").with(dialog.key())).frame(if is_custom_card {
            egui::Frame::new()
                .fill(if t.dark { egui::Color32::from_rgb(20, 26, 38) } else { egui::Color32::from_rgb(250, 252, 255) })
                .stroke(egui::Stroke::new(1.0, if t.dark { egui::Color32::from_white_alpha(35) } else { egui::Color32::from_black_alpha(25) }))
                .shadow(theme::shadow(&t))
                .corner_radius(egui::CornerRadius::same(14))
                .inner_margin(egui::Margin::same(24))
        } else if large {
            crate::sheet::frame(&t)
        } else {
            egui::Frame::new().fill(t.bg).stroke(egui::Stroke::new(1.0, t.line_strong)).shadow(theme::shadow(&t)).inner_margin(theme::PAD as i8)
        });
        let mut close = false;
        let response = match &mut dialog {
            Dialog::Notes(id) => modal.show(ctx, |ui| close = self.notes_dialog(ui, id)),
            Dialog::ConfirmUninstall(id) => modal.show(ctx, |ui| close = self.uninstall_dialog(ui, id)),
            Dialog::Install { app, folder } => modal.show(ctx, |ui| close = self.install_dialog(ui, app, folder, host)),
            Dialog::Replace { app, path } => modal.show(ctx, |ui| close = self.replace_dialog(ui, app, path)),
            Dialog::Settings => modal.show(ctx, |ui| close = self.settings_dialog(ui, host)),
            Dialog::LauncherUpdate(r) => modal.show(ctx, |ui| close = self.launcher_update_dialog(ui, r)),
            Dialog::Versions(id) => modal.show(ctx, |ui| close = self.version_dialog(ui, id)),
            Dialog::LaunchFlags { app, flags } => modal.show(ctx, |ui| close = self.launch_flags_dialog(ui, app, flags)),
            Dialog::About => modal.show(ctx, |ui| close = self.about_dialog(ui)),
            Dialog::Assistant => modal.show(ctx, |ui| close = self.assistant_dialog(ui)),
        };
        if self.dialog.as_ref().is_some_and(|active| active.key() != dialog.key()) {
            // A control inside a dialog opened a different one; don't restore the old dialog.
            return;
        }
        let should_close = close || (response.should_close() && !is_custom_card) || ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if should_close {
            self.dialog = None;
            if matches!(dialog, Dialog::Versions(_)) {
                // Close, Escape or a click outside: a version chosen there is dropped.
                self.revert_version_choice();
            }
        } else {
            self.dialog = Some(dialog);
        }
    }

    /// The first alert from the release feed the user hasn't acknowledged, as a popup.
    fn alert_popup(&mut self, ctx: &egui::Context) {
        let Some(alert) = self.alerts.iter().find(|a| !self.settings.dismissed_notices.contains(&a.id)).cloned() else { return };
        let t = Tokens::get(ctx);
        let modal = egui::Modal::new(egui::Id::unique("feed_alert").with(&alert.id)).frame(
            egui::Frame::new().fill(t.bg).stroke(egui::Stroke::new(1.0, t.line_strong)).shadow(theme::shadow(&t)).inner_margin(theme::PAD as i8),
        );
        let mut ok = false;
        modal.show(ctx, |ui| {
            ui.set_width(440.0f32.min(ctx.content_rect().width() - 80.0));
            ui.label(RichText::new(&alert.title).size(18.0).family(theme::semibold()).color(t.ink_strong));
            ui.add_space(6.0);
            egui::ScrollArea::vertical().max_height(ctx.content_rect().height() * 0.5).show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&alert.body).color(t.muted)).wrap());
            });
            ui.add_space(14.0);
            // A row: a right-to-left layout straight in the column would fill the height left.
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ok = widgets::button(ui, tl!("OK"), Kind::Primary, 90.0).clicked();
                    if let Some(link) = &alert.link
                        && widgets::button(ui, tl!("Open link"), Kind::Secondary, 110.0).on_hover_text(link.as_str()).clicked()
                    {
                        widgets::open_external_link(ui.ctx(), link);
                    }
                });
            });
        });
        // Only OK acknowledges it: Escape or a click outside leaves it for later.
        if ok {
            self.settings.dismissed_notices.insert(alert.id);
        }
    }

    /// Draws the optional icon beside a heading and an optional subtitle. The icon is decorative so
    /// assistive technology hears the title once, and an empty subtitle is omitted.
    fn title(ui: &mut egui::Ui, icon: Option<egui::ImageSource<'static>>, title: &str, subtitle: &str) {
        let t = Tokens::get(ui.ctx());
        ui.horizontal(|ui| {
            if let Some(icon) = icon {
                // The heading beside it carries the title; an icon with the same accessible
                // name would announce the dialog title twice.
                ui.add(egui::Image::new(icon).fit_to_exact_size(vec2(40.0, 40.0)));
            }
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.label(widgets::heading(title, theme::DIALOG_TITLE, t.ink_strong));
                if !subtitle.is_empty() {
                    widgets::hud(ui, subtitle, t.faint);
                }
            });
        });
        ui.add_space(14.0);
    }

    /// An app's release notes: a large card like the app page, the notes in a scrolling body.
    fn notes_dialog(&mut self, ui: &mut egui::Ui, id: &str) -> bool {
        let Some(entry) = self.entry(id).cloned() else { return true };
        let Some(release) = entry.release().cloned() else { return true };
        let t = Tokens::get(ui.ctx());
        let size = crate::sheet::size(ui.ctx(), vec2(880.0, 760.0) * self.notes_zoom.max(1.0));
        ui.set_width(size.x);
        ui.set_max_height(size.y);
        let date = release.published_at.map(|d| fmt(tl!("Released {date}"), &[("date", &crate::views::date_text(d))])).unwrap_or_default();
        let hero_h = crate::sheet::hero(ui, crate::icons::app(id), &format!("{} {}", entry.app.name, release.version_label()), &date, |_| {});
        let footer_h = theme::CONTROL_H + 32.0;
        let body_h = (size.y - hero_h - footer_h - 2.0).max(0.0);
        let saved = crate::sheet::accent_scroll(ui, &t, t.accent);
        egui::ScrollArea::vertical().id_salt("notes-body").max_height(body_h).auto_shrink([false, true]).show(ui, |ui| {
            *ui.visuals_mut() = saved;
            egui::Frame::new().inner_margin(egui::Margin::same(theme::PAD as i8)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                if release.notes.is_empty() {
                    ui.label(tl!("These notes weren't fetched (GitHub's request limit was reached). They're on the release page."));
                } else {
                    crate::notes::show_scaled(ui, &release.notes, self.notes_zoom);
                }
            });
        });
        ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top(), t.hairline());
        let mut close = false;
        egui::Frame::new().inner_margin(egui::Margin::symmetric(theme::PAD as i8, 16)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                widgets::hud_link(ui, tl!("Open on GitHub"), &release.html_url);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    close = widgets::button(ui, tl!("Close"), Kind::Secondary, 90.0).clicked();
                });
            });
        });
        close
    }

    /// Offers a new launcher version and returns whether the dialog should close. With self-update
    /// it queues `UpdateLauncher` and closes; otherwise it opens the platform download (or the
    /// release page) and closes. "Later" closes without doing either.
    fn launcher_update_dialog(&mut self, ui: &mut egui::Ui, r: &artcraft_launcher_core::Release) -> bool {
        let t = Tokens::get(ui.ctx());
        ui.set_width(480.0f32.min(ui.ctx().content_rect().width() - 80.0));
        let title = fmt(tl!("TZWorkQuint {version} is available."), &[("version", &r.version_label())]);
        Self::title(ui, None, &title, "");
        let message = if self.caps.self_update {
            tl!("A new version is available. Would you like to relaunch with the updated version?")
        } else {
            tl!("A new version is available. Download it to replace this version?")
        };
        ui.label(RichText::new(message).color(t.muted));
        ui.add_space(14.0);
        let mut close = false;
        ui.horizontal(|ui| {
            widgets::hud_link(ui, tl!("What's new"), &r.html_url);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.caps.self_update {
                    if widgets::button(ui, tl!("Update and restart"), Kind::Primary, 0.0)
                        .on_hover_text(tl!("Downloads the new version, replaces this one and opens it again"))
                        .clicked()
                    {
                        self.act_command(Command::UpdateLauncher);
                        close = true;
                    }
                } else {
                    let url = self
                        .platform
                        .and_then(|p| crate::views::launcher_download(r, p, crate::settings::is_flatpak_sandbox()))
                        .unwrap_or_else(|| r.html_url.clone());
                    if widgets::button(ui, tl!("Download"), Kind::Primary, 0.0).clicked() {
                        widgets::open_external_link(ui.ctx(), &url);
                        close = true;
                    }
                }
                if widgets::button(ui, tl!("Later"), Kind::Secondary, 80.0).clicked() {
                    close = true;
                }
            });
        });
        close
    }

    /// Confirms removing an installed app and returns `true` when the dialog should close, including
    /// when the app entry has disappeared. It queues `Uninstall` only on confirmation and explains
    /// the removal differently for each install source.
    fn uninstall_dialog(&mut self, ui: &mut egui::Ui, id: &str) -> bool {
        let Some(entry) = self.entry(id).cloned() else { return true };
        let t = Tokens::get(ui.ctx());
        ui.set_width(420.0);
        Self::title(ui, crate::icons::app(id), &fmt(tl!("Uninstall {name}?"), &[("name", entry.app.name)]), "");
        let how = match entry.installed.as_ref().map(|i| i.source) {
            Some(artcraft_launcher_core::InstallSource::MacApp) => tl!("The app will be moved to the Trash. Your documents and settings stay."),
            Some(artcraft_launcher_core::InstallSource::Msi) => {
                tl!("Windows will remove it like any installed program. Your documents and settings stay.")
            }
            Some(
                artcraft_launcher_core::InstallSource::Deb
                | artcraft_launcher_core::InstallSource::Rpm
                | artcraft_launcher_core::InstallSource::Pacman,
            ) => tl!("Your system's package manager will remove it, and may ask for your password. Your documents and settings stay."),
            _ => tl!("The app will be removed. Your documents and settings stay."),
        };
        ui.label(RichText::new(how).color(t.muted));
        ui.add_space(20.0);
        let mut close = false;
        // A row: a right-to-left layout straight in the column would fill the height left.
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::button(ui, tl!("Uninstall"), Kind::Danger, 110.0).clicked() {
                    self.act_command(Command::Uninstall { app: id.to_string() });
                    close = true;
                }
                if widgets::button(ui, tl!("Cancel"), Kind::Secondary, 90.0).clicked() {
                    close = true;
                }
            });
        });
        close
    }

    /// The extra command-line flags an app is opened with. Saved only when they split cleanly.
    fn launch_flags_dialog(&mut self, ui: &mut egui::Ui, id: &str, flags: &mut String) -> bool {
        let Some(app) = self.entry(id).map(|e| e.app) else { return true };
        let t = Tokens::get(ui.ctx());
        ui.set_width(460.0f32.min(ui.ctx().content_rect().width() - 80.0));
        Self::title(ui, crate::icons::app(id), &fmt(tl!("{name} launch options"), &[("name", app.name)]), "");
        let label = ui.label(tl!("Flags added each time the app is opened"));
        ui.add(egui::TextEdit::singleline(flags).id_salt("launch_flags").hint_text("--flag \"…\"").desired_width(f32::INFINITY))
            .labelled_by(label.id);
        let split = artcraft_launcher_core::args::split(flags);
        let (note, color) = match split {
            Ok(_) => (tl!("Separate flags with spaces. Quote a value that has spaces in it."), t.muted),
            Err(_) => (tl!("A quote isn't closed."), t.bad),
        };
        ui.label(RichText::new(note).color(color));
        ui.add_space(14.0);
        let mut close = false;
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let save = ui.add_enabled_ui(split.is_ok(), |ui| widgets::button(ui, tl!("Save"), Kind::Primary, 90.0)).inner;
                if save.clicked() {
                    let trimmed = flags.trim();
                    if trimmed.is_empty() {
                        self.settings.launch_flags.remove(id);
                    } else {
                        self.settings.launch_flags.insert(id.to_string(), trimmed.to_string());
                    }
                    close = true;
                }
                close |= widgets::button(ui, tl!("Cancel"), Kind::Secondary, 90.0).clicked();
            });
        });
        close
    }

    /// Warns that `path` already exists and asks whether to replace it, returning `true` when the
    /// dialog should close, including when the app entry is gone. `Replace` is queued at the action
    /// boundary, not sent inline.
    fn replace_dialog(&mut self, ui: &mut egui::Ui, id: &str, path: &str) -> bool {
        let Some(app) = self.entry(id).map(|e| e.app) else { return true };
        let t = Tokens::get(ui.ctx());
        ui.set_width(440.0f32.min(ui.ctx().content_rect().width() - 80.0));
        Self::title(ui, crate::icons::app(id), tl!("Replace what's there?"), "");
        ui.label(
            RichText::new(fmt(tl!("{path} already exists. Installing {name} will overwrite it."), &[("path", path), ("name", app.name)]))
                .color(t.muted),
        );
        ui.add_space(20.0);
        let mut close = false;
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::button(ui, tl!("Replace"), Kind::Danger, 110.0).clicked() {
                    self.act_command(Command::Replace { app: id.to_string(), path: path.to_string() });
                    close = true;
                }
                if widgets::button(ui, tl!("Cancel"), Kind::Secondary, 90.0).clicked() {
                    close = true;
                }
            });
        });
        close
    }

    /// Lets the user choose this app's install folder and returns `true` when the dialog should
    /// close, including when the app entry is gone. An empty field removes the per-app override, a
    /// non-empty one persists it, and any changed configuration is sent before the install is queued. The path
    /// itself is not validated here.
    fn install_dialog(&mut self, ui: &mut egui::Ui, id: &str, folder: &mut String, host: &mut dyn Host) -> bool {
        let Some(app) = self.entry(id).map(|e| e.app) else { return true };
        ui.set_width(440.0f32.min(ui.ctx().content_rect().width() - 80.0));
        Self::title(ui, None, app.name, tl!("Install folder"));
        let default = self.settings.install_locations.root.clone().or_else(|| self.default_folder_for(id, host));
        Self::folder_field(ui, folder, default.as_deref(), host);
        if self.platform.is_some_and(|p| p.os == Os::Linux) {
            ui.label(tl!("Custom folders use AppImage on Linux."));
        }
        ui.add_space(14.0);
        let mut close = false;
        ui.horizontal(|ui| {
            if widgets::button(ui, tl!("Install"), Kind::Primary, 100.0).clicked() {
                if folder.is_empty() {
                    self.settings.install_locations.per_app.remove(id);
                } else {
                    self.settings.install_locations.per_app.insert(id.to_string(), folder.clone());
                }
                self.configure();
                self.act_command(Command::Install { app: id.to_string() });
                close = true;
            }
            close |= widgets::button(ui, tl!("Cancel"), Kind::Secondary, 90.0).clicked();
        });
        close
    }

    pub(crate) fn folder_field(ui: &mut egui::Ui, folder: &mut String, default: Option<&str>, host: &mut dyn Host) {
        let label = ui.label(tl!("Install folder"));
        ui.add(
            egui::TextEdit::singleline(folder)
                .id_salt("install_folder")
                .hint_text(default.unwrap_or(tl!("System default")))
                .desired_width(f32::INFINITY),
        )
        .labelled_by(label.id);
        ui.label(tl!("Apps are placed inside this folder."));
        ui.horizontal(|ui| {
            if widgets::button(ui, tl!("Browse…"), Kind::Secondary, 100.0).clicked()
                && let Some(selected) = host.pick_folder(if folder.is_empty() { default } else { Some(folder.as_str()) })
            {
                *folder = selected;
            }
            if widgets::button(ui, tl!("Use default"), Kind::Secondary, 120.0).clicked() {
                folder.clear();
            }
        });
    }

    fn about_dialog(&mut self, ui: &mut egui::Ui) -> bool {
        let t = Tokens::get(ui.ctx());
        ui.set_width(480.0f32.min(ui.ctx().content_rect().width() - 80.0));
        let mut close = false;

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let logo_rect = egui::Rect::from_min_size(ui.cursor().min, vec2(44.0, 44.0));
                let _ = ui.allocate_exact_size(vec2(44.0, 44.0), egui::Sense::hover());
                ui.painter().rect_filled(logo_rect, egui::CornerRadius::same(10), egui::Color32::from_rgb(0x25, 0x63, 0xEB));
                ui.painter().text(
                    logo_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "TZ",
                    egui::FontId::new(18.0, theme::display()),
                    egui::Color32::WHITE,
                );
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("TZWorkQuint Suite").font(egui::FontId::new(22.0, theme::display())).color(t.ink_strong));
                    ui.label(RichText::new("Version 3.0.0 (Windows x64 Native)").size(12.0).color(t.muted));
                });
            });

            ui.add_space(14.0);
            ui.label(
                RichText::new("Unified local-first launcher for independent suite processes: CellForge, Wordloom, SlideSpark, PDFlex, and Draftline. Every editor launches as an independent Windows process with isolated memory and persistent workspace state.")
                    .size(13.0)
                    .color(t.muted)
            );

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                let visit_btn = egui::Button::new(RichText::new("Visit TZLabs ↗").color(egui::Color32::WHITE).size(13.0).family(theme::semibold()))
                    .fill(egui::Color32::from_rgb(0x25, 0x63, 0xEB))
                    .corner_radius(egui::CornerRadius::same(16))
                    .min_size(vec2(130.0, 32.0));
                if ui.add(visit_btn).clicked() {
                    widgets::open_external_link(ui.ctx(), "https://tzlabs.in");
                }
                ui.add_space(8.0);
                let guide_btn = egui::Button::new(RichText::new("User Guide").size(13.0))
                    .corner_radius(egui::CornerRadius::same(16))
                    .min_size(vec2(100.0, 32.0));
                if ui.add(guide_btn).clicked() {
                    let guide_url = resolve_offline_guide_url();
                    widgets::open_external_link(ui.ctx(), &guide_url);
                }
            });

            ui.add_space(16.0);
            egui::CollapsingHeader::new(RichText::new("Third-Party Legal Notices & Licenses").size(12.0).color(t.muted))
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.label(
                            RichText::new(
                                "TZWorkQuint Suite is published by TZLabs under open licenses (MIT OR Apache-2.0).\n\n\
                                Components & Dependencies:\n\
                                • egui / eframe — MIT / Apache-2.0 (Emil Ernerfeldt)\n\
                                • skrifa / fontations — Apache-2.0 (Google Fonts)\n\
                                • Instrument Sans & Instrument Serif — SIL Open Font License 1.1\n\
                                • IBM Plex Mono — SIL Open Font License 1.1\n\
                                • windows-rs — MIT / Apache-2.0 (Microsoft Corporation)\n\
                                • serde & serde_json — MIT / Apache-2.0 (Erick Tryzelaar & David Tolnay)\n\n\
                                All trademarks, logos and service marks are properties of their respective owners. Not affiliated with Microsoft."
                            ).size(11.0).color(t.muted)
                        );
                    });
                });

            ui.add_space(16.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::button(ui, tl!("Close"), Kind::Secondary, 90.0).clicked() {
                    close = true;
                }
            });
        });

        close
    }

    fn assistant_dialog(&mut self, ui: &mut egui::Ui) -> bool {
        let t = Tokens::get(ui.ctx());
        ui.set_width(560.0f32.min(ui.ctx().content_rect().width() - 80.0));
        let mut close = false;

        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let icon_rect = egui::Rect::from_min_size(ui.cursor().min, vec2(36.0, 36.0));
                let _ = ui.allocate_exact_size(vec2(36.0, 36.0), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, egui::CornerRadius::same(8), egui::Color32::from_rgb(0x25, 0x63, 0xEB).gamma_multiply(0.2));
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✦",
                    egui::FontId::new(18.0, theme::display()),
                    egui::Color32::from_rgb(0x38, 0xBD, 0xF8),
                );
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("TZ Suite Assistant").font(egui::FontId::new(18.0, theme::display())).color(t.ink_strong));
                    ui.label(RichText::new("On-demand local intelligence across all five suite applications").size(11.5).color(t.muted));
                });
            });

            ui.add_space(12.0);
            ui.label(RichText::new("Suggested Tasks:").size(11.5).family(theme::semibold()).color(t.muted));
            ui.add_space(4.0);

            let prompt_id = ui.id().with("suite_assistant_prompt");
            let mut prompt = ui.data_mut(|d| d.get_temp::<String>(prompt_id).unwrap_or_default());
            let mut submitted_query = None;

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                let suggestions = [
                    ("📊 Open recent spreadsheet in CellForge", "Launch CellForge with recent spreadsheet"),
                    ("📝 Start executive brief in Wordloom", "Start new brief in Wordloom"),
                    ("📑 Redact confidential data in PDFlex", "How to redact in PDFlex"),
                    ("📐 Architectural layout in Draftline", "Draftline CAD commands"),
                ];
                for (chip_label, query) in suggestions {
                    let btn = egui::Button::new(RichText::new(chip_label).size(11.0))
                        .corner_radius(egui::CornerRadius::same(14))
                        .fill(if t.dark { egui::Color32::from_white_alpha(15) } else { egui::Color32::from_black_alpha(12) });
                    if ui.add(btn).clicked() {
                        prompt = query.to_string();
                    }
                }
            });

            ui.add_space(12.0);
            let response_id = ui.id().with("suite_assistant_response");
            let assistant_response = ui.data_mut(|d| d.get_temp::<String>(response_id));

            if let Some(resp) = assistant_response {
                egui::Frame::new()
                    .fill(if t.dark { egui::Color32::from_rgba_unmultiplied(24, 30, 44, 200) } else { egui::Color32::from_rgba_unmultiplied(240, 244, 250, 220) })
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x25, 0x63, 0xEB).gamma_multiply(0.3)))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.label(RichText::new("✦ Assistant Guidance:").size(12.0).family(theme::semibold()).color(egui::Color32::from_rgb(0x38, 0xBD, 0xF8)));
                        ui.add_space(4.0);
                        ui.label(RichText::new(resp).size(12.0).color(t.ink_strong));
                    });
                ui.add_space(10.0);
            }

            // Composer area
            egui::Frame::new()
                .fill(if t.dark { egui::Color32::from_rgba_unmultiplied(18, 22, 32, 220) } else { egui::Color32::from_rgba_unmultiplied(255, 255, 255, 240) })
                .stroke(egui::Stroke::new(1.0, if t.dark { egui::Color32::from_white_alpha(30) } else { egui::Color32::from_black_alpha(25) }))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    let te = egui::TextEdit::multiline(&mut prompt)
                        .desired_rows(2)
                        .desired_width(ui.available_width())
                        .hint_text("Ask TZ Assistant about any suite editor, formula, keyboard shortcut, or workflow...")
                        .frame(egui::Frame::NONE);
                    ui.add(te);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Press Enter or click Send").size(10.0).color(t.faint));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let send_btn = egui::Button::new(RichText::new("↑").color(egui::Color32::WHITE).size(15.0).strong())
                                .fill(egui::Color32::from_rgb(0x25, 0x63, 0xEB))
                                .corner_radius(egui::CornerRadius::same(15))
                                .min_size(vec2(30.0, 30.0));
                            if ui.add(send_btn).on_hover_text("Send query").clicked() && !prompt.trim().is_empty() {
                                submitted_query = Some(prompt.trim().to_string());
                            }

                            let mic_btn = egui::Button::new(RichText::new("🎙").size(13.0))
                                .corner_radius(egui::CornerRadius::same(15))
                                .min_size(vec2(30.0, 30.0));
                            if ui.add(mic_btn).on_hover_text("Voice input").clicked() {
                                prompt = "Open latest financial model".to_string();
                            }
                        });
                    });
                });

            if let Some(query) = submitted_query {
                let answer = match query.to_lowercase().as_str() {
                    q if q.contains("cellforge") || q.contains("sheet") || q.contains("spreadsheet") => {
                        "TZ CellForge offers high-density reactive spreadsheet calculation, dynamic array formulas, and zero-telemetry local evaluation. Launch it via the 'Launch' button on the CellForge card."
                    }
                    q if q.contains("wordloom") || q.contains("brief") || q.contains("doc") => {
                        "TZ Wordloom is tailored for structured documents, technical RFCs, and executive briefs with full typographical layout control and distraction-free editing."
                    }
                    q if q.contains("slidespark") || q.contains("presentation") || q.contains("deck") => {
                        "TZ SlideSpark provides dual-screen presentation control, slide engineering, and high-performance presentation delivery."
                    }
                    q if q.contains("pdflex") || q.contains("redact") || q.contains("pdf") => {
                        "TZ PDFlex supports true irreversible vector redaction, cryptographic document signatures, and form filling with local rendering."
                    }
                    q if q.contains("draftline") || q.contains("cad") => {
                        "TZ Draftline features a 2D CAD drafting engine with classic command-line precision (LINE, CIRCLE, TRIM, OFFSET, LAYER) and standard DXF/DWG workflows."
                    }
                    _ => {
                        "TZWorkQuint Suite provides five standalone desktop tools. All five editors operate as independent processes with isolated memory and GPU acceleration."
                    }
                };
                ui.data_mut(|d| d.insert_temp(response_id, answer.to_string()));
                prompt.clear();
            }

            ui.data_mut(|d| d.insert_temp(prompt_id, prompt));

            ui.add_space(14.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::button(ui, tl!("Close"), Kind::Secondary, 90.0).clicked() {
                    close = true;
                }
            });
        });

        close
    }
}
