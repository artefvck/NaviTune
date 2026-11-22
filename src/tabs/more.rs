pub use crate::tabs::log::DebugSubTab;
use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub fn render_debug_more(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.columns(2, |cols| {
                self.render_more_power_settings_card(&mut cols[0]);
                self.render_more_throttler_card(&mut cols[1]);
            });
        });
    }
}
