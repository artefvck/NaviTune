pub use crate::tabs::log::DebugSubTab;
use crate::PowerPlayApp;
use eframe::egui;

#[derive(PartialEq, Clone, Copy, Default)]
pub enum MoreSubTab {
    #[default]
    PowerThrottler,
    SafeTune,
    Advance,
}

impl PowerPlayApp {
    pub fn render_debug_more(&mut self, ui: &mut egui::Ui) {
        let sub_id = egui::Id::new("more_active_subtab");
        let mut sub = ui.ctx().data(|d| d.get_temp::<MoreSubTab>(sub_id).unwrap_or_default());

        ui.horizontal(|ui| {
            ui.selectable_value(&mut sub, MoreSubTab::PowerThrottler, "1. Power & Throttler");
            ui.selectable_value(&mut sub, MoreSubTab::SafeTune, "2. Safe Tune");
            ui.selectable_value(&mut sub, MoreSubTab::Advance, "3. Advance");
        });
        ui.ctx().data_mut(|d| d.insert_temp(sub_id, sub));
        ui.separator();

        match sub {
            MoreSubTab::PowerThrottler => {
                ui.columns(2, |cols| {
                    self.render_more_power_settings_card(&mut cols[0]);
                    self.render_more_throttler_card(&mut cols[1]);
                });
            }
            MoreSubTab::SafeTune => self.draw_safe_tune_tab(ui),
            MoreSubTab::Advance => self.draw_advance(ui),
        }
    }
}
