use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub fn draw_inspect_tab(&mut self, ui: &mut egui::Ui) {
        if self.raw_buffer.is_empty() {
            crate::card_layout::draw_arch_note_card(
                ui,
                "Inspect — VBIOS & PowerPlay Table Overview",
                "No table loaded. Click Load or select a GPU from the top selector to view full VBIOS headers, supported PptArch UI cards, and the 1:1 CLI --inspect report.",
            );
            return;
        }

        self.sync_fields_to_buffer();
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let src_label = self
            .loaded_file
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "GUI Buffer".to_string());
        let full_report = self.format_inspect_report(&src_label);

        let query_id = egui::Id::new("inspect_tab_search_query");
        let mut query = ui.ctx().data(|d| d.get_temp::<String>(query_id).unwrap_or_default());

        ui.horizontal_wrapped(|ui| {
            if ui.button("Copy Inspect Report").clicked() {
                ui.ctx().copy_text(full_report.clone());
                self.status_msg = "Copied full Inspect report to clipboard.".to_string();
            }
            if ui.button("Export Inspect (.txt)").clicked() {
                crate::io::spawn_save_file_dialog(
                    crate::io::FileDialogAction::ExportInspectTxt,
                    "Save File",
                    Some("navitune_inspect.txt"),
                    &[("Text Report (*.txt)", &["txt"])],
                );
            }
            ui.separator();
            ui.label("Filter:");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut query)
                    .desired_width(240.0)
                    .hint_text("Filter cards, fields, 0x offsets, or values..."),
            );
            if resp.changed() {
                ui.ctx().data_mut(|d| d.insert_temp(query_id, query.clone()));
            }
            if !query.is_empty() && ui.small_button("✕").clicked() {
                query.clear();
                ui.ctx().data_mut(|d| d.insert_temp(query_id, query.clone()));
            }
        });
        ui.add_space(6.0);

        let q = query.trim().to_ascii_lowercase();

        crate::card_layout::draw_card(
            ui,
            &format!("Supported UI Cards & Fields — {} ({} B)", arch.label(), self.raw_buffer.len()),

            |ui| {
                egui::Grid::new("inspect_supported_cards_grid")
                    .num_columns(2)
                    .striped(true)
                    .spacing([14.0, 5.0])
                    .show(ui, |ui| {
                        ui.strong("UI Card");
                        ui.strong("Active Fields (PptArch Single Source of Truth)");
                        ui.end_row();
                        for card in crate::ppt_arch::PptArch::ALL_CARDS {
                            if !self.is_card_usable(card) {
                                continue;
                            }
                            let title = arch.card_title(card);
                            let fields = arch.card_inspect_fields(card, &self.raw_buffer);
                            if !q.is_empty()
                                && !title.to_ascii_lowercase().contains(&q)
                                && !fields.to_ascii_lowercase().contains(&q)
                            {
                                continue;
                            }
                            ui.colored_label(egui::Color32::from_rgb(95, 200, 145), title);
                            ui.monospace(fields);
                            ui.end_row();
                        }
                    });
            },
        );

        let filtered_report = if q.is_empty() {
            full_report
        } else {
            let lines: Vec<&str> = full_report
                .lines()
                .filter(|l| l.to_ascii_lowercase().contains(&q))
                .collect();
            if lines.is_empty() {
                format!("(No inspect lines matching '{}')", query.trim())
            } else {
                lines.join("\n")
            }
        };

        crate::card_layout::draw_card(ui, "Full CLI --inspect Report (1:1 Parity Output)", |ui| {
            let mut buf = filtered_report;
            ui.add(
                egui::TextEdit::multiline(&mut buf)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
        });
    }
}
