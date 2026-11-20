mod gpu_telemetry;
mod advance_os;
mod card_layout;
mod debug_tab;
mod export_formats;
mod import_formats;
mod io_ppt;
mod macos_ioreg;
mod more_popups;
mod pptable_rdna2;
mod smu_features;
mod ui_theme;
mod vbios;

use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(crate::ui_theme::load_saved_window_size())
            .with_min_inner_size([580.0, 500.0])
            .with_resizable(true)
            .with_transparent(true),
        #[cfg(target_os = "windows")]
        renderer: eframe::Renderer::Glow,
        vsync: true,
        multisampling: 0,
        ..Default::default()
    };
    eframe::run_native(
        "NaviTune - RDNA 2 PowerPlay Editor",
        options,
        Box::new(|cc| {
            crate::ui_theme::apply_modern_theme(&cc.egui_ctx);
            crate::ui_theme::log_debug("APP", "NaviTune started");
            let mut app = PowerPlayApp::default();
            app.refresh_ioreg_gpus();
            Ok(Box::new(app))
        }),
    )
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Features,
    OcLimits,
    Power,
    Frequency,
    Curve,
    Fan,
    Advance,
    More,
}

#[derive(PartialEq, Clone, Copy)]
enum AdvanceOsTab {
    MacOS,
    Windows,
    Linux,
}

pub struct PowerPlayApp {
    selected_gpu: usize,
    gpus: Vec<String>,
    ioreg_entries: Vec<crate::macos_ioreg::IoregGpuEntry>,
    live_telemetry_on: bool,
    telemetry_data: std::sync::Arc<std::sync::Mutex<crate::gpu_telemetry::GpuTelemetry>>,
    active_tab: Tab,
    advance_os_tab: AdvanceOsTab,
    pub raw_buffer: Vec<u8>,
    pub baseline_gen: u64,
    pub vbios_buffer: Option<Vec<u8>>,
    pub vbios_ppt_offset: Option<usize>,
    #[allow(dead_code)]
    pub loaded_file: Option<PathBuf>,
    pub probe_offset: String,
    pub probe_write_val: String,
    pub probe_write_type: usize,
    pub debug_subtab: crate::debug_tab::DebugSubTab,
    pub debug_only_mismatch: bool,
    pub show_smu_features_window: bool,
    pub show_struct_inspector_window: bool,
    pub show_opencore_copy_modal: bool,
    pub more_popups: crate::more_popups::MorePopupsState,
    pub show_error_modal: bool,
    pub modal_error_title: String,
    pub modal_error_msg: String,
    pub status_msg: String,

    pub gpu_clock_limit: bool, pub gpu_clock_curve: bool, pub memory_clock_limit: bool, pub zero_rpm: bool,
    pub min_fan_speed: bool, pub fan_target_temp_limit: bool, pub operating_temp_limit: bool, pub fan_acoustic_rpm: bool,
    pub fan_curve: bool, pub power_limit_feat: bool, pub timing_tuning: bool, pub power_mode: bool,
    pub smu_features: [bool; 64],
    pub auto_undervolt_gpu: bool, pub auto_overclock_gpu: bool, pub auto_overclock_mem: bool, pub auto_fan_acoustic_limit: bool,

    pub oc_gfx_clock: [i32; 2], pub oc_mem_clock: [i32; 2], pub oc_power_limit: [i32; 2],
    pub oc_fan_rpm: [i32; 2], pub oc_fan_acoustic_rpm: [i32; 2],
    pub oc_mem_timing_ctrl: i32, pub oc_power_mode_ctrl: i32, pub oc_zero_rpm_ctrl: i32,

    pub pm_quiet: [i32; 2], pub pm_balanced: [i32; 2], pub pm_turbo: [i32; 2], pub pm_rage: [i32; 2],
    pub ac_quiet: [i32; 2], pub ac_balanced: [i32; 2], pub ac_turbo: [i32; 2], pub ac_rage: [i32; 2],

    pub volt_gfx: [i32; 2], pub volt_soc: [i32; 2], pub power_limit_w: i32, pub tdc_gfx: i32, pub tdc_soc: i32,
    pub ulv_gfx: [i32; 2], pub ulv_soc: [i32; 2], pub tvmin_gfx: [i32; 2], pub tvmin_soc: [i32; 2],
    pub vmin_lh_gfx: [i32; 2], pub vmin_lh_soc: [i32; 2],
    pub mem_dpm_volt: [[i32; 2]; 4], pub mem_dpm_freq: [[i32; 2]; 4],

    pub freq_gfx: [i32; 2], pub freq_soc: [i32; 2], pub freq_vclk: [i32; 2], pub freq_dclk: [i32; 2],
    pub freq_dcefclk: [i32; 2], pub freq_dispclk: [i32; 2], pub freq_pixclk: [i32; 2], pub freq_phyclk: [i32; 2],
    pub freq_dtbclk: [i32; 2], pub freq_fclk: [i32; 2], pub fclk_boost_freq: i32,
    pub pcie_gen_speed: [i32; 2], pub pcie_lane_count: [i32; 2], pub lclk_mhz: [i32; 2],

    pub static_volt_a: [f64; 2], pub static_volt_b: [f64; 2], pub static_volt_c: [f64; 2],
    pub static_off_a: [f64; 2], pub static_off_b: [f64; 2], pub static_off_c: [f64; 2],
    pub dcbtc_gfx_en: bool, pub dcbtc_soc_en: bool,
    pub dcbtc_min: [f64; 2], pub dcbtc_max: [f64; 2], pub dcbtc_gb: [f64; 2], pub dctol: [i32; 2],

