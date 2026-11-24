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
                    ui.label("Load a valid Polaris (384/833 B), Vega 10 (512 B), Vega 20 (1534/1924 B), RDNA 1 (1674/1924 B), RDNA 2 (2470/2690 B), or RDNA 3 (5424 B) PPTable first."); return;
                };
                let mut copy_lines = vec![format!("# Verified Silicon Struct: {} ({} bytes)", arch, b.len())];
                if let (Some(rom), Some(ppt_off)) = (self.vbios_buffer.as_deref(), self.vbios_ppt_offset) {
                    if let Some(vi) = crate::vbios::inspect_vbios_info(rom, ppt_off) {
                        let psp_tag = vi.psp_dir_offset.map(|o| format!("0x{:X} ({} ent)", o, vi.psp_entry_count)).unwrap_or_else(|| "None".to_string());
                        let csum_tag = if !vi.has_legacy_opt_rom { "SPI".to_string() } else if vi.checksum_sum == 0 { format!("VALID (0x{:02X})", vi.checksum_byte) } else { format!("INVALID (0x{:02X}, sum=0x{:02X})", vi.checksum_byte, vi.checksum_sum) };
                        let vb_line = format!("VBIOS: {} [{:04X}:{:04X}] | PPTable @ 0x{:X} | $PSP={} | Csum={}", vi.device_name, vi.vendor_id, vi.device_id, ppt_off, psp_tag, csum_tag);
                        ui.monospace(&vb_line);
                        copy_lines.push(vb_line);
                    }
                }
                for (sec, field, off, len, val) in &entries {
                    let hex: String = b[*off..(*off + *len).min(b.len())].iter().take(8).map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ");
                    let voff = self.vbios_ppt_offset.map(|vo| format!("0x{:05X}", vo + *off)).unwrap_or_else(|| "-".into());
                    copy_lines.push(format!("0x{:03X} | {:<7} | {:<10} | {:<32} | {:<23} | {}", off, voff, sec, field, hex, val));
                }
                let copy_blob = copy_lines.join("\n");
                ui.horizontal(|ui| {
                    ui.strong(format!("Verified Silicon Struct: {} ({} bytes)", arch, b.len()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            self.show_struct_inspector_window = false;
                        }
                        if ui.button("Copy Table").clicked() {
                            ui.ctx().copy_text(copy_blob);
                        }
                    });
                });
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
