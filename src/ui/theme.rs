use crate::PowerPlayApp;
use eframe::egui::{self, Color32, Margin, Rounding, Shadow, Stroke};

pub const DEFAULT_ACCENT: Color32 = Color32::from_rgb(42, 130, 218);
pub const DEFAULT_GLASS_BASE: Color32 = Color32::from_rgb(35, 38, 41);
pub const DEFAULT_TEXT_COLOR: Color32 = Color32::from_rgb(239, 240, 241);
pub const DEFAULT_CHECK_COLOR: Color32 = Color32::from_rgb(61, 174, 233);

#[derive(Clone, Copy)]
pub struct ThemePreset {
    pub name: &'static str,
    pub alpha: f32,
    pub accent: Color32,
    pub base: Color32,
    pub text: Color32,
    pub check: Color32,
}

pub const THEME_PRESETS: &[ThemePreset] = &[
    ThemePreset {
        name: "Dark",
        alpha: 0.96,
        accent: Color32::from_rgb(42, 130, 218),
        base: Color32::from_rgb(35, 38, 41),
        text: Color32::from_rgb(239, 240, 241),
        check: Color32::from_rgb(61, 174, 233),
    },
    ThemePreset {
        name: "Light",
        alpha: 0.98,
        accent: Color32::from_rgb(0, 122, 255),
        base: Color32::from_rgb(242, 242, 247),
        text: Color32::from_rgb(28, 28, 30),
        check: Color32::from_rgb(0, 122, 255),
    },
    ThemePreset {
        name: "Dracula",
        alpha: 0.95,
        accent: Color32::from_rgb(189, 147, 249),
        base: Color32::from_rgb(40, 42, 54),
        text: Color32::from_rgb(248, 248, 242),
        check: Color32::from_rgb(80, 250, 123),
    },
    ThemePreset {
        name: "Nord",
        alpha: 0.95,
        accent: Color32::from_rgb(136, 192, 208),
        base: Color32::from_rgb(46, 52, 64),
        text: Color32::from_rgb(236, 239, 244),
        check: Color32::from_rgb(163, 190, 140),
    },
    ThemePreset {
        name: "Solarized Dark",
        alpha: 0.96,
        accent: Color32::from_rgb(38, 139, 210),
        base: Color32::from_rgb(0, 43, 54),
        text: Color32::from_rgb(238, 232, 213),
        check: Color32::from_rgb(133, 153, 0),
    },
];

fn is_light_base(c: Color32) -> bool {
    (c.r() as u16 + c.g() as u16 + c.b() as u16) > 380
}

fn shift_surface(c: Color32, delta: i16) -> Color32 {
    let apply = |ch: u8| -> u8 { (ch as i16 + delta).clamp(0, 255) as u8 };
    Color32::from_rgb(apply(c.r()), apply(c.g()), apply(c.b()))
}

fn theme_config_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(std::path::PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"));
    base.map(|d| d.join("navitune").join("theme.cfg"))
}

fn save_theme_config(alpha: f32, accent: Color32, base: Color32, text: Color32, check: Color32) {
    let Some(path) = theme_config_path() else { return; };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let [w, h] = load_saved_window_size();
    let data = format!(
        "alpha={:.4}\naccent={},{},{}\nbase={},{},{}\ntext={},{},{}\ncheck={},{},{}\nsize={:.0},{:.0}\n",
        alpha,
        accent.r(), accent.g(), accent.b(),
        base.r(), base.g(), base.b(),
        text.r(), text.g(), text.b(),
        check.r(), check.g(), check.b(),
        w, h,
    );
    let _ = std::fs::write(path, data);
}

fn load_theme_config() -> Option<(f32, Color32, Color32, Color32, Color32)> {
    let path = theme_config_path()?;
    let txt = std::fs::read_to_string(path).ok()?;
    let mut alpha = 0.96f32;
    let mut accent = DEFAULT_ACCENT;
    let mut base = DEFAULT_GLASS_BASE;
    let mut text = DEFAULT_TEXT_COLOR;
    let mut check = DEFAULT_CHECK_COLOR;
    let parse_rgb = |s: &str| -> Option<Color32> {
        let parts: Vec<u8> = s.split(',').filter_map(|x| x.trim().parse::<u8>().ok()).collect();
        (parts.len() == 3).then(|| Color32::from_rgb(parts[0], parts[1], parts[2]))
    };
    for line in txt.lines() {
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "alpha" => if let Ok(a) = v.trim().parse::<f32>() { alpha = a.clamp(0.35, 1.0); },
                "accent" => if let Some(c) = parse_rgb(v) { accent = c; },
                "base" => if let Some(c) = parse_rgb(v) { base = c; },
                "text" => if let Some(c) = parse_rgb(v) { text = c; },
                "check" => if let Some(c) = parse_rgb(v) { check = c; },
                _ => {}
            }
        }
    }
    Some((alpha, accent, base, text, check))
}

