use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub(crate) fn draw_debug_capture_log(&mut self, ui: &mut egui::Ui) {
        let log_path = crate::ui_theme::debug_log_path();
        let view_id = egui::Id::new("capture_log_show_file");
        let query_id = egui::Id::new("capture_log_search_query");
        let mut show_file = ui.ctx().data(|d| d.get_temp::<bool>(view_id).unwrap_or(false));
        let mut query = ui.ctx().data(|d| d.get_temp::<String>(query_id).unwrap_or_default());

        let session_tail = log_path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default();

        let report_text = if session_tail.is_empty() {
            self.build_full_debug_dump()
        } else {
            format!(
                "{}\n=== SESSION EVENT LOG ({}) ===\n{}",
                self.build_full_debug_dump(),
                log_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "debug.log".into()),
                session_tail
            )
        };

        let raw_display = if show_file {
            if session_tail.is_empty() {
                "No session events recorded in debug.log yet.".to_string()
            } else {
                session_tail.clone()
            }
        } else {
            report_text.clone()
        };

        ui.horizontal_wrapped(|ui| {
            if ui.button("Save to debug.log")
                .on_hover_text("Writes structured Capture Log & session events directly to ~/.config/navitune/debug.log (no raw hex dump)")
                .clicked()
            {
                if let Some(ref lp) = log_path {
                    if let Some(parent) = lp.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match std::fs::write(lp, &report_text) {
                        Ok(_) => {
                            crate::ui_theme::log_debug(
                                "CAPTURE",
                                &format!(
                                    "Saved Capture Log ({} B table, {} modified bytes) to {}",
                                    self.raw_buffer.len(),
                                    self.modified_byte_count(),
                                    lp.display()
                                ),
                            );
                            self.status_msg = format!("Saved Capture Log to {}", lp.display());
                        }
                        Err(e) => self.show_error("debug.log Write Error", &e.to_string()),
                    }
                }
            }
            if ui.button("Copy Log").clicked() {
                ui.ctx().copy_text(raw_display.clone());
                self.status_msg = "Copied Capture Log to clipboard.".to_string();
            }
            if ui.button("Export As...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("navitune_debug.log")
                    .add_filter("Log / Text", &["log", "txt"])
                    .save_file()
                {
                    match std::fs::write(&path, &report_text) {
                        Ok(_) => self.status_msg = format!("Exported Capture Log to {}", path.display()),
                        Err(e) => self.show_error("Export Log Error", &e.to_string()),
                    }
                }
            }
            if ui.button("Clear debug.log").clicked() {
                if let Some(ref lp) = log_path {
                    let _ = std::fs::write(lp, "");
                    crate::ui_theme::log_debug("RESET", "Cleared debug.log from Capture Log tab");
                    self.status_msg = format!("Cleared {}", lp.display());
                }
            }
            ui.separator();
            if ui.selectable_label(!show_file, "Full Capture Report").clicked() {
                show_file = false;
                ui.ctx().data_mut(|d| d.insert_temp(view_id, show_file));
            }
            if ui.selectable_label(show_file, "Live debug.log Events").clicked() {
                show_file = true;
                ui.ctx().data_mut(|d| d.insert_temp(view_id, show_file));
            }
        });

        ui.horizontal(|ui| {
            ui.label("Search:");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut query)
                    .desired_width(240.0)
                    .hint_text("Filter by field, 0x offset, section, or tag..."),
            );
            if resp.changed() {
                ui.ctx().data_mut(|d| d.insert_temp(query_id, query.clone()));
            }
            if !query.is_empty() && ui.small_button("✕").clicked() {
                query.clear();
                ui.ctx().data_mut(|d| d.insert_temp(query_id, query.clone()));
            }
            let total_lines = raw_display.lines().count();
            if query.trim().is_empty() {
                if let Some(ref lp) = log_path {
                    ui.small(format!("{} lines  |  {}", total_lines, lp.display()));
                }
            } else {
                let q = query.trim().to_ascii_lowercase();
                let matched = raw_display.lines().filter(|l| l.to_ascii_lowercase().contains(&q)).count();
                ui.colored_label(
                    egui::Color32::from_rgb(95, 200, 145),
                    egui::RichText::new(format!("{} / {} matching lines", matched, total_lines)).small(),
                );
            }
        });

        let display_text = if query.trim().is_empty() {
            raw_display
        } else {
            let q = query.trim().to_ascii_lowercase();
            let filtered: Vec<&str> = raw_display
                .lines()
                .filter(|l| l.to_ascii_lowercase().contains(&q))
                .collect();
            if filtered.is_empty() {
                format!("(No lines matching '{}')", query.trim())
            } else {
                filtered.join("\n")
            }
        };

        ui.add_space(4.0);
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            let mut buf = display_text;
            ui.add(
                egui::TextEdit::multiline(&mut buf)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
        });
    }
}
