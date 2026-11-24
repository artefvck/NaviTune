use crate::PowerPlayApp;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
pub enum DebugSubTab {
    Audit,
    CaptureLog,
    Inspect,
    NativeOffset,
}

impl PowerPlayApp {
    pub fn draw_log(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::Audit, "1. Offset Audit");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::CaptureLog, "2. Capture Log");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::Inspect, "3. Inspect");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::NativeOffset, "4. Native Offset");
            ui.separator();
            if ui.button("Struct Window").clicked() {
                self.show_struct_inspector_window = !self.show_struct_inspector_window;
            }
        });
        ui.separator();
        match self.debug_subtab {
            DebugSubTab::Audit => self.draw_debug_audit(ui),
            DebugSubTab::CaptureLog => self.draw_debug_capture_log(ui),
            DebugSubTab::Inspect => self.draw_inspect_tab(ui),
            DebugSubTab::NativeOffset => self.draw_debug_native_offset(ui),
        }
        self.render_struct_inspector_viewport(ui.ctx());
    }
}