pub fn apply_modern_theme(ctx: &egui::Context) {
    apply_agave_fonts(ctx);
    use egui::{FontFamily, FontId, TextStyle};
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(14.5, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(12.5, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(12.5, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
    ].into();
    ctx.set_style(style);

    if let Some((alpha, accent, base, text, check)) = load_theme_config() {
        apply_full_theme(ctx, alpha, accent, base, text, check);
    } else {
        let p = THEME_PRESETS[0];
        apply_full_theme(ctx, p.alpha, p.accent, p.base, p.text, p.check);
    }
}

pub fn apply_full_theme(
    ctx: &egui::Context,
    alpha: f32,
    accent: Color32,
    base: Color32,
    text: Color32,
    check: Color32,
) {
    let a = alpha.clamp(0.35, 1.0);
    let light = is_light_base(base);
    let a_win = (a * 250.0) as u8;
    let a_card = (a * 225.0) as u8;
    let card_rgb = if light { Color32::WHITE } else { shift_surface(base, 12) };
    let ctrl_rgb = if light { shift_surface(base, -12) } else { shift_surface(base, 20) };
    let hover_rgb = if light { shift_surface(base, -24) } else { shift_surface(base, 36) };
    let border_col = if light {
        Color32::from_rgba_unmultiplied(180, 185, 195, 190)
    } else {
        Color32::from_rgba_unmultiplied(110, 122, 138, 120)
    };

    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new("tahoe_glass_alpha"), a);
        d.insert_temp(egui::Id::new("tahoe_accent_color"), accent);
        d.insert_temp(egui::Id::new("tahoe_base_color"), base);
        d.insert_temp(egui::Id::new("tahoe_text_color"), text);
        d.insert_temp(egui::Id::new("tahoe_check_color"), check);
    });
    save_theme_config(a, accent, base, text, check);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin = Margin::same(12.0);

    let r = Rounding::same(6.0_f32);
    let mut v = if light { egui::Visuals::light() } else { egui::Visuals::dark() };

    v.override_text_color = Some(text);
    v.window_fill = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), a_win);
    v.panel_fill = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), a_win);
    v.faint_bg_color = if light {
        Color32::from_rgba_unmultiplied(0, 0, 0, 8)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 10)
    };
    v.extreme_bg_color = if light {
        Color32::from_rgba_unmultiplied(255, 255, 255, 240)
    } else {
        shift_surface(base, -8)
    };

    v.widgets.noninteractive.bg_fill = Color32::from_rgba_unmultiplied(card_rgb.r(), card_rgb.g(), card_rgb.b(), a_card);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border_col);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text);
    v.widgets.noninteractive.rounding = r;

    v.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(ctrl_rgb.r(), ctrl_rgb.g(), ctrl_rgb.b(), 220);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgba_unmultiplied(ctrl_rgb.r(), ctrl_rgb.g(), ctrl_rgb.b(), 220);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border_col);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, text);
    v.widgets.inactive.rounding = r;

    v.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(hover_rgb.r(), hover_rgb.g(), hover_rgb.b(), 235);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgba_unmultiplied(hover_rgb.r(), hover_rgb.g(), hover_rgb.b(), 235);
    v.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, accent);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, text);
    v.widgets.hovered.rounding = r;

    v.widgets.active.bg_fill = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 240);
    v.widgets.active.weak_bg_fill = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 240);
    v.widgets.active.bg_stroke = Stroke::new(1.5_f32, accent);
    v.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
    v.widgets.active.rounding = r;

    v.selection.bg_fill = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 220);
    v.selection.stroke = Stroke::new(1.5_f32, accent);
    v.hyperlink_color = accent;
    v.window_rounding = r;
    v.window_stroke = Stroke::new(1.0_f32, border_col);
    v.window_shadow = Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur: 14.0,
        spread: 0.0,
        color: Color32::from_black_alpha(if light { 40 } else { 110 }),
    };

    style.visuals = v;
    ctx.set_style(style);
}

pub fn card_title_color(ui: &egui::Ui) -> Color32 {
    ui.ctx().data(|d| d.get_temp::<Color32>(egui::Id::new("tahoe_accent_color")).unwrap_or(DEFAULT_ACCENT))
}

