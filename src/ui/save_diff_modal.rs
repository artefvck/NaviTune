use crate::vbios;
use crate::PowerPlayApp;
use eframe::egui;

pub struct ByteDiffEntry {
    pub ppt_offset: usize,
    pub vbios_offset: Option<usize>,
    pub old_byte: u8,
    pub new_byte: u8,
    pub old_u16: u16,
    pub new_u16: u16,
}

pub fn compute_ppt_diff(
    baseline: &[u8],
    current: &[u8],
    vbios_ppt_offset: Option<usize>,
) -> Vec<ByteDiffEntry> {
    let len = baseline.len().min(current.len());
    let mut diffs = Vec::new();
    for i in 0..len {
        if baseline[i] != current[i] {
            let old_u16 = if i + 1 < len {
                u16::from_le_bytes([baseline[i], baseline[i + 1]])
            } else {
                baseline[i] as u16
            };
            let new_u16 = if i + 1 < len {
                u16::from_le_bytes([current[i], current[i + 1]])
            } else {
                current[i] as u16
            };
            diffs.push(ByteDiffEntry {
                ppt_offset: i,
                vbios_offset: vbios_ppt_offset.map(|base| base + i),
                old_byte: baseline[i],
                new_byte: current[i],
                old_u16,
                new_u16,
            });
        }
    }
    diffs
}

pub fn preview_patched_checksum(
    vbios_buf: Option<&[u8]>,
    ppt_offset: Option<usize>,
    new_ppt: &[u8],
) -> Option<(u8, u8)> {
    let orig = vbios_buf?;
    let off = ppt_offset?;
    let old_csum = orig.get(0x21).copied()?;
    let patched = vbios::patch_ppt_into_rom(orig, off, new_ppt).ok()?;
    let new_csum = patched.get(0x21).copied()?;
    Some((old_csum, new_csum))
}

