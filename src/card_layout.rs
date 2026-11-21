use eframe::egui;

pub fn dynamic_box_w(ui: &egui::Ui, min_w: f32, cols: usize) -> f32 {
    let avail = ui.available_width();
    let target = if cols <= 1 {
        avail * 0.24
    } else {
        (avail * 0.44) / (cols as f32)
    };
    target.clamp(min_w.max(72.0), 116.0)
}

pub fn two_col_cards(
    ui: &mut egui::Ui,
    id: &str,
    left: impl FnOnce(&mut egui::Ui),
    right: impl FnOnce(&mut egui::Ui),
) {
    let gap = 12.0;
    let col_w = ((ui.available_width() - gap) * 0.5).floor().max(240.0);
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
}

pub fn draw_card<R>(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui) -> R) {
    let outer_w = ui.available_width();
    let inner_w = (outer_w - 28.0).max(180.0);
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("cur_card_title"), title.to_string()));
    crate::ui_theme::card_frame(ui).show(ui, |ui| {
        ui.set_width(inner_w);
        ui.set_max_width(inner_w);
        ui.spacing_mut().item_spacing.y = 5.0;
        ui.colored_label(
            crate::ui_theme::card_title_color(ui),
            egui::RichText::new(title).size(13.5).strong(),
        );
        ui.add_space(4.0);
        add_contents(ui);
    });
    ui.add_space(10.0);
}

pub fn label_with_var(ui: &mut egui::Ui, label: &str, var_tag: &str) {
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("cur_field_info"), (label.to_string(), var_tag.to_string())));
    let resp = ui.label(egui::RichText::new(label).size(12.5));
    if !var_tag.is_empty() {
        resp.on_hover_text(var_tag);
    }
}

pub fn kv_row_1(ui: &mut egui::Ui, label: &str, var_tag: &str, val: &mut i32, box_w: f32) {
    let w = dynamic_box_w(ui, box_w, 2);
    ui.horizontal(|ui| {
        label_with_var(ui, label, var_tag);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            crate::num_box(ui, val, w);
        });
    });
}

pub fn kv_row_2(ui: &mut egui::Ui, label: &str, var_tag: &str, pair: &mut [i32; 2], box_w: f32) {
    let w = dynamic_box_w(ui, box_w, 2);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        label_with_var(ui, label, var_tag);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            crate::num_box(ui, &mut pair[1], w);
            crate::num_box(ui, &mut pair[0], w);
        });
    });
}

pub fn kv_header_2(ui: &mut egui::Ui, h1: &str, h2: &str, box_w: f32) {
    let w = dynamic_box_w(ui, box_w, 2);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label("");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for h in [h2, h1] {
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
    });
}
