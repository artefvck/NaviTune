use eframe::egui;

pub fn dynamic_box_w(_ui: &egui::Ui, min_w: f32, _cols: usize) -> f32 {
    min_w.max(86.0)
}

pub fn two_col_cards_adaptive(
    ui: &mut egui::Ui,
    id: &str,
    left_visible: bool,
    right_visible: bool,
    left: impl FnOnce(&mut egui::Ui),
    right: impl FnOnce(&mut egui::Ui),
) {
    match (left_visible, right_visible) {
        (true, true) => two_col_cards(ui, id, left, right),
        (true, false) => {
            ui.push_id(id, |ui| {
                left(ui);
            });
        }
        (false, true) => {
            ui.push_id(id, |ui| {
                right(ui);
            });
        }
        (false, false) => {}
    }
}

pub fn two_col_cards(
    ui: &mut egui::Ui,
    id: &str,
    left: impl FnOnce(&mut egui::Ui),
    right: impl FnOnce(&mut egui::Ui),
) {
    let gap = 10.0;
    let avail = (ui.available_width() - 6.0).max(260.0);
    if avail >= 620.0 {
        let col_w = ((avail - gap) * 0.5).floor();
        egui::Grid::new(id)
            .num_columns(2)
            .spacing([gap, 0.0])
            .min_col_width(col_w)
            .max_col_width(col_w)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    left(ui);
                });
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    right(ui);
                });
                ui.end_row();
            });
    } else {
        ui.vertical(|ui| {
            ui.set_width(avail);
            left(ui);
            right(ui);
        });
    }
}

pub fn draw_card<R>(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui) -> R) {
    let outer_w = ui.available_width();
    let inner_w = (outer_w - 30.0).max(180.0);
    ui.ctx().data_mut(|d| {
        d.insert_temp(egui::Id::new("cur_card_title"), title.to_string());
        d.insert_temp(egui::Id::new("cur_col_units"), (String::new(), String::new()));
        d.insert_temp(egui::Id::new("cur_box_idx"), 0usize);
    });
    crate::ui_theme::card_frame(ui).show(ui, |ui| {
        ui.set_width(inner_w);
        ui.set_max_width(inner_w);
        ui.spacing_mut().item_spacing.y = 5.0;
        ui.colored_label(
            crate::ui_theme::card_title_color(ui),
            egui::RichText::new(title).size(13.5).strong(),
        );
        ui.add_space(3.0);
        add_contents(ui);
    });
    ui.add_space(8.0);
}

pub fn label_with_var(ui: &mut egui::Ui, label: &str, var_tag: &str) {
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("cur_field_info"), (label.to_string(), var_tag.to_string())));
    let resp = ui.label(egui::RichText::new(label).size(12.5));
    if !var_tag.is_empty() {
        resp.on_hover_text(format!("{}\nStruct / Offset: {}", label, var_tag));
    }
}

pub fn kv_row_1(ui: &mut egui::Ui, label: &str, var_tag: &str, val: &mut i32, box_w: f32) {
    let w = dynamic_box_w(ui, box_w, 1);
    let unit = extract_unit_from_text(label);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.ctx().data_mut(|d| {
            d.insert_temp(egui::Id::new("cur_field_info"), (label.to_string(), var_tag.to_string()));
            d.insert_temp(egui::Id::new("cur_box_idx"), 0usize);
            d.insert_temp(egui::Id::new("cur_col_units"), (unit.to_string(), String::new()));
        });
        crate::num_box(ui, val, w);
        label_with_var(ui, label, var_tag);
    });
}

pub fn kv_row_2(ui: &mut egui::Ui, label: &str, var_tag: &str, pair: &mut [i32; 2], box_w: f32) {
    let w = dynamic_box_w(ui, box_w, 2);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.ctx().data_mut(|d| {
            d.insert_temp(egui::Id::new("cur_field_info"), (label.to_string(), var_tag.to_string()));
            d.insert_temp(egui::Id::new("cur_box_idx"), 0usize);
        });
        crate::num_box(ui, &mut pair[0], w);
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("cur_box_idx"), 1usize));
        crate::num_box(ui, &mut pair[1], w);
        ui.add_space(2.0);
        label_with_var(ui, label, var_tag);
    });
}

