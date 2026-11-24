use eframe::egui;
use crate::{AdvanceOsTab, PowerPlayApp};

impl PowerPlayApp {
    pub fn draw_advance(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::MacOS, "macOS");
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::Windows, "Windows");
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::Linux, "Linux");
        });
        ui.separator();
        ui.add_space(4.0);

        if self.raw_buffer.is_empty() {
            crate::card_layout::draw_card(ui, "OS Presets", |ui| {
                ui.label("No table loaded");
            });
            return;
        }

        match self.advance_os_tab {
            AdvanceOsTab::MacOS => self.draw_advance_macos(ui),
            AdvanceOsTab::Windows => self.draw_advance_windows(ui),
            AdvanceOsTab::Linux => self.draw_advance_linux(ui),
        }

    }
}
