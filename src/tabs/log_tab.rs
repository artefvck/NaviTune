use crate::PowerPlayApp;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
pub enum DebugSubTab {
    Audit,
    CaptureLog,
    NativeOffset,
}

impl PowerPlayApp {
    pub fn draw_log_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::Audit, "1. Offset Audit");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::CaptureLog, "2. Capture Log");
            ui.selectable_value(&mut self.debug_subtab, DebugSubTab::NativeOffset, "3. Native Offset");
            ui.separator();
            if ui.button("Struct Window").clicked() {
                self.show_struct_inspector_window = !self.show_struct_inspector_window;
            }
            if ui.button("Dump Audit -> debug.log").clicked() {
                self.dump_audit_to_debug_log();
            }
        });
        ui.separator();
        match self.debug_subtab {
            DebugSubTab::Audit => self.draw_debug_audit(ui),
            DebugSubTab::CaptureLog => self.draw_debug_capture_log(ui),
            DebugSubTab::NativeOffset => self.draw_debug_native_offset(ui),
        }
        self.render_struct_inspector_viewport(ui.ctx());
    }
}