    pub fan_pwm_min: i32, pub fan_acoustic_limit: i32, pub fan_throttling_rpm: i32, pub fan_max_rpm: i32,
    pub fan_mode: i32, pub fan_hyst_ctrl: i32,
    pub fan_target_temp: i32, pub fan_target_gfx_clock: i32, pub fan_zero_rpm_enable: bool,
    pub fan_stop_temp: i32, pub fan_start_temp: i32,
    pub od_fan_temp: [i32; 2], pub od_fan_speed: [i32; 2],
    pub fan_curve_temp: [i32; 5], pub fan_curve_pwm: [i32; 5],
    pub temp_limits: [i32; 9],
    pub power_limit_ac: [i32; 4],
}

impl Default for PowerPlayApp {
    fn default() -> Self {
        Self {
            fan_curve_temp: [0; 5],
            fan_curve_pwm: [0; 5],
            temp_limits: [0; 9],
            power_limit_ac: [0; 4],
            selected_gpu: 0, gpus: Vec::new(), ioreg_entries: Vec::new(),
            live_telemetry_on: false,
            telemetry_data: std::sync::Arc::new(std::sync::Mutex::new(crate::gpu_telemetry::GpuTelemetry::default())),
            active_tab: Tab::Features,
            advance_os_tab: AdvanceOsTab::MacOS, raw_buffer: Vec::new(),
            baseline_gen: 0,
            vbios_buffer: None, vbios_ppt_offset: None, loaded_file: None,
            probe_offset: format!("0x{:03X}", std::mem::offset_of!(crate::pptable_rdna2::AtomSmcDpmInfoV4_9, socket_power_limit_ac)), probe_write_val: String::new(), probe_write_type: 1,
            debug_subtab: crate::debug_tab::DebugSubTab::Audit, debug_only_mismatch: false,
            show_smu_features_window: false,
            show_struct_inspector_window: false,
            show_opencore_copy_modal: false, more_popups: crate::more_popups::MorePopupsState::default(),
            show_error_modal: false, modal_error_title: String::new(), modal_error_msg: String::new(),
            status_msg: "Ready".to_string(),
            gpu_clock_limit: false, gpu_clock_curve: false, memory_clock_limit: false, zero_rpm: false,
            min_fan_speed: false, fan_target_temp_limit: false, operating_temp_limit: false, fan_acoustic_rpm: false,
            fan_curve: false, power_limit_feat: false, timing_tuning: false, power_mode: false,
            smu_features: [false; 64],
            auto_undervolt_gpu: false, auto_overclock_gpu: false, auto_overclock_mem: false, auto_fan_acoustic_limit: false,
            oc_gfx_clock: [0, 0], oc_mem_clock: [0, 0], oc_power_limit: [0, 0],
            oc_fan_rpm: [0, 0], oc_fan_acoustic_rpm: [0, 0],
            oc_mem_timing_ctrl: 0, oc_power_mode_ctrl: 0, oc_zero_rpm_ctrl: 0,
            pm_quiet: [0, 0], pm_balanced: [0, 0], pm_turbo: [0, 0], pm_rage: [0, 0],
            ac_quiet: [0, 0], ac_balanced: [0, 0], ac_turbo: [0, 0], ac_rage: [0, 0],
            volt_gfx: [0, 0], volt_soc: [0, 0], power_limit_w: 0, tdc_gfx: 0, tdc_soc: 0,
            ulv_gfx: [0, 0], ulv_soc: [0, 0], tvmin_gfx: [0, 0], tvmin_soc: [0, 0],
            vmin_lh_gfx: [0, 0], vmin_lh_soc: [0, 0],
            mem_dpm_volt: [[0, 0]; 4], mem_dpm_freq: [[0, 0]; 4],
            freq_gfx: [0, 0], freq_soc: [0, 0], freq_vclk: [0, 0], freq_dclk: [0, 0],
            freq_dcefclk: [0, 0], freq_dispclk: [0, 0], freq_pixclk: [0, 0], freq_phyclk: [0, 0],
            freq_dtbclk: [0, 0], freq_fclk: [0, 0], fclk_boost_freq: 0,
            pcie_gen_speed: [0, 0], pcie_lane_count: [0, 0], lclk_mhz: [0, 0],
            static_volt_a: [0.0, 0.0], static_volt_b: [0.0, 0.0], static_volt_c: [0.0, 0.0],
            static_off_a: [0.0, 0.0], static_off_b: [0.0, 0.0], static_off_c: [0.0, 0.0],
            dcbtc_gfx_en: false, dcbtc_soc_en: false,
            dcbtc_min: [0.0, 0.0], dcbtc_max: [0.0, 0.0], dcbtc_gb: [0.0, 0.0], dctol: [0, 0],
            fan_pwm_min: 0, fan_acoustic_limit: 0, fan_throttling_rpm: 0, fan_max_rpm: 0,
            fan_mode: 0, fan_hyst_ctrl: 0,
            fan_target_temp: 0, fan_target_gfx_clock: 0, fan_zero_rpm_enable: false,
            fan_stop_temp: 0, fan_start_temp: 0,
            od_fan_temp: [0, 0], od_fan_speed: [0, 0],
        }
    }
}
impl eframe::App for PowerPlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rect) = ctx.input(|i| i.viewport().inner_rect) { crate::ui_theme::save_window_size(ctx, rect.size()); }
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("ppt_loaded"), !self.raw_buffer.is_empty());
            let prev_lbl = d.get_temp::<String>(egui::Id::new("ppt_loaded_label")).unwrap_or_default();
            if prev_lbl != self.status_msg {
                d.insert_temp(egui::Id::new("ppt_loaded_label"), self.status_msg.clone());
                let gen = d.get_temp::<u64>(egui::Id::new("ppt_baseline_gen")).unwrap_or(0).wrapping_add(1);
                d.insert_temp(egui::Id::new("ppt_baseline_gen"), gen);
            }
        });
        self.render_error_modal(ctx);
        if ctx.data_mut(|d| {
            let dirty = d.get_temp::<bool>(egui::Id::new("ppt_dirty")).unwrap_or(false);
            if dirty { d.insert_temp(egui::Id::new("ppt_dirty"), false); }
            dirty
        }) {
            self.sync_fields_to_buffer();
        }
        self.render_more_viewports(ctx);

        egui::TopBottomPanel::bottom("bottom_action_bar")
            .exact_height(38.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Load").clicked() {
                        self.load_ppt_file();
                    }
                    if ui.add(egui::Button::new("Save").fill(egui::Color32::from_rgb(200, 42, 65))).clicked() {
                        self.save_ppt_file();
                    }
                    if ui.button("Reset All").clicked() {
                        self.reset_all_values();
                    }
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(165, 175, 195), &self.status_msg);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Exit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            crate::ui_theme::card_frame(ui).inner_margin(6.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let current_text = self.gpus.get(self.selected_gpu).cloned().unwrap_or_else(|| "Select GPU Path...".into());
                    let prev_gpu = self.selected_gpu;
                    egui::ComboBox::from_id_salt("gpu_selector")
                        .width(ui.available_width() - 86.0)
                        .selected_text(current_text)
                        .show_ui(ui, |ui| {
                            for (i, gpu) in self.gpus.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_gpu, i, gpu);
                            }
                        });
                    if prev_gpu != self.selected_gpu {
                        self.select_ioreg_gpu(self.selected_gpu);
                    }
                    if ui.button("Refresh").clicked() {
                        self.refresh_ioreg_gpus();
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let was_on = self.live_telemetry_on;
                    ui.toggle_value(&mut self.live_telemetry_on, "Live Telemetry");
                    if self.live_telemetry_on && !was_on {
                        let shared = self.telemetry_data.clone();
                        std::thread::spawn(move || {
                            loop {
                                let sample = crate::gpu_telemetry::poll_gpu_telemetry();
                                if let Ok(mut guard) = shared.lock() { *guard = sample; }
                                std::thread::sleep(std::time::Duration::from_secs(1));
                            }
                        });
                    }
                    if self.live_telemetry_on {
                        ctx.request_repaint_after(std::time::Duration::from_secs(1));
                        if let Ok(t) = self.telemetry_data.lock() {
                            if t.valid {
                                let vram_str = if t.vram_total_mb > 0 {
                                    format!(" | VRAM: {}/{} MB", t.vram_used_mb, t.vram_total_mb)
                                } else if t.vram_used_mb > 0 {
                                    format!(" | VRAM: {} MB", t.vram_used_mb)
                                } else {
                                    String::new()
                                };
                                ui.colored_label(
                                    egui::Color32::from_rgb(72, 235, 148),
                                    egui::RichText::new(format!(
                                        "GFX: {} MHz | MEM: {} MHz | Temp: {}°C (Hot {}°C) | Load: {}% | Power: {} W | Fan: {} RPM ({}%){}",
                                        t.gfx_mhz, t.mem_mhz, t.temp_c, t.hotspot_c, t.load_pct, t.power_w, t.fan_rpm, t.fan_pct, vram_str
                                    )).monospace(),
                                );
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(165, 175, 195), "Polling GPU sensors...");
                            }
                        }
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(140, 150, 170), "Click Live Telemetry to monitor real-time GFX/MEM clocks, temp, power & fan.");
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    let id_open = egui::Id::new("tahoe_theme_open");
                    let mut theme_open = ctx.data(|d| d.get_temp::<bool>(id_open).unwrap_or(false));
                    if ui.toggle_value(&mut theme_open, "Theme").changed() {
                        ctx.data_mut(|d| d.insert_temp(id_open, theme_open));
                    }
                    if theme_open {
                        let id_a = egui::Id::new("tahoe_glass_alpha");
                        let id_acc = egui::Id::new("tahoe_accent_color");
                        let id_base = egui::Id::new("tahoe_base_color");
                        let id_txt = egui::Id::new("tahoe_text_color");
                        let id_chk = egui::Id::new("tahoe_check_color");

                        let mut alpha = ctx.data(|d| d.get_temp::<f32>(id_a).unwrap_or(0.90));
                        let mut accent = ctx.data(|d| d.get_temp::<egui::Color32>(id_acc).unwrap_or(crate::ui_theme::DEFAULT_ACCENT));
                        let mut base = ctx.data(|d| d.get_temp::<egui::Color32>(id_base).unwrap_or(crate::ui_theme::DEFAULT_GLASS_BASE));
                        let mut text = ctx.data(|d| d.get_temp::<egui::Color32>(id_txt).unwrap_or(crate::ui_theme::DEFAULT_TEXT_COLOR));
                        let mut check = ctx.data(|d| d.get_temp::<egui::Color32>(id_chk).unwrap_or(crate::ui_theme::DEFAULT_CHECK_COLOR));
                        let mut changed = false;

                        for preset in crate::ui_theme::THEME_PRESETS {
                            if ui.small_button(preset.name).clicked() {
                                alpha = preset.alpha;
                                accent = preset.accent;
                                base = preset.base;
                                text = preset.text;
                                check = preset.check;
                                changed = true;
                            }
                        }
                        ui.separator();
                        ui.small("Opacity:");
                        changed |= ui.add(egui::Slider::new(&mut alpha, 0.35..=1.0).show_value(false)).changed();
                        ui.small(format!("{:.0}%", alpha * 100.0));
                        ui.small("Accent:");
                        changed |= ui.color_edit_button_srgba(&mut accent).changed();
                        ui.small("Glass:");
                        changed |= ui.color_edit_button_srgba(&mut base).changed();
                        ui.small("Text:");
                        changed |= ui.color_edit_button_srgba(&mut text).changed();
                        ui.small("Tick:");
                        changed |= ui.color_edit_button_srgba(&mut check).changed();

                        if changed {
                            crate::ui_theme::apply_full_theme(ctx, alpha, accent, base, text, check);
                        }
                    }
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                for (tab, label) in [
                    (Tab::Features, "Features"),
                    (Tab::OcLimits, "OC Limits"),
                    (Tab::Power, "Power"),
                    (Tab::Frequency, "Frequency"),
                    (Tab::Curve, "Curve"),
                    (Tab::Fan, "Fan"),
                    (Tab::Advance, "Advance"),
                    (Tab::More, "More"),
                ] {
                    ui.selectable_value(&mut self.active_tab, tab, label);
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                match self.active_tab {
                    Tab::Features => self.draw_features_tab(ui),
                    Tab::OcLimits => self.draw_oc_limits_tab(ui),
                    Tab::Power => self.draw_power_tab(ui),
                    Tab::Frequency => self.draw_frequency_tab(ui),
                    Tab::Curve => self.draw_curve_tab(ui),
                    Tab::Fan => self.draw_fan_tab(ui),
                    Tab::Advance => self.draw_advance_tab(ui),
                    Tab::More => self.render_debug_more_tab(ui),
                }
            });
        });
    }
}