impl PowerPlayApp {
    pub(crate) fn draw_save_diff_modal(&mut self, ctx: &egui::Context) {
        if !self.show_save_diff_modal {
            return;
        }
        self.sync_fields_to_buffer();
        let diffs = compute_ppt_diff(
            &self.baseline_buffer,
            &self.raw_buffer,
            self.vbios_ppt_offset,
        );
        let cache_key = (self.baseline_gen, self.vbios_ppt_offset, self.raw_buffer.clone());
        let cache_id = egui::Id::new("save_diff_csum_cache");
        let csum_preview = ctx.data_mut(|d| {
            if let Some((prev_key, prev_val)) = d.get_temp::<((u64, Option<usize>, Vec<u8>), Option<(u8, u8)>)>(cache_id) {
                if prev_key == cache_key {
                    return prev_val;
                }
            }
            let val = preview_patched_checksum(
                self.vbios_buffer.as_deref(),
                self.vbios_ppt_offset,
                &self.raw_buffer,
            );
            d.insert_temp(cache_id, (cache_key, val));
            val
        });

        let mut open = self.show_save_diff_modal;
        let mut close_modal = false;

        egui::Window::new("Pre-Save Binary Diff & Checksum Audit")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([560.0, 380.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(format!("Modified Bytes: {}", diffs.len()));
                    if let Some((old_c, new_c)) = csum_preview {
                        ui.separator();
                        ui.colored_label(
                            egui::Color32::from_rgb(80, 210, 120),
                            format!("VBIOS Checksum (0x21): 0x{:02X} -> 0x{:02X}", old_c, new_c),
                        );
                    }
                });
                ui.add_space(6.0);

                if diffs.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(175, 190, 210),
                        "No binary changes detected relative to the loaded baseline.",
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Grid::new("pre_save_diff_grid")
                                .num_columns(5)
                                .striped(true)
                                .spacing([14.0, 4.0])
                                .show(ui, |ui| {
                                    ui.strong("PPTable Off");
                                    ui.strong("VBIOS Off");
                                    ui.strong("Byte (Old -> New)");
                                    ui.strong("u16 LE Old");
                                    ui.strong("u16 LE New");
                                    ui.end_row();
                                    for d in &diffs {
                                        ui.monospace(format!("0x{:04X}", d.ppt_offset));
                                        ui.monospace(
                                            d.vbios_offset
                                                .map(|o| format!("0x{:06X}", o))
                                                .unwrap_or_else(|| "—".to_string()),
                                        );
                                        ui.monospace(format!("0x{:02X} -> 0x{:02X}", d.old_byte, d.new_byte));
                                        ui.monospace(format!("{}", d.old_u16));
                                        ui.colored_label(
                                            egui::Color32::from_rgb(245, 185, 65),
                                            format!("{}", d.new_u16),
                                        );
                                        ui.end_row();
                                    }
                                });
                        });
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Export PPTable (.bin)").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name("pp_table_patched.bin")
                            .save_file()
                        {
                            if let Err(e) = std::fs::write(&path, &self.raw_buffer) {
                                self.status_msg = format!("Save error: {}", e);
                            } else {
                                self.baseline_buffer = self.raw_buffer.clone();
                                self.status_msg = format!("Saved {} B -> {}", self.raw_buffer.len(), path.display());
                                close_modal = true;
                            }
                        }
                    }
                    if let (Some(orig_rom), Some(off)) = (self.vbios_buffer.clone(), self.vbios_ppt_offset) {
                        if ui.button("Patch VBIOS ROM (.rom)").clicked() {
                            match vbios::patch_ppt_into_rom(&orig_rom, off, &self.raw_buffer) {
                                Ok(patched_rom) => {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_file_name("vbios_patched.rom")
                                        .save_file()
                                    {
                                        if let Err(e) = std::fs::write(&path, &patched_rom) {
                                            self.status_msg = format!("ROM write error: {}", e);
                                        } else {
                                            self.vbios_buffer = Some(patched_rom);
                                            self.baseline_buffer = self.raw_buffer.clone();
                                            self.status_msg = format!("Patched VBIOS ROM ({} B) -> {}", orig_rom.len(), path.display());
                                            close_modal = true;
                                        }
                                    }
                                }
                                Err(e) => self.status_msg = format!("Patch error: {}", e),
                            }
                        }
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if close_modal {
            open = false;
        }
        self.show_save_diff_modal = open;
    }
}

/// Collects up to `max_entries` changed byte offsets `(offset, old_byte, new_byte)` safely across
/// buffers of differing lengths without panicking or allocating unbounded diff rows in the UI.
pub(crate) type BoundedByteDelta = (usize, Option<u8>, Option<u8>);

pub(crate) fn collect_bounded_byte_diffs(
    baseline: &[u8],
    current: &[u8],
    max_entries: usize,
) -> (Vec<BoundedByteDelta>, usize) {
    let max_len = baseline.len().max(current.len());
    let mut total_diffs = 0usize;
    let mut preview = Vec::with_capacity(max_entries.min(256));
    for idx in 0..max_len {
        let old_b = baseline.get(idx).copied();
        let new_b = current.get(idx).copied();
        if old_b != new_b {
            if preview.len() < max_entries {
                preview.push((idx, old_b, new_b));
            }
            total_diffs += 1;
        }
    }
    (preview, total_diffs)
}

#[cfg(test)]
mod save_diff_modal_bounds_tests {
    use super::collect_bounded_byte_diffs;

    #[test]
    fn bounded_diff_handles_mismatched_and_large_buffers_safely() {
        let base = vec![0xAAu8; 5424];
        let mut cur = base.clone();
        cur[0] = 0xBB;
        cur[100] = 0xCC;
        cur.push(0xDD);

        let (preview, total) = collect_bounded_byte_diffs(&base, &cur, 2);
        assert_eq!(total, 3);
        assert_eq!(preview.len(), 2);
        assert_eq!(preview[0], (0, Some(0xAA), Some(0xBB)));
        assert_eq!(preview[1], (100, Some(0xAA), Some(0xCC)));
    }
}