pub fn card_frame(ui: &egui::Ui) -> egui::Frame {
    let card_r = Rounding::same(10.0_f32);
    let alpha = ui.ctx().data(|d| d.get_temp::<f32>(egui::Id::new("tahoe_glass_alpha")).unwrap_or(0.96));
    let base = ui.ctx().data(|d| d.get_temp::<Color32>(egui::Id::new("tahoe_base_color")).unwrap_or(DEFAULT_GLASS_BASE));
    let light = is_light_base(base);
    let card_rgb = if light { Color32::WHITE } else { shift_surface(base, 12) };
    let border_col = if light {
        Color32::from_rgba_unmultiplied(190, 195, 205, 130)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 22)
    };
    let fill_a = (alpha.clamp(0.35, 1.0) * 225.0) as u8;

    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(card_rgb.r(), card_rgb.g(), card_rgb.b(), fill_a))
        .stroke(Stroke::new(1.0_f32, border_col))
        .rounding(card_r)
        .inner_margin(Margin::same(12.0))
        .shadow(Shadow {
            offset: egui::vec2(0.0, 3.0),
            blur: 10.0,
            spread: 0.0,
            color: Color32::from_black_alpha(if light { 28 } else { 72 }),
        })
}

pub fn green_checkbox(ui: &mut egui::Ui, checked: &mut bool, text: impl Into<egui::WidgetText>) -> egui::Response {
    let check = ui.ctx().data(|d| d.get_temp::<Color32>(egui::Id::new("tahoe_check_color")).unwrap_or(DEFAULT_CHECK_COLOR));
    let txt = ui.ctx().data(|d| d.get_temp::<Color32>(egui::Id::new("tahoe_text_color")).unwrap_or(DEFAULT_TEXT_COLOR));
    ui.scope(|ui| {
        let v = ui.visuals_mut();
        v.widgets.inactive.fg_stroke = Stroke::new(2.0_f32, check);
        v.widgets.hovered.fg_stroke = Stroke::new(2.0_f32, shift_surface(check, 25));
        v.widgets.active.fg_stroke = Stroke::new(2.0_f32, check);
        v.override_text_color = Some(txt);
        ui.checkbox(checked, text)
    }).inner
}

impl PowerPlayApp {
    pub fn show_error(&mut self, title: &str, msg: &str) {
        log_debug("ERROR", &format!("{}: {}", title, msg));
        self.modal_error_title = title.to_string();
        self.modal_error_msg = msg.to_string();
        self.show_error_modal = true;
    }

    pub fn render_error_modal(&mut self, ctx: &egui::Context) {
        if !self.show_error_modal { return; }
        let mut open = true;
        egui::Window::new(&self.modal_error_title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(&self.modal_error_msg);
                ui.add_space(8.0);
                if ui.button("OK").clicked() { self.show_error_modal = false; }
            });
        if !open { self.show_error_modal = false; }
    }
}

pub fn debug_log_path() -> Option<std::path::PathBuf> {
    theme_config_path().and_then(|p| p.parent().map(|d| d.join("debug.log")))
}

fn format_utc_now() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let yr = if mo <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}Z", yr, mo, d, h, m, s)
}

pub fn log_debug(tag: &str, msg: &str) {
    let Some(path) = debug_log_path() else { return; };
    if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > 512 * 1024 {
            let _ = std::fs::write(&path, "");
        }
    }
    use std::io::Write;
    let ts = format_utc_now();
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "[{}] [{:<8}] {}", ts, tag, msg);
    }
}

pub fn load_saved_window_size() -> [f32; 2] {
    let Some(path) = theme_config_path() else { return [957.0, 653.0]; };
    let Ok(txt) = std::fs::read_to_string(path) else { return [957.0, 653.0]; };
    for line in txt.lines() {
        if let Some(("size", v)) = line.split_once('=') {
            let parts: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect();
            if parts.len() == 2 && parts[0] >= 580.0 && parts[1] >= 500.0 {
                return [parts[0], parts[1]];
            }
        }
    }
    [957.0, 653.0]
}

pub fn save_window_size(ctx: &egui::Context, size: egui::Vec2) {
    if size.x < 580.0 || size.y < 500.0 { return; }
    let prev = ctx.data(|d| d.get_temp::<[i32; 2]>(egui::Id::new("saved_win_wh")).unwrap_or([0, 0]));
    let cur = [size.x.round() as i32, size.y.round() as i32];
    if prev == cur { return; }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("saved_win_wh"), cur));
    let Some(path) = theme_config_path() else { return; };
    if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().filter(|l| !l.starts_with("size=")).map(String::from).collect();
    lines.push(format!("size={},{}", cur[0], cur[1]));
    let _ = std::fs::write(path, lines.join("\n") + "\n");
}


pub fn apply_agave_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "agave_propo".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/AgaveNerdFontPropo-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "agave_mono".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/AgaveNerdFontMono-Regular.ttf")).into(),
    );
    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "agave_propo".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "agave_mono".to_owned());
    ctx.set_fonts(fonts);
}