impl PowerPlayApp {
    fn reset_all_values(&mut self) {
        let gpus = self.gpus.clone();
        let entries = self.ioreg_entries.clone();
        *self = Self::default();
        self.gpus = gpus;
        self.ioreg_entries = entries;
        self.status_msg = "Reset all values".to_string();
    }

    fn refresh_ioreg_gpus(&mut self) {
        let entries = crate::macos_ioreg::scan_ioreg_gpus();
        let mut list = vec!["Select GPU Path...".into()];
        if entries.is_empty() {
            #[cfg(target_os = "macos")]
            list.push("PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0) [No PPTable]".into());
            #[cfg(target_os = "windows")]
            list.push("HKLM\\...\\Class\\{4d36e968...}\\0000 [No PPTable]".into());
            #[cfg(target_os = "linux")]
            list.push("/sys/class/drm/card0/device/pp_table [No PPTable]".into());
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            list.push("AMD Radeon GPU [No PPTable]".into());
            self.ioreg_entries.clear();
        } else {
            for e in &entries {
                if e.ppt_bytes.is_some() {
                    list.push(format!("{} [PPTable]", e.label));
                } else {
                    list.push(format!("{} [No PPTable]", e.label));
                }
            }
            self.ioreg_entries = entries;
        }
        let prev_label = self.gpus.get(self.selected_gpu).cloned();
        self.gpus = list;
        self.selected_gpu = prev_label
            .and_then(|lbl| self.gpus.iter().position(|g| g == &lbl))
            .unwrap_or(0);
    }

