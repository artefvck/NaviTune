mod gpu_telemetry;
mod advance_os;
mod card_layout;
mod tabs;
mod export_formats;
mod import_formats;
mod io_ppt;
mod macos_ioreg;
mod more_popups;
mod pptable_rdna2;
mod pptable_rdna3;
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
            app.selected_gpu = usize::MAX;
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
    SafeTune,
    More,
    Log,
}

#[derive(PartialEq, Clone, Copy)]
pub enum AdvanceOsTab {
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
    pub advance_os_tab: AdvanceOsTab,
    pub raw_buffer: Vec<u8>,
    pub baseline_gen: u64,
    pub vbios_buffer: Option<Vec<u8>>,
    pub vbios_ppt_offset: Option<usize>,
    #[allow(dead_code)]
    pub loaded_file: Option<PathBuf>,
    pub probe_offset: String,
    pub probe_write_val: String,
    pub probe_write_type: usize,
    pub debug_subtab: crate::tabs::more_tab::DebugSubTab,
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
            debug_subtab: crate::tabs::more_tab::DebugSubTab::Audit, debug_only_mismatch: false,
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

        egui::TopBottomPanel::bottom("verified_status_footer")
            .exact_height(24.0)
            .show(ctx, |ui| {
                let is_rdna3 = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer).is_some();
                let is_rdna2 = !is_rdna3 && crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).is_some();
                let mut issues: Vec<&str> = Vec::new();
                if !self.raw_buffer.is_empty() {
                    if self.oc_gfx_clock > self.oc_gfx_clock || self.oc_mem_clock > self.oc_mem_clock || self.oc_power_limit > self.oc_power_limit {
                        issues.push("Inverted OD bounds");
                    }
                    if self.freq_gfx > self.freq_gfx || self.freq_soc > self.freq_soc || self.freq_fclk > self.freq_fclk {
                        issues.push("Min clock > Max clock");
                    }
                    if self.volt_gfx > self.volt_gfx || self.volt_soc > self.volt_soc {
                        issues.push("Vmin > Vmax");
                    }
                    if self.temp_limits.iter().any(|&t| !(0..=130).contains(&t)) {
                        issues.push("Thermal limit out of 0..130°C");
                    }
                }
                ui.horizontal(|ui| {
                    if is_rdna3 {
                        ui.colored_label(egui::Color32::from_rgb(95, 200, 145), egui::RichText::new("RDNA 3 · SMU 13.0").small());
                    } else if is_rdna2 {
                        ui.colored_label(egui::Color32::from_rgb(95, 200, 145), egui::RichText::new("RDNA 2 · SMU 11.0").small());
                    } else {
                        ui.label(egui::RichText::new("No table loaded").small().weak());
                    }
                    if ui.ctx().input(|i| i.pointer.hover_pos().is_some()) {
                        let hov = ui.ctx().data(|d| d.get_temp::<String>(egui::Id::new("hovered_field_banner")).unwrap_or_default());
                        if !hov.is_empty() {
                            ui.separator();
                            ui.colored_label(egui::Color32::from_rgb(165, 195, 235), egui::RichText::new(hov).small().monospace());
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !self.raw_buffer.is_empty() {
                            if issues.is_empty() {
                                ui.colored_label(egui::Color32::from_rgb(80, 210, 120), egui::RichText::new("✔ Verified").small());
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(245, 185, 65), egui::RichText::new(format!("⚠ {}", issues.join(" · "))).small());
                            }
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("bottom_action_bar")
            .exact_height(38.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Load").clicked() {
                        self.load_ppt_file();
                    }
                    if ui.add_sized([68.0, 25.0], egui::Button::new(egui::RichText::new("Save").strong().color(egui::Color32::WHITE)).fill(egui::Color32::from_rgb(205, 48, 72))).clicked() {
                        self.save_ppt_file();
                    }
                    if ui.add_sized([82.0, 25.0], egui::Button::new("Reset All")).clicked() {
                        self.reset_all_values();
                    }
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
                let current_text = self.gpus.get(self.selected_gpu).cloned().unwrap_or_else(|| if self.loaded_file.is_some() && !self.status_msg.is_empty() { self.status_msg.clone() } else { "Select GPU Path...".into() });
                let mut picked_gpu: Option<usize> = None;
                egui::ComboBox::from_id_salt("gpu_selector")
                    .width((ui.available_width() - 92.0).max(140.0))
                    .selected_text(current_text)
                    .show_ui(ui, |ui| {
                        for (i, gpu) in self.gpus.iter().enumerate() {
                            if ui.selectable_value(&mut self.selected_gpu, i, gpu).clicked() {
                                picked_gpu = Some(i);
                            }
                        }
                    });
                if let Some(idx) = picked_gpu {
                    self.select_ioreg_gpu(idx);
                }
                let id_open = egui::Id::new("tahoe_theme_open");
                let mut theme_open = ctx.data(|d| d.get_temp::<bool>(id_open).unwrap_or(false));
                if ui.add_sized([76.0, 22.0], egui::Button::new("Theme").selected(theme_open)).clicked() {
                    theme_open = !theme_open;
                    ctx.data_mut(|d| d.insert_temp(id_open, theme_open));
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
                let theme_open = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("tahoe_theme_open")).unwrap_or(false));
                if theme_open {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
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
                    });
                }
            });
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for (tab, label) in [
                    (Tab::Features, "Features"),
                    (Tab::OcLimits, "OC Limits"),
                    (Tab::Power, "Power"),
                    (Tab::Frequency, "Frequency"),
                    (Tab::Curve, "Curve"),
                    (Tab::Fan, "Fan"),
                    (Tab::Advance, "Advance"),
                    (Tab::SafeTune, "Safe Tune"),
                    (Tab::More, "More"),
                    (Tab::Log, "Log"),
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
                    Tab::SafeTune => self.draw_safe_tune_tab(ui),
                    Tab::More => self.render_debug_more_tab(ui),
                    Tab::Log => self.draw_log_tab(ui),
                }
            });
        });
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
        ui.add_sized([width, 20.0], egui::TextEdit::singleline(&mut empty).font(egui::TextStyle::Monospace).interactive(false));
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
        ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.5));
        if !modified {
            let vis = ui.visuals_mut();
            vis.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(128, 134, 144, 34);
            vis.widgets.inactive.weak_bg_fill = egui::Color32::from_rgba_unmultiplied(128, 134, 144, 34);
            vis.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(135, 142, 154, 75));
            vis.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(162, 170, 184));
            vis.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(128, 134, 144, 52);
            vis.widgets.hovered.weak_bg_fill = egui::Color32::from_rgba_unmultiplied(128, 134, 144, 52);
            vis.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(165, 174, 190, 120));
        } else {
            let vis = ui.visuals_mut();
            vis.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(185, 35, 52, 55);
            vis.widgets.inactive.weak_bg_fill = egui::Color32::from_rgba_unmultiplied(185, 35, 52, 55);
            vis.widgets.inactive.bg_stroke = egui::Stroke::new(1.25_f32, egui::Color32::from_rgb(240, 70, 90));
            vis.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(205, 45, 65, 75);
            vis.widgets.hovered.weak_bg_fill = egui::Color32::from_rgba_unmultiplied(205, 45, 65, 75);
            vis.widgets.hovered.bg_stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 95, 115));
            vis.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 95, 115));
        }
        let unit_suffix = if field_lbl.contains("(°C)") || field_tag.contains("temp") {
            " °C"
        } else if field_lbl.contains("(MHz)") || field_tag.contains("freq") || field_tag.contains("lclk") || field_tag.contains("od_gfx") || field_tag.contains("od_mem") {
            " MHz"
        } else if field_lbl.contains("(mV)") || field_tag.contains("volt") || field_tag.contains("ulv") || field_tag.contains("dcbtc") {
            " mV"
        } else if field_lbl.contains("(W)") || field_tag.contains("power_limit") {
            " W"
        } else if field_lbl.contains("(A)") || field_tag.contains("tdc") {
            " A"
        } else if field_lbl.contains("(RPM)") || field_tag.contains("rpm") || field_tag.contains("acoustic") {
            " RPM"
        } else if field_lbl.contains("(%)") || field_tag.contains("pwm") || field_tag.contains("od_pwr") {
            " %"
        } else {
            ""
        };
        let mut dv = egui::DragValue::new(val).speed(speed);
        if !unit_suffix.is_empty() {
            dv = dv.suffix(unit_suffix);
        }
        if let Some(dec) = fixed_decimals {
            dv = dv.fixed_decimals(dec);
        }
        let card_title = ui.ctx().data(|d| d.get_temp::<String>(egui::Id::new("cur_card_title")).unwrap_or_default());
        let (field_lbl, field_tag) = ui.ctx().data(|d| d.get_temp::<(String, String)>(egui::Id::new("cur_field_info")).unwrap_or_default());
        let resp = ui.add_sized([width, 20.0], dv);
        if resp.changed() {
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
        }
        if resp.hovered() || resp.dragged() {
            let info_text = if field_tag.is_empty() {
                format!("{} › {}", card_title, field_lbl)
            } else {
                format!("{} › {} ({})", card_title, field_lbl, field_tag)
            };
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("hovered_field_banner"), info_text.clone()));
            let tip = if modified {
                format!("{}\nFactory default: {:.4}", info_text, base_val.to_f64())
            } else {
                info_text
            };
            resp.on_hover_text(tip);
        }
    });
}

