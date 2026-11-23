use eframe::egui;
use crate::PowerPlayApp;

impl PowerPlayApp {
    pub fn draw_features_tab(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let loaded = !self.raw_buffer.is_empty();
        let gen = ctx.data(|d| d.get_temp::<u64>(egui::Id::new("ppt_baseline_gen")).unwrap_or(0));

        if !arch.is_gcn_subtable() {
            ui.vertical_centered(|ui| {
                let btn = egui::Button::new("PPTable Features").min_size(egui::vec2(180.0, 28.0));
                if ui.add(btn).clicked() {
                    self.show_smu_features_window = !self.show_smu_features_window;
                }
            });
            ui.add_space(6.0);
        }

        let card_title = if arch.is_gcn_subtable() {
            "OverDrive & Platform Capabilities (0x013)"
        } else {
            "OverDrive Features"
        };

        let ctx = ctx.clone();
        crate::card_layout::draw_card(ui, card_title, |ui| {
            if !loaded {
                ui.disable();
            }
                ui.columns(2, |cols| {
                    cols[0].spacing_mut().item_spacing.y = 4.0;
                    cols[1].spacing_mut().item_spacing.y = 4.0;
                    let left_items: [(&mut bool, &str, &str, bool); 8] = [
                        (&mut self.gpu_clock_limit, "GPU Clock Limit", "od_feat_0", true),
                        (&mut self.gpu_clock_curve, "GPU Clock Curve", "od_feat_1", !arch.is_gcn_subtable()),
                        (&mut self.memory_clock_limit, "Memory Clock Limit", "od_feat_2", true),
                        (&mut self.zero_rpm, "Zero RPM", "od_feat_3", arch != crate::ppt_arch::PptArch::Polaris),
                        (&mut self.min_fan_speed, "Minimum Fan Speed", "od_feat_4", true),
                        (&mut self.fan_target_temp_limit, "Fan Target Temperature Limit", "od_feat_5", true),
                        (&mut self.operating_temp_limit, "Operating Temperature Limit", "od_feat_6", true),
                        (&mut self.fan_acoustic_rpm, "Fan Acoustic RPM", "od_feat_7", !arch.is_gcn_subtable()),
                    ];
                    for (flag, lbl, tag, supported) in left_items {
                        if !supported {
                            continue;
                        }
                        let base_id = egui::Id::new(("od_feat_base", gen, tag));
                        let base_val = ctx.data_mut(|d| {
                            if let Some(v) = d.get_temp::<bool>(base_id) {
                                v
                            } else {
                                d.insert_temp(base_id, *flag);
                                *flag
                            }
                        });
                        cols[0].horizontal(|ui| {
                            let resp = crate::ui_theme::green_checkbox(ui, flag, lbl);
                            if resp.changed() {
                                ctx.data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                            }
                            if *flag != base_val {
                                ui.colored_label(egui::Color32::from_rgb(240, 70, 90), "●");
                            }
                            if resp.hovered() || resp.clicked() {
                                let state = if *flag { "Enabled" } else { "Disabled" };
                                let def_str = if base_val { "Enabled" } else { "Disabled" };
                                let info = if *flag != base_val {
                                    format!("{} › {} ({}) · {} (Factory: {})", card_title, lbl, tag, state, def_str)
                                } else {
                                    format!("{} › {} ({}) · {}", card_title, lbl, tag, state)
                                };
                                ctx.data_mut(|d| d.insert_temp(egui::Id::new("hovered_field_banner"), info.clone()));
                                resp.on_hover_text(info);
                            }
                        });
                    }
                    let right_items: [(&mut bool, &str, &str, bool); 8] = [
                        (&mut self.fan_curve, "Fan Curve", "od_feat_8", !arch.is_gcn_subtable()),
                        (&mut self.power_limit_feat, "Power Limit", "od_feat_9", true),
                        (&mut self.timing_tuning, "Timing Tuning", "od_feat_10", !arch.is_gcn_subtable()),
                        (&mut self.power_mode, "Power Mode", "od_feat_11", arch.has_multi_power_modes()),
                        (&mut self.auto_undervolt_gpu, "Auto Undervolt GPU", "od_feat_12", !arch.is_gcn_subtable()),
                        (&mut self.auto_overclock_gpu, "Auto Overclock GPU", "od_feat_13", !arch.is_gcn_subtable()),
                        (&mut self.auto_overclock_mem, "Auto Overclock Memory", "od_feat_14", !arch.is_gcn_subtable()),
                        (&mut self.auto_fan_acoustic_limit, "Auto Fan Acoustic Limit", "od_feat_15", !arch.is_gcn_subtable()),
                    ];
                    for (flag, lbl, tag, supported) in right_items {
                        if !supported {
                            continue;
                        }
                        let base_id = egui::Id::new(("od_feat_base", gen, tag));
                        let base_val = ctx.data_mut(|d| {
                            if let Some(v) = d.get_temp::<bool>(base_id) {
                                v
                            } else {
                                d.insert_temp(base_id, *flag);
                                *flag
                            }
                        });
                        cols[1].horizontal(|ui| {
                            let resp = crate::ui_theme::green_checkbox(ui, flag, lbl);
                            if resp.changed() {
                                ctx.data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                            }
                            if *flag != base_val {
                                ui.colored_label(egui::Color32::from_rgb(240, 70, 90), "●");
                            }
                            if resp.hovered() || resp.clicked() {
                                let state = if *flag { "Enabled" } else { "Disabled" };
                                let def_str = if base_val { "Enabled" } else { "Disabled" };
                                let info = if *flag != base_val {
                                    format!("{} › {} ({}) · {} (Factory: {})", card_title, lbl, tag, state, def_str)
                                } else {
                                    format!("{} › {} ({}) · {}", card_title, lbl, tag, state)
                                };
                                ctx.data_mut(|d| d.insert_temp(egui::Id::new("hovered_field_banner"), info.clone()));
                                resp.on_hover_text(info);
                            }
                        });
                    }
                });
        });

        if self.show_smu_features_window {
            let vp_id = egui::ViewportId::from_hash_of("smu_ppt_features_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("PPTable Features (FeaturesToRun)")
                .with_inner_size([420.0, 500.0])
                .with_resizable(true);
            ui.ctx().show_viewport_immediate(vp_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_smu_features_window = false;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    if self.raw_buffer.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("Load a PPTable first to inspect SMU features.");
                        });
                        return;
                    }
                    let mut live_mask = 0u64;
                    for (bit, &en) in self.smu_features.iter().enumerate() {
                        if en { live_mask |= 1u64 << bit; }
                    }
                    let arch = crate::smu_features::detect_silicon_arch(&self.raw_buffer);
                    let active_cnt = live_mask.count_ones();
                    ui.horizontal(|ui| {
                        ui.strong("FeaturesToRun:");
                        ui.monospace(format!("0x{:016X}", live_mask));
                        ui.separator();
                        ui.colored_label(
                            egui::Color32::from_rgb(72, 235, 148),
                            egui::RichText::new(format!("{} [{}] ({} active)", arch.label(), arch.mask_tag(), active_cnt)).strong(),
                        );
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("smu_vp_scroll")
                        .auto_shrink([false, false])
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        for def in &crate::smu_features::SMU_FEATURE_DEFS {
                            let (tag, supported) = crate::smu_features::arch_badge(def, arch);
                            ui.horizontal(|ui| {
                                crate::ui_theme::green_checkbox(ui, &mut self.smu_features[def.bit], format!("{:02}: {}", def.bit, def.name));
                                let badge_col = if supported {
                                    egui::Color32::from_rgb(72, 235, 148)
                                } else {
                                    egui::Color32::from_rgb(130, 145, 170)
                                };
                                ui.colored_label(
                                    badge_col,
                                    egui::RichText::new(format!("[smu_feat_{:02} | {}]", def.bit, tag)).monospace().small(),
                                );
                            });
                        }
                    });
                });
            });
        }
            crate::card_layout::draw_card(ui, "VBIOS & Table Header", |ui| {
                let info = self
                    .vbios_buffer
                    .as_ref()
                    .and_then(|rom| crate::vbios::inspect_vbios_info(rom, self.vbios_ppt_offset.unwrap_or(0)));

                egui::Grid::new("vbios_header_grid")
                    .num_columns(2)
                    .spacing([14.0, 6.0])
                    .show(ui, |ui| {
                        let (src_part, vbios_part) = if let Some((left, right)) = self.status_msg.split_once(" | VBIOS ROM:") {
                            (left.trim(), Some(right.trim()))
                        } else if let Some((left, right)) = self.status_msg.split_once(" | ") {
                            (left.trim(), Some(right.trim()))
                        } else {
                            (self.status_msg.as_str(), None)
                        };
                        ui.label("Loaded Source:");
                        ui.colored_label(
                            egui::Color32::from_rgb(165, 195, 235),
                            egui::RichText::new(src_part).monospace(),
                        );
                        ui.end_row();

                        if let Some(vb_txt) = vbios_part {
                            ui.label("");
                            ui.colored_label(
                                egui::Color32::from_rgb(165, 195, 235),
                                egui::RichText::new(vb_txt).monospace(),
                            );
                            ui.end_row();
                        }

                        ui.label("Silicon Family:");
                        let did = info.as_ref().map(|vi| vi.device_id).unwrap_or(0);
                        if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer) {
                            ui.colored_label(
                                egui::Color32::from_rgb(95, 200, 145),
                                egui::RichText::new(t3.detect_chip(did).display_name()).strong(),
                            );
                        } else if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(&self.raw_buffer) {
                            ui.colored_label(
                                egui::Color32::from_rgb(95, 200, 145),
                                egui::RichText::new(t1.detect_chip(did).display_name()).strong(),
                            );
                        } else if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Polaris {
                            let pchip = crate::pptable_polaris::PolarisChip::from_did(did);
                            ui.colored_label(
                                egui::Color32::from_rgb(95, 200, 145),
                                egui::RichText::new(pchip.display_name()).strong(),
                            );
                        } else if matches!(crate::ppt_arch::PptArch::detect(&self.raw_buffer), crate::ppt_arch::PptArch::Vega10 | crate::ppt_arch::PptArch::Vega20) {
                            let frev_v = self.raw_buffer.get(2).copied().unwrap_or(0);
                            let vchip = crate::pptable_vega::VegaChip::from_did_and_rev(did, frev_v);
                            ui.colored_label(
                                egui::Color32::from_rgb(95, 200, 145),
                                egui::RichText::new(vchip.display_name()).strong(),
                            );
                        } else if let Some(t) = crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer) {
                            let chip = t.detect_chip(&self.raw_buffer);
                            ui.colored_label(
                                egui::Color32::from_rgb(95, 200, 145),
                                egui::RichText::new(chip.display_name()).strong(),
                            );
                        } else {
                            ui.weak(if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Rdna1 { "RDNA 1 (Navi 10 / 12 / 14)" } else { "Not detected" });
                        }
                        ui.end_row();

                        ui.label("VBIOS ROM:");
                        if let Some(ref vi) = info {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.monospace(format!(
                                    "{} [{:04X}:{:04X}]",
                                    vi.device_name,
                                    vi.vendor_id,
                                    vi.device_id,
                                ));
                                ui.monospace(format!(
                                    "{} B (Legacy {} B, PPTable @ 0x{:X}, {} B)",
                                    vi.total_size,
                                    vi.legacy_size,
                                    vi.ppt_offset,
                                    vi.ppt_size
                                ));
                            });
                        } else {
                            ui.weak("Standalone PPTable / Registry buffer (no full ROM)");
                        }
                        ui.end_row();

                        if let Some(ref vi) = info {
                            ui.label("AIB / Subsys:");
                            ui.monospace(format!(
                                "{} [{:04X}:{:04X}] | {}",
                                vi.vendor_name, vi.subvendor_id, vi.subdevice_id, vi.pci_vendor_name
                            ));
                            ui.end_row();

                            ui.label("BIOS Version:");
                            ui.monospace(format!(
                                "{}{}",
                                vi.bios_version,
                                vi.bios_ver_offset
                                    .map(|o| format!(" (@ 0x{:X})", o))
                                    .unwrap_or_default()
                            ));
                            ui.end_row();

                            ui.label("$PSP Security:");
                            if let Some(psp_off) = vi.psp_dir_offset {
                                let signed_info = match (vi.psp_signed_size, vi.rsa_sig_offset, vi.rsa_sig_size) {
                                    (Some(ssz), Some(roff), Some(rsz)) => {
                                        format!("Signed ({} B) | RSA Sig @ 0x{:X} ({} B)", ssz, roff, rsz)
                                    }
                                    (Some(ssz), _, _) => format!("Signed ({} B)", ssz),
                                    _ => "Present (unsigned or partial)".to_string(),
                                };
                                ui.colored_label(
                                    egui::Color32::from_rgb(240, 180, 60),
                                    egui::RichText::new(format!("ACTIVE @ 0x{:X}: {}", psp_off, signed_info)).monospace(),
                                );
                            } else {
                                ui.colored_label(
                                    egui::Color32::from_rgb(80, 210, 120),
                                    "No $PSP directory (unsigned / legacy)",
                                );
                            }
                            ui.end_row();

                            ui.label("AtomBIOS Chain:");
                            if let (Some(ah), Some(md)) = (vi.atom_header_offset, vi.master_data_offset) {
                                ui.monospace(format!("ATOM @ 0x{:X} -> MasterData @ 0x{:X}", ah, md));
                            } else {
                                ui.weak("N/A (SPI / Raw Table)");
                            }
                            ui.end_row();

                            ui.label("$PSP Dir:");
                            if let Some(psp_off) = vi.psp_dir_offset {
                                let pk = vi
                                    .psp_pubkey_offset
                                    .map(|o| format!(" | PubKey @ 0x{:X}", o))
                                    .unwrap_or_default();
                                ui.monospace(format!("Offset 0x{:X} | {} entries{}", psp_off, vi.psp_entry_count, pk));
                            } else {
                                ui.weak("None");
                            }
                            ui.end_row();

                            ui.label("ROM Checksum:");
                            if !vi.has_legacy_opt_rom {
                                ui.colored_label(
                                    egui::Color32::from_rgb(138, 162, 198),
                                    egui::RichText::new("N/A (SPI Flash)").monospace(),
                                );
                            } else if vi.checksum_sum == 0 {
                                let txt = format!("VALID (0x{:02X})", vi.checksum_byte);
                                ui.colored_label(egui::Color32::from_rgb(80, 210, 120), egui::RichText::new(txt).monospace());
                            } else {
                                ui.horizontal(|ui| {
                                    let txt = format!("INVALID (0x{:02X}, sum 0x{:02X})", vi.checksum_byte, vi.checksum_sum);
                                    ui.colored_label(egui::Color32::from_rgb(245, 75, 95), egui::RichText::new(txt).monospace());
                                    if ui.small_button("Recalc").clicked() {
                                        if let Some(rom) = self.vbios_buffer.as_mut() {
                                            crate::vbios::recalc_vbios_checksum(rom);
                                            self.status_msg = "Recalculated VBIOS 8-bit Option ROM checksum (byte[0x21]).".into();
                                        }
                                    }
                                });
                            }
                            ui.end_row();
                        }

                        ui.label("Header Verification:");
                        let hdr_info = if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer) {
                            Some(("V4_12", { t3.header.structure_size }, { t3.header.format_revision }, { t3.header.content_revision }, { t3.header.table_revision }))
                        } else { crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).map(|t| ("V4_9", { t.header.structure_size }, { t.header.format_revision }, { t.header.content_revision }, { t.table_revision })) };
                        if let Some((ver, sz, fmt_rev, cnt_rev, tbl_rev)) = hdr_info {
                            let ok = sz as usize == self.raw_buffer.len();
                            let msg = format!(
                                "{} ({}) | size={} B | rev=0x{:02X}.{:02X}.{:02X}",
                                if ok { "VALID" } else { "SIZE MISMATCH" },
                                ver,
                                sz,
                                fmt_rev,
                                cnt_rev,
                                tbl_rev
                            );
                            if ok {
                                ui.colored_label(egui::Color32::from_rgb(80, 210, 120), egui::RichText::new(msg).monospace());
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(245, 75, 95), egui::RichText::new(msg).monospace());
                            }
                        } else {
                            ui.weak(match crate::ppt_arch::PptArch::detect(&self.raw_buffer) {
                            crate::ppt_arch::PptArch::Polaris => "Valid Polaris (SMU 8.0, rev 0x07) table loaded",
                            crate::ppt_arch::PptArch::Vega10 => "Valid Vega 10 (SMU 9.0, rev 0x08) table loaded",
                            crate::ppt_arch::PptArch::Vega20 => "Valid Vega 20 (SMU 11.0, rev 0x0B) table loaded",
                            crate::ppt_arch::PptArch::Rdna1 => "Valid RDNA 1 (SMU 11.0, V4_5) table loaded",
                            _ => "No valid Polaris, Vega, or RDNA table loaded",
                        });
                        }
                        ui.end_row();
                    });
            });
        }
    }