    fn select_ioreg_gpu(&mut self, idx: usize) {
        if idx == 0 {
            return;
        }
        if let Some(entry) = self.ioreg_entries.get(idx - 1).cloned() {
            if let Some(bytes) = entry.ppt_bytes {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                self.apply_ppt_bytes(bytes, &entry.label);
            } else {
                self.status_msg = format!("{} has no active Soft PowerPlay table in OS registry/sysfs (load a .bin/.rom/.mpt file to edit)", entry.label);
            }
        } else if let Some(label) = self.gpus.get(idx) {
            self.status_msg = format!("{} has no active PowerPlay table", label);
        }
    }

    fn draw_features_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            let btn = egui::Button::new("PPTable Features").min_size(egui::vec2(180.0, 28.0));
            if ui.add(btn).clicked() {
                self.show_smu_features_window = !self.show_smu_features_window;
            }
        });
        ui.add_space(6.0);

        crate::card_layout::draw_card(ui, "OverDrive Features", |ui| {
            ui.columns(2, |cols| {
                cols[0].spacing_mut().item_spacing.y = 4.0;
                cols[1].spacing_mut().item_spacing.y = 4.0;
                let left_items = [
                    (&mut self.gpu_clock_limit, "GPU Clock Limit", "od_feat_0"),
                    (&mut self.gpu_clock_curve, "GPU Clock Curve", "od_feat_1"),
                    (&mut self.memory_clock_limit, "Memory Clock Limit", "od_feat_2"),
                    (&mut self.zero_rpm, "Zero RPM", "od_feat_3"),
                    (&mut self.min_fan_speed, "Minimum Fan Speed", "od_feat_4"),
                    (&mut self.fan_target_temp_limit, "Fan Target Temperature Limit", "od_feat_5"),
                    (&mut self.operating_temp_limit, "Operating Temperature Limit", "od_feat_6"),
                    (&mut self.fan_acoustic_rpm, "Fan Acoustic RPM", "od_feat_7"),
                ];
                for (flag, lbl, tag) in left_items {
                    cols[0].horizontal(|ui| {
                        crate::ui_theme::green_checkbox(ui, flag, lbl);
                        ui.colored_label(egui::Color32::from_rgb(130, 145, 170), egui::RichText::new(format!("[{}]", tag)).monospace().small());
                    });
                }
                let right_items = [
                    (&mut self.fan_curve, "Fan Curve", "od_feat_8"),
                    (&mut self.power_limit_feat, "Power Limit", "od_feat_9"),
                    (&mut self.timing_tuning, "Timing Tuning", "od_feat_10"),
                    (&mut self.power_mode, "Power Mode", "od_feat_11"),
                    (&mut self.auto_undervolt_gpu, "Auto Undervolt GPU", "od_feat_12"),
                    (&mut self.auto_overclock_gpu, "Auto Overclock GPU", "od_feat_13"),
                    (&mut self.auto_overclock_mem, "Auto Overclock Memory", "od_feat_14"),
                    (&mut self.auto_fan_acoustic_limit, "Auto Fan Acoustic Limit", "od_feat_15"),
                ];
                for (flag, lbl, tag) in right_items {
                    cols[1].horizontal(|ui| {
                        crate::ui_theme::green_checkbox(ui, flag, lbl);
                        ui.colored_label(egui::Color32::from_rgb(130, 145, 170), egui::RichText::new(format!("[{}]", tag)).monospace().small());
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
        ui.add_space(6.0);
        crate::card_layout::draw_card(ui, "VBIOS ROM & Table Header Verification", |ui| {
            let info = self
                .vbios_buffer
                .as_ref()
                .and_then(|rom| crate::vbios::inspect_vbios_info(rom, self.vbios_ppt_offset.unwrap_or(0)));

            egui::Grid::new("vbios_header_grid")
                .num_columns(2)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Silicon Family:");
                    if let Some(t) = crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer) {
                        let chip = t.detect_chip(&self.raw_buffer);
                        ui.colored_label(
                            egui::Color32::from_rgb(95, 200, 145),
                            egui::RichText::new(chip.display_name()).strong(),
                        );
                    } else {
                        ui.weak("Not detected");
                    }
                    ui.end_row();

                    ui.label("VBIOS ROM:");
                    if let Some(ref vi) = info {
                        ui.monospace(format!(
                            "{} [{:04X}:{:04X}] | {} B (Legacy {} B, PPTable @ 0x{:X}, {} B)",
                            vi.device_name,
                            vi.vendor_id,
                            vi.device_id,
                            vi.total_size,
                            vi.legacy_size,
                            vi.ppt_offset,
                            vi.ppt_size
                        ));
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
                        let csum_ok = vi.checksum_sum == 0;
                        let csum_txt = format!(
                            "{} (byte[0x21]=0x{:02X}, 8-bit sum=0x{:02X})",
                            if csum_ok { "VALID" } else { "INVALID" },
                            vi.checksum_byte,
                            vi.checksum_sum
                        );
                        if csum_ok {
                            ui.colored_label(egui::Color32::from_rgb(80, 210, 120), egui::RichText::new(csum_txt).monospace());
                        } else {
                            ui.colored_label(egui::Color32::from_rgb(245, 75, 95), egui::RichText::new(csum_txt).monospace());
                        }
                        ui.end_row();
                    }

                    ui.label("Header Verification:");
                    if let Some(t) = crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer) {
                        let sz = { t.header.structure_size };
                        let fmt_rev = { t.header.format_revision };
                        let cnt_rev = { t.header.content_revision };
                        let tbl_rev = { t.table_revision };
                        let ok = sz as usize == self.raw_buffer.len();
                        let msg = format!(
                            "{} | size={} B | rev=0x{:02X}.{:02X}.{:02X}",
                            if ok { "VALID" } else { "SIZE MISMATCH" },
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
                        ui.weak("No 2,470-byte AtomSmcDpmInfoV4_9 table loaded");
                    }
                    ui.end_row();
                });
        });
    }

    fn draw_oc_limits_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;
        two_col_cards(ui, "oc_cols", |ui| {
            draw_card(ui, "Overdrive", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                kv_row_2(ui, "GFX Clock (MHz)", "od_gfx_min/max", &mut self.oc_gfx_clock, 62.0);
                kv_row_2(ui, "Memory Clock (MHz)", "od_mem_min/max", &mut self.oc_mem_clock, 62.0);
                kv_row_2(ui, "Power Limit (%)", "od_pwr_min/max", &mut self.oc_power_limit, 62.0);
                kv_row_1(ui, "Mem Timing Ctrl", "od_mem_timing", &mut self.oc_mem_timing_ctrl, 62.0);
                kv_row_1(ui, "Power Mode Ctrl", "od_pwr_mode", &mut self.oc_power_mode_ctrl, 62.0);
                kv_row_2(ui, "Fan (RPM)", "od_fan_min/max", &mut self.oc_fan_rpm, 62.0);
                kv_row_2(ui, "Fan Acoustic (RPM)", "od_acoustic_min/max", &mut self.oc_fan_acoustic_rpm, 62.0);
                kv_row_1(ui, "Zero RPM Ctrl", "od_zero_rpm", &mut self.oc_zero_rpm_ctrl, 62.0);
            });
        }, |ui| {
            draw_card(ui, "Power Modes", |ui| {
                kv_header_2(ui, "Limit (%)", "Temp (°C)", 62.0);
                kv_row_2(ui, "Quiet", "pm_quiet_pwr/temp", &mut self.pm_quiet, 62.0);
                kv_row_2(ui, "Balanced", "pm_bal_pwr/temp", &mut self.pm_balanced, 62.0);
                kv_row_2(ui, "Turbo", "pm_turbo_pwr/temp", &mut self.pm_turbo, 62.0);
                kv_row_2(ui, "Rage", "pm_rage_pwr/temp", &mut self.pm_rage, 62.0);
                ui.add_space(4.0);
                kv_header_2(ui, "Acoustic", "Target", 62.0);
                kv_row_2(ui, "Quiet", "pm_quiet_ac", &mut self.ac_quiet, 62.0);
                kv_row_2(ui, "Balanced", "pm_bal_ac", &mut self.ac_balanced, 62.0);
                kv_row_2(ui, "Turbo", "pm_turbo_ac", &mut self.ac_turbo, 62.0);
                kv_row_2(ui, "Rage", "pm_rage_ac", &mut self.ac_rage, 62.0);
            });
        });
    }

    fn draw_power_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;
        two_col_cards(ui, "pwr_cols", |ui| {
            draw_card(ui, "Voltage Vmin / Vmax (mV)", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                kv_row_2(ui, "GFX", "vmin/vmax_gfx", &mut self.volt_gfx, 62.0);
                kv_row_2(ui, "Soc", "vmin/vmax_soc", &mut self.volt_soc, 62.0);
            });
            draw_card(ui, "Power & TDC Limits", |ui| {
                kv_row_1(ui, "Power Limit (W)", "power_limit_gpu", &mut self.power_limit_w, 62.0);
                kv_row_1(ui, "TDC GFX (A)", "tdc_gfx", &mut self.tdc_gfx, 62.0);
                kv_row_1(ui, "TDC Soc (A)", "tdc_soc", &mut self.tdc_soc, 62.0);
            });
            draw_card(ui, "ULV Vmin / Offset (mV)", |ui| {
                kv_header_2(ui, "Vmin", "Offset", 62.0);
                kv_row_2(ui, "GFX", "ulv_gfx", &mut self.ulv_gfx, 62.0);
                kv_row_2(ui, "Soc", "ulv_soc", &mut self.ulv_soc, 62.0);
            });
        }, |ui| {
            draw_card(ui, "Memory VDDCI / MVDD (mV)", |ui| {
                kv_header_2(ui, "VDDCI", "MVDD", 62.0);
                for i in 0..4 {
                    kv_row_2(ui, &format!("DPM {}", i), &format!("vddci/mvdd_{}", i), &mut self.mem_dpm_volt[i], 62.0);
                }
            });
            draw_card(ui, "Temperature Dependent Vmin", |ui| {
                kv_header_2(ui, "TVmin (°C)", "Hyst (°C)", 62.0);
                kv_row_2(ui, "GFX", "tvmin/hyst_gfx", &mut self.tvmin_gfx, 62.0);
                kv_row_2(ui, "Soc", "tvmin/hyst_soc", &mut self.tvmin_soc, 62.0);
                ui.add_space(4.0);
                kv_header_2(ui, "Low", "High", 62.0);
                kv_row_2(ui, "GFX", "vmin_low/high_gfx", &mut self.vmin_lh_gfx, 62.0);
                kv_row_2(ui, "Soc", "vmin_low/high_soc", &mut self.vmin_lh_soc, 62.0);
            });
        });
    }

    fn draw_frequency_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;
        two_col_cards(ui, "freq_cols", |ui| {
            draw_card(ui, "Clock Limits (MHz)", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                for (lbl, tag, pair) in [
                    ("GFX", "freq_gfx", &mut self.freq_gfx),
                    ("Soc", "freq_soc", &mut self.freq_soc),
                    ("VCLK", "freq_vclk", &mut self.freq_vclk),
                    ("DCLK", "freq_dclk", &mut self.freq_dclk),
                    ("DCEFCLK", "freq_dcef", &mut self.freq_dcefclk),
                    ("DISPCLK", "freq_disp", &mut self.freq_dispclk),
                    ("PIXCLK", "freq_pix", &mut self.freq_pixclk),
                    ("PHYCLK", "freq_phy", &mut self.freq_phyclk),
                    ("DTBCLK", "freq_dtb", &mut self.freq_dtbclk),
                    ("FCLK", "freq_fclk", &mut self.freq_fclk),
                ] {
                    kv_row_2(ui, lbl, tag, pair, 62.0);
                }
                kv_row_1(ui, "FCLK Boost", "fclk_boost", &mut self.fclk_boost_freq, 62.0);
            });
        }, |ui| {
            draw_card(ui, "Memory DPM (MHz / Div)", |ui| {
                kv_header_2(ui, "Freq", "Div", 62.0);
                for i in 0..4 {
                    kv_row_2(ui, &format!("DPM {}", i), &format!("mem_dpm/div_{}", i), &mut self.mem_dpm_freq[i], 62.0);
                }
            });
            draw_card(ui, "PCIe & LCLK", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                kv_row_2(ui, "PCIe Gen", "pcie_gen", &mut self.pcie_gen_speed, 62.0);
                kv_row_2(ui, "PCIe Lanes", "pcie_lane", &mut self.pcie_lane_count, 62.0);
                kv_row_2(ui, "LCLK (MHz)", "lclk", &mut self.lclk_mhz, 62.0);
            });
        });
    }

    fn draw_curve_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;
        two_col_cards(ui, "crv_cols", |ui| {
            draw_card(ui, "Static Voltage Curve (VID)", |ui| {
                kv_header_2(ui, "GFX", "Soc", 62.0);
                for (lbl, tag, pair) in [
                    ("Coeff a", "curve_a", &mut self.static_volt_a),
                    ("Coeff b", "curve_b", &mut self.static_volt_b),
                    ("Coeff c", "curve_c", &mut self.static_volt_c),
                ] {
                    ui.horizontal(|ui| {
                        label_with_var(ui, lbl, tag);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            float_box(ui, &mut pair[1], 62.0);
                            float_box(ui, &mut pair[0], 62.0);
                        });
                    });
                }
            });
            draw_card(ui, "Static Voltage Offset", |ui| {
                kv_header_2(ui, "GFX", "Soc", 62.0);
                for (lbl, tag, pair) in [
                    ("Coeff a", "off_a", &mut self.static_off_a),
                    ("Coeff b", "off_b", &mut self.static_off_b),
                    ("Coeff c", "off_c", &mut self.static_off_c),
                ] {
                    ui.horizontal(|ui| {
                        label_with_var(ui, lbl, tag);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            float_box(ui, &mut pair[1], 62.0);
                            float_box(ui, &mut pair[0], 62.0);
                        });
                    });
                }
            });
        }, |ui| {
            draw_card(ui, "Thermal Limits (°C)", |ui| {
                let names = [
                    ("Edge", "temp_edge"),
                    ("Hotspot", "temp_hotspot"),
                    ("Memory", "temp_mem"),
                    ("VR GFX", "temp_vrgfx"),
                    ("VR Mem0", "temp_vrmem0"),
                    ("VR Mem1", "temp_vrmem1"),
                    ("VR Soc", "temp_vrsoc"),
                    ("Liquid0", "temp_liq0"),
                    ("Liquid1", "temp_liq1"),
                ];
                for (i, (lbl, tag)) in names.iter().enumerate() {
                    kv_row_1(ui, lbl, tag, &mut self.temp_limits[i], 62.0);
                }
            });
            draw_card(ui, "DcBtc & DcTol (mV)", |ui| {
                kv_header_2(ui, "GFX", "Soc", 62.0);
                ui.horizontal(|ui| {
                    label_with_var(ui, "DcBtc Enable", "dcbtc_en");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.allocate_ui_with_layout(egui::vec2(62.0, 20.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            crate::ui_theme::green_checkbox(ui, &mut self.dcbtc_soc_en, "");
                        });
                        ui.allocate_ui_with_layout(egui::vec2(62.0, 20.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            crate::ui_theme::green_checkbox(ui, &mut self.dcbtc_gfx_en, "");
                        });
                    });
                });
                for (lbl, tag, pair) in [
                    ("DcBtc Min", "dcbtc_min", &mut self.dcbtc_min),
                    ("DcBtc Max", "dcbtc_max", &mut self.dcbtc_max),
                    ("DcBtc GB", "dcbtc_gb", &mut self.dcbtc_gb),
                ] {
                    ui.horizontal(|ui| {
                        label_with_var(ui, lbl, tag);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            dec_box(ui, &mut pair[1], 62.0);
                            dec_box(ui, &mut pair[0], 62.0);
                        });
                    });
                }
                kv_row_2(ui, "DcTol", "dctol", &mut self.dctol, 62.0);
            });
        });
    }

    fn draw_fan_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;
        two_col_cards(ui, "fan_cols", |ui| {
            draw_card(ui, "Fan Parameters", |ui| {
                kv_row_1(ui, "PWM Min (%)", "fan_pwm_min", &mut self.fan_pwm_min, 62.0);
                kv_row_1(ui, "Acoustic Limit", "fan_acoustic_lim", &mut self.fan_acoustic_limit, 62.0);
                kv_row_1(ui, "Throttling RPM", "fan_throttling", &mut self.fan_throttling_rpm, 62.0);
                kv_row_1(ui, "Max RPM", "fan_max_rpm", &mut self.fan_max_rpm, 62.0);
                kv_row_1(ui, "Target Temp (°C)", "fan_target_temp", &mut self.fan_target_temp, 62.0);
                kv_row_1(ui, "Target GFX Clock", "fan_target_gfx", &mut self.fan_target_gfx_clock, 62.0);
                kv_row_1(ui, "Fan Mode", "fan_mode", &mut self.fan_mode, 62.0);
                kv_row_1(ui, "Hyst / Ctrl", "fan_hyst_ctrl", &mut self.fan_hyst_ctrl, 62.0);
            });
            draw_card(ui, "Fan OverDrive Limits", |ui| {
                kv_header_2(ui, "Min", "Max", 62.0);
                kv_row_2(ui, "Fan RPM", "od_fan_rpm", &mut self.oc_fan_rpm, 62.0);
                kv_row_2(ui, "Acoustic RPM", "od_acoustic_rpm", &mut self.oc_fan_acoustic_rpm, 62.0);
                kv_row_2(ui, "OD Temp (°C)", "od_fan_temp", &mut self.od_fan_temp, 62.0);
                kv_row_2(ui, "OD Speed (%)", "od_fan_spd", &mut self.od_fan_speed, 62.0);
            });
        }, |ui| {
            draw_card(ui, "Zero RPM", |ui| {
                ui.horizontal(|ui| {
                    crate::ui_theme::green_checkbox(ui, &mut self.fan_zero_rpm_enable, "Enable Zero RPM");
                    ui.colored_label(egui::Color32::from_rgb(130, 145, 170), egui::RichText::new("[od_zero_rpm]").monospace().small());
                });
                ui.add_space(4.0);
                kv_row_1(ui, "Stop Temp (°C)", "fan_stop_temp", &mut self.fan_stop_temp, 62.0);
                kv_row_1(ui, "Start Temp (°C)", "fan_start_temp", &mut self.fan_start_temp, 62.0);
            });
            draw_card(ui, "Fan Curve (5-Point)", |ui| {
                kv_header_2(ui, "Temp (°C)", "PWM (%)", 62.0);
                for i in 0..5 {
                    let mut pair = [self.fan_curve_temp[i], self.fan_curve_pwm[i]];
                    kv_row_2(ui, &format!("Point {}", i + 1), &format!("fan_pt_{}", i), &mut pair, 62.0);
                    self.fan_curve_temp[i] = pair[0];
                    self.fan_curve_pwm[i] = pair[1];
                }
            });
        });
    }
    fn draw_advance_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::MacOS, "macOS");
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::Windows, "Windows");
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::Linux, "Linux");
        });
        ui.separator();
        ui.add_space(4.0);

        if self.raw_buffer.is_empty() {
            crate::card_layout::draw_card(ui, "OS Optimization Presets", |ui| {
                ui.label("Load a PowerPlay table (.bin, .rom, .reg, .mpt, or ioreg) first.");
            });
            return;
        }

        match self.advance_os_tab {
            AdvanceOsTab::MacOS => self.draw_advance_macos(ui),
            AdvanceOsTab::Windows => self.draw_advance_windows(ui),
            AdvanceOsTab::Linux => self.draw_advance_linux(ui),
        }

        if self.show_opencore_copy_modal {
            let vp_id = egui::ViewportId::from_hash_of("opencore_copy_modal_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("Copy PP_PhmSoftPowerPlayTable")
                .with_inner_size([400.0, 175.0])
                .with_resizable(false);
            ui.ctx().show_viewport_immediate(vp_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) { self.show_opencore_copy_modal = false; }
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.sync_fields_to_buffer();
                    let b64 = crate::export_formats::encode_base64(&self.raw_buffer);
                    let hex_raw: String = self.raw_buffer.iter().map(|b| format!("{:02X}", b)).collect();
                    ui.label(format!("Synced PP_PhmSoftPowerPlayTable ({} bytes):", self.raw_buffer.len()));
                    ui.monospace(format!("Base64: {}...", &b64[..b64.len().min(36)]));
                    ui.monospace(format!("Hex   : {}...", &hex_raw[..hex_raw.len().min(36)]));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Copy Base64").clicked() { ui.ctx().copy_text(b64); self.show_opencore_copy_modal = false; }
                        if ui.button("Copy Hex (No Space)").clicked() { ui.ctx().copy_text(hex_raw); self.show_opencore_copy_modal = false; }
                        if ui.button("Close").clicked() { self.show_opencore_copy_modal = false; }
                    });
                });
            });
        }
    }
}