fn extract_unit_from_text(s: &str) -> &'static str {
    if s.contains("(°C)") { " °C" }
    else if s.contains("(MHz)") { " MHz" }
    else if s.contains("(mV)") { " mV" }
    else if s.contains("(W)") { " W" }
    else if s.contains("(A)") { " A" }
    else if s.contains("(RPM)") { " RPM" }
    else if s.contains("(%)") { " %" }
    else { "" }
}

pub fn kv_header_2(ui: &mut egui::Ui, h1: &str, h2: &str, box_w: f32) {
    let u1 = extract_unit_from_text(h1);
    let u2 = extract_unit_from_text(h2);
    if !u1.is_empty() || !u2.is_empty() || h2 == "Div" {
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("cur_col_units"), (u1.to_string(), u2.to_string())));
    }
    let w = dynamic_box_w(ui, box_w, 2);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for h in [h1, h2] {
            ui.allocate_ui_with_layout(
                egui::vec2(w, 16.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),

                |ui| {
                    ui.label(
                        egui::RichText::new(h)
                            .size(11.0)
                            .strong()
                            .color(egui::Color32::from_rgb(160, 182, 214)),
                    );
                },
            );
        }
    });
}

pub fn kv_header_1(ui: &mut egui::Ui, h1: &str, box_w: f32) {
    let w = dynamic_box_w(ui, box_w, 1);
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(w, 16.0),
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),

            |ui| {
                ui.label(
                    egui::RichText::new(h1)
                        .size(11.0)
                        .strong()
                        .color(egui::Color32::from_rgb(160, 182, 214)),
                );
            },
        );
    });
}

pub fn draw_check_row(ui: &mut egui::Ui, label: &str, var_tag: &str, checked: &mut bool) -> bool {
    let resp = ui.checkbox(checked, egui::RichText::new(label).size(12.5));
    if !var_tag.is_empty() {
        resp.clone().on_hover_text(var_tag);
    }
    resp.changed()
}

pub fn draw_check_grid(ui: &mut egui::Ui, id: &str, cols: usize, items: &mut [(&str, &str, &mut bool)]) {
    egui::Grid::new(id)
        .num_columns(cols)
        .spacing([18.0, 6.0])
        .show(ui, |ui| {
            for (idx, (label, tag, flag)) in items.iter_mut().enumerate() {
                draw_check_row(ui, label, tag, flag);
                if (idx + 1) % cols == 0 {
                    ui.end_row();
                }
            }
        });
}

pub fn draw_tracked_check(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    gen: u64,
    scope: &str,
    tag: &str,
    label: &str,
    flag: &mut bool,
) -> bool {
    let base_id = egui::Id::new((scope, gen, tag));
    let base_val = ctx.data_mut(|d| {
        if let Some(v) = d.get_temp::<bool>(base_id) {
            v
        } else {
            d.insert_temp(base_id, *flag);
            *flag
        }
    });
    let mut changed = false;
    ui.horizontal(|ui| {
        let resp = crate::ui_theme::green_checkbox(ui, flag, label);
        let tip = if *flag != base_val {
            format!("{} ({})\nCurrent: {}  |  Factory: {}", label, tag, *flag, base_val)
        } else {
            format!("{} ({})\nFactory Default: {}", label, tag, base_val)
        };
        if resp.on_hover_text(tip).changed() {
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
            changed = true;
        }
        if *flag != base_val {
            ui.colored_label(egui::Color32::from_rgb(240, 70, 90), "●");
        }
    });
    changed
}

pub fn draw_tracked_check_list(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    gen: u64,
    scope: &str,
    items: &mut [(&mut bool, &str, &str, bool)],
) {
    ui.spacing_mut().item_spacing.y = 4.0;
    for (flag, label, tag, supported) in items.iter_mut() {
        if !*supported {
            continue;
        }
        draw_tracked_check(ui, ctx, gen, scope, tag, label, flag);
    }
}

pub fn draw_arch_note_card(ui: &mut egui::Ui, title: &str, body: &str) {
    draw_card(ui, title, |ui| {
        ui.colored_label(egui::Color32::from_rgb(235, 190, 95), body);
    });
}

/// Safely formats a slice of DPM values into a compact summary string without panicking on short slices.
pub fn format_dpm_range(vals: &[i32]) -> String {
    match (vals.first(), vals.last()) {
        (Some(lo), Some(hi)) => format!("{} .. {}", lo, hi),
        _ => "N/A".to_string(),
    }
}