pub fn num_box(ui: &mut egui::Ui, val: &mut i32, width: f32) {
    let w = width;
    draw_tracked_drag(ui, val, w, 1.0, None, |cur, base| cur != base);
}

pub fn float_box(ui: &mut egui::Ui, val: &mut f64, width: f32) {
    let w = width;
    draw_tracked_drag(ui, val, w, 0.001, Some(6), |cur, base| (cur - base).abs() > 1e-7);
}

pub fn dec_box(ui: &mut egui::Ui, val: &mut f64, width: f32) {
    let w = width;
    draw_tracked_drag(ui, val, w, 1.0, Some(0), |cur, base| (cur - base).abs() > 0.5);
}

impl PowerPlayApp {
    pub fn reset_all_values(&mut self) {
        let gpus = self.gpus.clone();
        let entries = self.ioreg_entries.clone();
        let live_on = self.live_telemetry_on;
        let telem = self.telemetry_data.clone();
        let active_tab = self.active_tab;
        let advance_os_tab = self.advance_os_tab;
        *self = Self::default();
        self.gpus = gpus;
        self.ioreg_entries = entries;
        self.selected_gpu = usize::MAX;
        self.live_telemetry_on = live_on;
        self.telemetry_data = telem;
        self.active_tab = active_tab;
        self.advance_os_tab = advance_os_tab;
        self.baseline_gen = self.baseline_gen.wrapping_add(1);
    }

    pub fn refresh_ioreg_gpus(&mut self) {
        let entries = crate::macos_ioreg::scan_ioreg_gpus();
        self.selected_gpu = usize::MAX;
        if entries.is_empty() {
            self.ioreg_entries = Vec::new();
            self.gpus = vec!["[0] No AMD GPU detected".to_string()];
        } else {
            self.gpus = entries
                .iter()
                .enumerate()
                .map(|(i, e)| format!("[{}] {}", i, e.label))
                .collect();
            self.ioreg_entries = entries;
        }
    }

    pub fn select_ioreg_gpu(&mut self, idx: usize) {
        self.selected_gpu = idx;
        if let Some(entry) = self.ioreg_entries.get(idx).cloned() {
            if let Some(bytes) = entry.ppt_bytes {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                self.loaded_file = None;
                self.apply_ppt_bytes(bytes, &entry.label);
            } else {
                self.status_msg = format!("Selected {} (no PP_PhmSoftPowerPlayTable in IORegistry)", entry.label);
            }
        }
    }
}