fn draw_tracked_drag<Num: egui::emath::Numeric + PartialEq + Copy + Send + Sync + 'static>(
    ui: &mut egui::Ui,
    val: &mut Num,
    width: f32,
    speed: f64,
    fixed_decimals: Option<usize>,
    is_modified: impl Fn(Num, Num) -> bool,
) {
    let loaded = ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::new("ppt_loaded")).unwrap_or(false));
    let gen = ui.ctx().data(|d| d.get_temp::<u64>(egui::Id::new("ppt_baseline_gen")).unwrap_or(0));
    if !loaded {
        let mut empty = String::new();
        ui.add_sized([width, 20.0], egui::TextEdit::singleline(&mut empty).interactive(false));
        return;
    }

    let base_id = egui::Id::new(("factory_base", gen, val as *const Num as usize));
    let base_val = ui.ctx().data_mut(|d| {
        if let Some(v) = d.get_temp::<Num>(base_id) {
            v
        } else {
            d.insert_temp(base_id, *val);
            *val
        }
    });

    let modified = is_modified(*val, base_val);
    ui.scope(|ui| {
        if modified {
            let vis = ui.visuals_mut();
            vis.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(185, 35, 52, 55);
            vis.widgets.inactive.weak_bg_fill = egui::Color32::from_rgba_unmultiplied(185, 35, 52, 55);
            vis.widgets.inactive.bg_stroke = egui::Stroke::new(1.25_f32, egui::Color32::from_rgb(240, 70, 90));
            vis.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(205, 45, 65, 75);
            vis.widgets.hovered.weak_bg_fill = egui::Color32::from_rgba_unmultiplied(205, 45, 65, 75);
            vis.widgets.hovered.bg_stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 95, 115));
            vis.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 95, 115));
        }
        let mut dv = egui::DragValue::new(val).speed(speed);
        if let Some(dec) = fixed_decimals {
            dv = dv.fixed_decimals(dec);
        }
        let resp = ui.add_sized([width, 20.0], dv);
        if resp.changed() {
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
        }
        if modified {
            resp.on_hover_text("Modified from loaded factory default");
        }
    });
}

pub fn num_box(ui: &mut egui::Ui, val: &mut i32, width: f32) {
    draw_tracked_drag(ui, val, width, 1.0, None, |cur, base| cur != base);
}

pub fn float_box(ui: &mut egui::Ui, val: &mut f64, width: f32) {
    draw_tracked_drag(ui, val, width, 0.001, Some(6), |cur, base| (cur - base).abs() > 1e-7);
}

pub fn dec_box(ui: &mut egui::Ui, val: &mut f64, width: f32) {
    draw_tracked_drag(ui, val, width, 1.0, Some(0), |cur, base| (cur - base).abs() > 0.5);
}
