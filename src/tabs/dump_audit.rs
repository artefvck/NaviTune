use crate::PowerPlayApp;

impl PowerPlayApp {
    pub(crate) fn format_inspect_report(&self, source_label: &str) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let (frev, crev, trev) = (
            self.raw_buffer.get(2).copied().unwrap_or(0),
            self.raw_buffer.get(3).copied().unwrap_or(0),
            self.raw_buffer.get(4).copied().unwrap_or(0),
        );
        let _ = writeln!(out, "File         : {}", source_label);
        if let Some(off) = self.vbios_ppt_offset {
            if let Some(vi) = self.vbios_buffer.as_deref().and_then(|vb| crate::vbios::inspect_vbios_info(vb, off)) {
                let psp_tag = vi.psp_dir_offset.map(|p| format!("0x{:X} ({} ent)", p, vi.psp_entry_count)).unwrap_or_else(|| "None".to_string());
                let csum_tag = if !vi.has_legacy_opt_rom { "SPI".to_string() } else if vi.checksum_sum == 0 { format!("VALID (0x{:02X})", vi.checksum_byte) } else { format!("INVALID (0x{:02X}, sum=0x{:02X})", vi.checksum_byte, vi.checksum_sum) };
                let _ = writeln!(out, "VBIOS Info   : Offset 0x{:06X} | DID 0x{:04X} | SVID {:04X}:{:04X} | {} | $PSP={} | Csum={}", off, vi.device_id, vi.subvendor_id, vi.subdevice_id, vi.bios_version, psp_tag, csum_tag);
                let mut hdr_parts = Vec::new();
                if let Some(ref b) = vi.board_banner { hdr_parts.push(format!("Board=\"{}\"", b)); }
                if let Some(ref d) = vi.build_date { hdr_parts.push(format!("Date={}", d)); }
                if let Some((maj, min)) = vi.atom_rev { hdr_parts.push(format!("ATOM=v{}.{}", maj, min)); }
                if vi.pcir_image_count > 0 {
                    hdr_parts.push(format!("ROM_Images={} ({})", vi.pcir_image_count, if vi.has_uefi_gop { "Legacy + UEFI GOP" } else { "Legacy BIOS" }));
                }
                if !hdr_parts.is_empty() {
                    let _ = writeln!(out, "ROM Headers  : {}", hdr_parts.join(" | "));
                }
            } else {
                let _ = writeln!(out, "VBIOS Offset : 0x{:06X}", off);
            }
        }
        let _ = writeln!(out, "Architecture : {:?} (rev 0x{:02X}.{:02X}.{:02X}, {} B)", arch, frev, crev, trev, self.raw_buffer.len());
        let _ = writeln!(out, "Status       : {}", self.status_msg);
        let _ = writeln!(out, "GFX Clocks   : {} .. {} MHz (OD: {} .. {} MHz)", self.freq_gfx[0], self.freq_gfx[1], self.oc_gfx_clock[0], self.oc_gfx_clock[1]);
        let _ = writeln!(out, "SOC / FCLK   : SOC {} .. {} MHz | FCLK {} .. {} MHz (Boost: {} MHz)", self.freq_soc[0], self.freq_soc[1], self.freq_fclk[0], self.freq_fclk[1], self.fclk_boost_freq);
        let uclk_peak = self.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
        let _ = writeln!(out, "Memory/UCLK  : DPM0={} MHz .. Peak={} MHz (OD: {} .. {} MHz)", self.mem_dpm_freq[0][0], uclk_peak, self.oc_mem_clock[0], self.oc_mem_clock[1]);
        let _ = writeln!(out, "GFX / SOC mV : GFX {} .. {} mV (ULV {} mV) | SOC {} .. {} mV", self.volt_gfx[0], self.volt_gfx[1], self.ulv_gfx[0], self.volt_soc[0], self.volt_soc[1]);
        let _ = writeln!(out, "Power & TDC  : {} W (OD: {} .. {}%) | TDC GFX={} A, Soc={} A", self.power_limit_w, self.oc_power_limit[0], self.oc_power_limit[1], self.tdc_gfx, self.tdc_soc);
        let _ = writeln!(out, "Fan Targets  : PWM Min={}%, Max={} RPM, Stop/Start={}/{} C, Target={} C, ZeroRPM={}", self.fan_pwm_min, self.fan_max_rpm, self.fan_stop_temp, self.fan_start_temp, self.fan_target_temp, self.fan_zero_rpm_enable);
        let warns = self.validate_tunables();
        if !warns.is_empty() {
            let _ = writeln!(out, "Warnings     : {}", warns.join("; "));
        }
        let _ = writeln!(out, "\n--- OverDrive Feature Flags ---");
        let _ = writeln!(out, "gpu_clock_limit={} gpu_clock_curve={} memory_clock_limit={} zero_rpm={} min_fan_speed={} fan_target_temp={} operating_temp={} fan_acoustic={} fan_curve={} power_limit={} timing_tuning={} power_mode={}", self.gpu_clock_limit, self.gpu_clock_curve, self.memory_clock_limit, self.zero_rpm, self.min_fan_speed, self.fan_target_temp_limit, self.operating_temp_limit, self.fan_acoustic_rpm, self.fan_curve, self.power_limit_feat, self.timing_tuning, self.power_mode);
        let _ = writeln!(out, "auto_uv_gpu={} auto_oc_gpu={} auto_oc_mem={} auto_fan_acoustic={}", self.auto_undervolt_gpu, self.auto_overclock_gpu, self.auto_overclock_mem, self.auto_fan_acoustic_limit);
        let mut smu_mask = 0u64;
        let mut bits = Vec::new();
        for (bit, &en) in self.smu_features.iter().enumerate() {
            if en { smu_mask |= 1u64 << bit; bits.push(bit.to_string()); }
        }
        let _ = writeln!(out, "\n--- SMU Features ---\nMask         : 0x{:016X} ({} enabled)\nEnabled Bits : {}", smu_mask, bits.len(), if bits.is_empty() { "none".to_string() } else { bits.join(", ") });
        let _ = writeln!(out, "\n--- OverDrive Limits & Controls ---\noc_gfx_clock : {:?} MHz | oc_mem_clock : {:?} MHz | oc_power_limit : {:?} %\noc_fan_rpm   : {:?} RPM | oc_acoustic  : {:?} RPM\nControls     : mem_timing={} power_mode={} zero_rpm={}", self.oc_gfx_clock, self.oc_mem_clock, self.oc_power_limit, self.oc_fan_rpm, self.oc_fan_acoustic_rpm, self.oc_mem_timing_ctrl, self.oc_power_mode_ctrl, self.oc_zero_rpm_ctrl);
        let _ = writeln!(out, "\n--- Power, TDC, Thermal & Power Modes ---\npower_limit_w: {} W | power_limit_ac : {:?} W\ntdc_gfx      : {} A | tdc_soc        : {} A\ntemp_limits  : {:?} C\nPM Presets   : Quiet={:?} Balanced={:?} Turbo={:?} Rage={:?}\nAC Acoustic  : Quiet={:?} Balanced={:?} Turbo={:?} Rage={:?}", self.power_limit_w, self.power_limit_ac, self.tdc_gfx, self.tdc_soc, self.temp_limits, self.pm_quiet, self.pm_balanced, self.pm_turbo, self.pm_rage, self.ac_quiet, self.ac_balanced, self.ac_turbo, self.ac_rage);
        let _ = writeln!(out, "\n--- Voltages, ULV, TVmin, AVFS & DCBTC ---\nvolt_gfx     : {:?} mV | volt_soc : {:?} mV\nulv_gfx      : {:?} mV | ulv_soc  : {:?} mV\ntvmin_gfx    : {:?} | tvmin_soc : {:?}\nvmin_lh_gfx  : {:?} mV | vmin_lh_soc : {:?} mV\nStatic V GFX : a={:.6} b={:.6} c={:.6} | Off: a={:.6} b={:.6} c={:.6}\nStatic V SOC : a={:.6} b={:.6} c={:.6} | Off: a={:.6} b={:.6} c={:.6}\nDCBTC        : gfx_en={} soc_en={} min={:?} max={:?} gb={:?} dctol={:?}", self.volt_gfx, self.volt_soc, self.ulv_gfx, self.ulv_soc, self.tvmin_gfx, self.tvmin_soc, self.vmin_lh_gfx, self.vmin_lh_soc, self.static_volt_a[0], self.static_volt_b[0], self.static_volt_c[0], self.static_off_a[0], self.static_off_b[0], self.static_off_c[0], self.static_volt_a[1], self.static_volt_b[1], self.static_volt_c[1], self.static_off_a[1], self.static_off_b[1], self.static_off_c[1], self.dcbtc_gfx_en, self.dcbtc_soc_en, self.dcbtc_min, self.dcbtc_max, self.dcbtc_gb, self.dctol);
        let _ = writeln!(out, "\n--- Clock Domains & PCIe ---\nGFX          : {:?} MHz | SOC  : {:?} MHz | FCLK : {:?} MHz (boost {} MHz)\nVCLK         : {:?} MHz | DCLK : {:?} MHz | LCLK : {:?} MHz\nDCEFCLK      : {:?} MHz | DISP : {:?} MHz | PIX  : {:?} MHz | PHY : {:?} MHz | DTB : {:?} MHz\nPCIe         : Gen {:?} | Lanes {:?}", self.freq_gfx, self.freq_soc, self.freq_fclk, self.fclk_boost_freq, self.freq_vclk, self.freq_dclk, self.lclk_mhz, self.freq_dcefclk, self.freq_dispclk, self.freq_pixclk, self.freq_phyclk, self.freq_dtbclk, self.pcie_gen_speed, self.pcie_lane_count);
        let _ = writeln!(out, "\n--- Memory / UCLK DPM States ---");
        for i in 0..4 {
            let _ = writeln!(out, "DPM{}         : UCLK={} MHz (div={}) | VDDCI={} mV | MVDD={} mV", i, self.mem_dpm_freq[i][0], self.mem_dpm_freq[i][1], self.mem_dpm_volt[i][0], self.mem_dpm_volt[i][1]);
        }
        let _ = writeln!(out, "\n--- Fan Controller & 5-Point Curve ---\nPWM Min={}%| Acoustic={} RPM | Throttling={} RPM | Max={} RPM | Mode={} | Hyst={}\nTarget Temp={} C | Target GFX={} MHz | ZeroRPM={} | Stop={} C | Start={} C\nOD Fan Range : Temp={:?} C | Speed={:?}% | Curve Temp={:?} C | Curve PWM={:?}%", self.fan_pwm_min, self.fan_acoustic_limit, self.fan_throttling_rpm, self.fan_max_rpm, self.fan_mode, self.fan_hyst_ctrl, self.fan_target_temp, self.fan_target_gfx_clock, self.fan_zero_rpm_enable, self.fan_stop_temp, self.fan_start_temp, self.od_fan_temp, self.od_fan_speed, self.fan_curve_temp, self.fan_curve_pwm);
        if let Some((struct_arch, entries)) = crate::tabs::native_offset::collect_native_struct_entries(&self.raw_buffer) {
            let _ = writeln!(out, "\n--- Verified Offset Map ({}) ---", struct_arch);
            for (sec, name, offset, len, val) in entries {
                let end = (offset + len).min(self.raw_buffer.len());
                let hex: String = self.raw_buffer.get(offset..end).unwrap_or_default()
                    .iter().take(8).map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ");
                let voff_tag = self.vbios_ppt_offset.map(|vo| format!(" | VBIOS 0x{:06X}", vo + offset)).unwrap_or_default();
                let _ = writeln!(out, "  0x{:03X}{} | {:<10} | {:<32} | {:<23} | {}", offset, voff_tag, sec, name, hex, val);
            }
        }
        out
    }

    pub(crate) fn build_full_debug_dump(&self) -> String {
        let mut smu_mask = 0u64;
        for (bit, &en) in self.smu_features.iter().enumerate() {
            if en { smu_mask |= 1u64 << bit; }
        }
        let _enabled_bits: Vec<String> = (0..64)
            .filter(|&b| self.smu_features[b])
            .map(|b| format!("{:02}", b))
            .collect();
        let gpu_desc = if self.selected_gpu == usize::MAX { "None (File Mode)".to_string() } else { format!("GPU #{}", self.selected_gpu) };

        let mut vars: Vec<(String, String)> = vec![
            ("vid_curve_gfx_a".into(), format!("{:.6}", self.static_volt_a[0])),
            ("vid_curve_gfx_b".into(), format!("{:.6}", self.static_volt_b[0])),
            ("vid_curve_gfx_c".into(), format!("{:.6}", self.static_volt_c[0])),
            ("vid_curve_soc_a".into(), format!("{:.6}", self.static_volt_a[1])),
            ("vid_curve_soc_b".into(), format!("{:.6}", self.static_volt_b[1])),
            ("vid_curve_soc_c".into(), format!("{:.6}", self.static_volt_c[1])),
            ("curve_gfx_a".into(), format!("{:.6}", self.static_off_a[0])),
            ("curve_gfx_b".into(), format!("{:.6}", self.static_off_b[0])),
            ("curve_gfx_c".into(), format!("{:.6}", self.static_off_c[0])),
            ("curve_soc_a".into(), format!("{:.6}", self.static_off_a[1])),
            ("curve_soc_b".into(), format!("{:.6}", self.static_off_b[1])),
            ("curve_soc_c".into(), format!("{:.6}", self.static_off_c[1])),
            ("dctol_gfx".into(), format!("{}", self.dctol[0])),
            ("dctol_soc".into(), format!("{}", self.dctol[1])),
            ("dcbtc_en_gfx".into(), format!("{}", self.dcbtc_gfx_en as u8)),
            ("dcbtc_en_soc".into(), format!("{}", self.dcbtc_soc_en as u8)),
            ("fan_pwm_min".into(), format!("{}", self.fan_pwm_min)),
            ("fan_acoustic_lim".into(), format!("{}", self.fan_acoustic_limit)),
            ("fan_throttling".into(), format!("{}", self.fan_throttling_rpm)),
            ("fan_max_rpm".into(), format!("{}", self.fan_max_rpm)),
            ("fan_target_temp".into(), format!("{}", self.fan_target_temp)),
            ("fan_target_gfx".into(), format!("{}", self.fan_target_gfx_clock)),
            ("fan_zero_rpm".into(), format!("{}", self.fan_zero_rpm_enable as u8)),
            ("fan_stop_temp".into(), format!("{}", self.fan_stop_temp)),
            ("fan_start_temp".into(), format!("{}", self.fan_start_temp)),
            ("freq_gfx_min".into(), format!("{}", self.freq_gfx[0])),
            ("freq_gfx_max".into(), format!("{}", self.freq_gfx[1])),
            ("freq_soc_min".into(), format!("{}", self.freq_soc[0])),
            ("freq_soc_max".into(), format!("{}", self.freq_soc[1])),
            ("freq_fclk_min".into(), format!("{}", self.freq_fclk[0])),
            ("freq_fclk_max".into(), format!("{}", self.freq_fclk[1])),
            ("lclk_min".into(), format!("{}", self.lclk_mhz[0])),
            ("lclk_max".into(), format!("{}", self.lclk_mhz[1])),
            ("pcie_gen_min".into(), format!("{}", self.pcie_gen_speed[0])),
            ("pcie_gen_max".into(), format!("{}", self.pcie_gen_speed[1])),
            ("pcie_lane_min".into(), format!("{}", self.pcie_lane_count[0])),
            ("pcie_lane_max".into(), format!("{}", self.pcie_lane_count[1])),
            ("power_limit_gpu".into(), format!("{}", self.power_limit_w)),
            ("tdc_gfx".into(), format!("{}", self.tdc_gfx)),
            ("tdc_soc".into(), format!("{}", self.tdc_soc)),
            ("vmin_gfx".into(), format!("{}", self.volt_gfx[0])),
            ("vmax_gfx".into(), format!("{}", self.volt_gfx[1])),
            ("vmin_soc".into(), format!("{}", self.volt_soc[0])),
            ("vmax_soc".into(), format!("{}", self.volt_soc[1])),
            ("smu_features_mask".into(), format!("0x{:016X}", smu_mask)),
        ];
        for i in 0..5 {
            vars.push((format!("fan_curve_pwm_{}", i), format!("{}", self.fan_curve_pwm[i])));
            vars.push((format!("fan_curve_temp_{}", i), format!("{}", self.fan_curve_temp[i])));
        }
        for i in 0..4 {
            vars.push((format!("mem_div{}", i), format!("{}", self.mem_dpm_freq[i][1])));
            vars.push((format!("mem_dpm{}", i), format!("{}", self.mem_dpm_freq[i][0])));
            vars.push((format!("vddci_dpm{}", i), format!("{}", self.mem_dpm_volt[i][0])));
            vars.push((format!("mvdd_dpm{}", i), format!("{}", self.mem_dpm_volt[i][1])));
            vars.push((format!("power_limit_ac_{}", i), format!("{}", self.power_limit_ac[i])));
        }
        for (i, limit) in self.temp_limits.iter().enumerate().take(9) {
            vars.push((format!("temp_limit_{}", i), format!("{}", limit)));
        }
        vars.push(("gfx_dpm_range".to_string(), crate::ui::cards::format_dpm_range(&self.freq_gfx)));
        vars.push(("modified_bytes".to_string(), self.modified_byte_count().to_string()));
        vars.push(("validation_warnings".to_string(), self.validate_tunables().len().to_string()));
        let (_, diff_preview_total) = crate::ui::save_diff_modal::collect_bounded_byte_diffs(&self.baseline_buffer, &self.raw_buffer, 64);
        vars.push(("diff_preview_total".to_string(), diff_preview_total.to_string()));
        let gfx_span_norm = crate::ui::curve_plot::normalize_curve_axis(
            self.freq_gfx[1] as f32,
            self.freq_gfx[0] as f32,
            3500.0,
        );
        vars.push(("gfx_span_norm".to_string(), format!("{:.3}", gfx_span_norm)));
        let badge = crate::ui::theme::tinted_badge_fill(eframe::egui::Color32::from_rgb(64, 160, 128), 36);
        vars.push(("theme_badge_alpha".to_string(), badge.a().to_string()));
        vars.sort_by(|a, b| a.0.cmp(&b.0));

        let mut out = self.format_inspect_report(&self.loaded_file.as_ref().map(|p| p.to_string_lossy()).unwrap_or_else(|| "GUI Buffer".into()));
        out.push_str("\n=== NaviTune FULL SYSTEM & TABLE DEBUG DUMP ===\n");
        out.push_str(&format!("OS / Arch        : {} / {}\n", std::env::consts::OS, std::env::consts::ARCH));
        out.push_str(&format!("Selected GPU     : {}\n", gpu_desc));
        out.push_str(&format!("Telemetry Active : {}\n", self.telemetry_data.lock().is_ok()));
        let did = self.vbios_buffer.as_deref().and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0))).map(|vi| vi.device_id).unwrap_or(0);
        let chip_name = if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer) {
            t3.detect_chip(did).display_name()
        } else if matches!(crate::ppt_arch::PptArch::detect(&self.raw_buffer), crate::ppt_arch::PptArch::Vega10 | crate::ppt_arch::PptArch::Vega20) {
            let frev_v = self.raw_buffer.get(2).copied().unwrap_or(0);
            crate::pptable_vega::VegaChip::from_did_and_rev(did, frev_v).display_name()
        } else if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Rdna1 {
            if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(&self.raw_buffer) {
                t1.detect_chip(did).display_name()
            } else {
                crate::pptable_rdna1::NaviRDNA1Chip::from_did_and_clock(did, self.oc_gfx_clock[1].max(0) as u32).display_name()
            }
        } else {
            crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).map(|t| t.detect_chip(&self.raw_buffer).display_name()).unwrap_or(if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Polaris { "Polaris (SMU 8.0, rev 0x07)" } else { "Unknown" })
        };
        out.push_str(&format!("Detected Silicon : {}\n", chip_name));
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        out.push_str(&format!("Table Struct     : {} ({}, default {} B)\n", arch.table_struct_name(), arch.smu_generation(), arch.default_table_size()));
        out.push_str(&format!("Encoded Size     : {} bytes\n", self.raw_buffer.len()));
        if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(&self.raw_buffer) {
            out.push_str(&format!("Vega 20 Offsets  : PPT0={}W (0x11D), TDC={}/{}A (0x13D), GFX_V={}..{}mV (0x159)\n", {v20.socket_power_limit_ac[0]}, {v20.tdc_limit_gfx}, {v20.tdc_limit_soc}, {v20.min_voltage_gfx}/4, {v20.max_voltage_gfx}/4));
        } else if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&self.raw_buffer) {
            out.push_str(&format!("Polaris Offsets  : OD_GFX={}MHz (0x017), OD_MEM={}MHz (0x01B), FanOff=0x{:03X}, HardLimOff=0x{:03X}\n", {pol.max_od_engine_clock}/100, {pol.max_od_memory_clock}/100, {pol.fan_table_offset}, {pol.hard_limit_table_offset}));
        } else if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(&self.raw_buffer) {
            out.push_str(&format!("Vega 10 Offsets  : OD_GFX={}MHz (0x017), OD_MEM={}MHz (0x01B), FanOff=0x{:03X}, HardLimOff=0x{:03X}\n", {v10.max_od_engine_clock}/100, {v10.max_od_memory_clock}/100, {v10.fan_table_offset}, {v10.hard_limit_table_offset}));
        }
        let feat_mask: u64 = if arch.is_gcn_subtable() && self.raw_buffer.len() >= 0x17 {
            u32::from_le_bytes([self.raw_buffer[0x13], self.raw_buffer[0x14], self.raw_buffer[0x15], self.raw_buffer[0x16]]) as u64
        } else {
            (0..64usize).fold(0u64, |acc, bit| if self.smu_features[bit] { acc | (1u64 << bit) } else { acc })
        };
        let decoded_feats: Vec<String> = arch.decode_feature_bits(feat_mask).into_iter().map(|(bit, name)| format!("{}:{}", bit, name)).collect();
        if arch.is_gcn_subtable() { out.push_str(&format!("PlatformCaps     : 0x{:08X} ({})\n", feat_mask as u32, decoded_feats.join(", "))); } else { out.push_str(&format!("FeaturesToRun    : 0x{:016X} ({})\n", feat_mask, decoded_feats.join(", "))); }
        out.push_str(&format!("Decoded Vars     : {} entries\n", vars.len()));
        for (k, v) in vars {
            out.push_str(&format!("  {} = {}\n", k, v));
        }
        out
    }

}
