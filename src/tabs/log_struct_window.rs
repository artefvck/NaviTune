use crate::tabs::native_offset::collect_native_struct_entries;
use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub(crate) fn render_struct_inspector_viewport(&mut self, ctx: &egui::Context) {
        if !self.show_struct_inspector_window { return; }
        let vp_id = egui::ViewportId::from_hash_of("struct_binary_verifier_viewport");
        let builder = egui::ViewportBuilder::default().with_title("Struct & Binary Structure Verifier").with_inner_size([720.0, 520.0]).with_resizable(true);
        ctx.show_viewport_immediate(vp_id, builder, |ctx, _| {
            if ctx.input(|i| i.viewport().close_requested()) { self.show_struct_inspector_window = false; }
            egui::CentralPanel::default().show(ctx, |ui| {
                let b = &self.raw_buffer;
                let Some((arch, entries)) = collect_native_struct_entries(b) else {
                    ui.label("Load a valid Polaris (384 B), Vega 10 (512 B), Vega 20 / RDNA 1 (1924 B), RDNA 2 (2470 B), or RDNA 3 (5424 B) PPTable first."); return;
                };
                ui.strong(format!("Verified Silicon Struct: {} ({} bytes)", arch, b.len()));
                ui.add_space(4.0);
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    egui::Grid::new("struct_verifier_grid").striped(true).spacing([10.0, 5.0]).show(ui, |ui| {
                        ui.strong("Section"); ui.strong("Struct Field"); ui.strong("Table Off"); ui.strong("VBIOS Off"); ui.strong("Raw Bytes"); ui.strong("Decoded Value"); ui.end_row();
                        for (sec, field, off, len, val) in entries {
                            let hex: String = b[off..(off + len).min(b.len())].iter().take(8).map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ");
                            let voff = self.vbios_ppt_offset.map(|vo| format!("0x{:05X}", vo + off)).unwrap_or_else(|| "-".into());
                            ui.label(sec); ui.monospace(field); ui.monospace(format!("0x{:03X}", off)); ui.monospace(voff); ui.monospace(hex); ui.monospace(val); ui.end_row();
                        }
                    });
                });
            });
        });
    }
}
