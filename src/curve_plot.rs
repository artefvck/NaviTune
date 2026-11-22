use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub(crate) fn draw_fan_curve_plot(&mut self, ui: &mut egui::Ui) {
        let desired_size = egui::vec2(ui.available_width().max(260.0), 175.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(18, 23, 33));
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(52, 66, 92)));

        let plot = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 36.0, rect.top() + 14.0),
            egui::pos2(rect.right() - 14.0, rect.bottom() - 22.0),
        );
        if plot.width() <= 20.0 || plot.height() <= 20.0 { return; }

        let (t_min, t_max) = (25.0_f32, 105.0_f32);
        let to_screen = |temp: f32, pwm: f32| -> egui::Pos2 {
            let nx = ((temp - t_min) / (t_max - t_min)).clamp(0.0, 1.0);
            let ny = (pwm / 100.0).clamp(0.0, 1.0);
            egui::pos2(plot.left() + nx * plot.width(), plot.bottom() - ny * plot.height())
        };
        let from_screen = |pos: egui::Pos2| -> (i32, i32) {
            let nx = ((pos.x - plot.left()) / plot.width()).clamp(0.0, 1.0);
            let ny = ((plot.bottom() - pos.y) / plot.height()).clamp(0.0, 1.0);
            ((t_min + nx * (t_max - t_min)).round() as i32, (ny * 100.0).round() as i32)
        };

        for pct in [0, 25, 50, 75, 100] {
            let y = to_screen(t_min, pct as f32).y;
            painter.line_segment([egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)], egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 145, 185, 28)));
            painter.text(egui::pos2(rect.left() + 4.0, y), egui::Align2::LEFT_CENTER, format!("{}%", pct), egui::FontId::proportional(10.0), egui::Color32::from_rgb(145, 165, 195));
        }
        for deg in [30, 50, 70, 90, 105] {
            let x = to_screen(deg as f32, 0.0).x;
            painter.line_segment([egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())], egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 145, 185, 28)));
            painter.text(egui::pos2(x, rect.bottom() - 4.0), egui::Align2::CENTER_BOTTOM, format!("{}°C", deg), egui::FontId::proportional(10.0), egui::Color32::from_rgb(145, 165, 195));
        }

        let pwm_min = self.fan_pwm_min.clamp(0, 100);
        let tgt_t = self.fan_target_temp.clamp(45, 100);
        let has_zrpm = self.fan_zero_rpm_enable && (self.fan_stop_temp > 0 || self.fan_start_temp > 0);
        let (mut pts, mut handles) = (Vec::new(), Vec::new());

        if has_zrpm {
            let stop_t = self.fan_stop_temp.clamp(25, (tgt_t - 10).max(26));
            let start_t = self.fan_start_temp.clamp(stop_t + 2, (tgt_t - 4).max(stop_t + 2));
            let (p_idle, p_stop, p_start, p_tgt, p_max) = (to_screen(t_min, 0.0), to_screen(stop_t as f32, 0.0), to_screen(start_t as f32, pwm_min as f32), to_screen(tgt_t as f32, 75.0_f32.max(pwm_min as f32)), to_screen(t_max, 100.0));
            pts.extend_from_slice(&[p_idle, p_stop, p_start, p_tgt, p_max]);
            handles.extend([(0usize, p_stop, "Stop"), (1, p_start, "Start / PWM Min"), (2, p_tgt, "Target")]);
        } else {
            let (p_base, p_knee, p_tgt, p_max) = (to_screen(t_min, pwm_min as f32), to_screen(45.0, pwm_min as f32), to_screen(tgt_t as f32, 75.0_f32.max(pwm_min as f32)), to_screen(t_max, 100.0));
            pts.extend_from_slice(&[p_base, p_knee, p_tgt, p_max]);
            handles.extend([(1usize, p_knee, "PWM Min"), (2, p_tgt, "Target")]);
        }

        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], egui::Stroke::new(2.2, egui::Color32::from_rgb(72, 185, 255)));
        }

        let drag_id = egui::Id::new("fan_curve_active_handle");
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let nearest = handles.iter().filter(|(_, hp, _)| hp.distance(pos) <= 18.0).min_by(|a, b| a.1.distance(pos).partial_cmp(&b.1.distance(pos)).unwrap_or(std::cmp::Ordering::Equal)).map(|(id, _, _)| *id);
                ui.ctx().data_mut(|d| d.insert_temp(drag_id, nearest));
            }
        }
        if response.dragged() {
            let active: Option<usize> = ui.ctx().data(|d| d.get_temp(drag_id)).flatten();
            if let (Some(h_id), Some(pos)) = (active, response.interact_pointer_pos()) {
                let (new_t, new_pwm) = from_screen(pos);
                match h_id {
                    0 => self.fan_stop_temp = new_t.clamp(25, (self.fan_start_temp - 2).max(25)),
                    1 => {
                        if has_zrpm { self.fan_start_temp = new_t.clamp(self.fan_stop_temp + 2, (self.fan_target_temp - 4).max(self.fan_stop_temp + 2)); }
                        self.fan_pwm_min = new_pwm.clamp(0, 95);
                    }
                    2 => self.fan_target_temp = new_t.clamp((self.fan_start_temp + 4).max(45), 100),
                    _ => {}
                }
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
            }
        }
        if response.drag_stopped() { ui.ctx().data_mut(|d| d.insert_temp(drag_id, None::<usize>)); }

        for (_, hp, tag) in &handles {
            painter.circle_filled(*hp, 5.5, egui::Color32::from_rgb(255, 195, 70));
            painter.circle_stroke(*hp, 5.5, egui::Stroke::new(1.2, egui::Color32::from_rgb(18, 23, 33)));
            painter.text(egui::pos2(hp.x, hp.y - 9.0), egui::Align2::CENTER_BOTTOM, *tag, egui::FontId::proportional(10.0), egui::Color32::from_rgb(225, 235, 250));
        }
    }

    pub(crate) fn draw_vf_curve_plot(&mut self, ui: &mut egui::Ui) {
        let desired_size = egui::vec2(ui.available_width().max(260.0), 175.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(18, 23, 33));
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(52, 66, 92)));

        let plot = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 44.0, rect.top() + 14.0),
            egui::pos2(rect.right() - 16.0, rect.bottom() - 22.0),
        );
        if plot.width() <= 20.0 || plot.height() <= 20.0 { return; }

        let f_axis_min = 300.0_f32;
        let f_axis_max = (self.oc_gfx_clock[1].max(self.freq_gfx[1]).max(2200) as f32 + 100.0).min(3400.0);
        let (v_axis_min, v_axis_max) = (600.0_f32, 1250.0_f32);

        let to_screen = |mhz: f32, mv: f32| -> egui::Pos2 {
            let nx = ((mhz - f_axis_min) / (f_axis_max - f_axis_min)).clamp(0.0, 1.0);
            let ny = ((mv - v_axis_min) / (v_axis_max - v_axis_min)).clamp(0.0, 1.0);
            egui::pos2(plot.left() + nx * plot.width(), plot.bottom() - ny * plot.height())
        };
        let from_screen = |pos: egui::Pos2| -> (i32, i32) {
            let nx = ((pos.x - plot.left()) / plot.width()).clamp(0.0, 1.0);
            let ny = ((plot.bottom() - pos.y) / plot.height()).clamp(0.0, 1.0);
            ((f_axis_min + nx * (f_axis_max - f_axis_min)).round() as i32, (v_axis_min + ny * (v_axis_max - v_axis_min)).round() as i32)
        };

        for mv in [650, 800, 950, 1100, 1200] {
            let y = to_screen(f_axis_min, mv as f32).y;
            painter.line_segment([egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)], egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 145, 185, 28)));
            painter.text(egui::pos2(rect.left() + 4.0, y), egui::Align2::LEFT_CENTER, format!("{}mV", mv), egui::FontId::proportional(10.0), egui::Color32::from_rgb(145, 165, 195));
        }

        let f0 = self.freq_gfx[0].max(300) as f32;
        let f1 = self.freq_gfx[1].max(self.freq_gfx[0] + 100) as f32;
        let v0 = if self.volt_gfx[0] > 0 { self.volt_gfx[0] as f32 } else { 750.0 };
        let v1 = if self.volt_gfx[1] > 0 { self.volt_gfx[1] as f32 } else { 1125.0 };

        let mut curve_pts = Vec::with_capacity(29);
        for s in 0..=28 {
            let t = s as f32 / 28.0;
            let f = f0 + t * (f1 - f0);
            let quad_bias = 0.12_f32;
            let u = t * t * (1.0 + quad_bias) - t * quad_bias;
            let v = (v0 + u.clamp(0.0, 1.15) * (v1 - v0)).clamp(v_axis_min, v_axis_max);
            curve_pts.push(to_screen(f, v));
        }
        for w in curve_pts.windows(2) {
            painter.line_segment([w[0], w[1]], egui::Stroke::new(2.2, egui::Color32::from_rgb(95, 225, 150)));
        }

        let handles = [(0usize, to_screen(f0, v0), "Fmin / Vmin"), (1usize, to_screen(f1, v1), "Fmax / Vmax")];
        let drag_id = egui::Id::new("vf_curve_active_handle");
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let nearest = handles.iter().filter(|(_, hp, _)| hp.distance(pos) <= 18.0).min_by(|a, b| a.1.distance(pos).partial_cmp(&b.1.distance(pos)).unwrap_or(std::cmp::Ordering::Equal)).map(|(id, _, _)| *id);
                ui.ctx().data_mut(|d| d.insert_temp(drag_id, nearest));
            }
        }
        if response.dragged() {
            let active: Option<usize> = ui.ctx().data(|d| d.get_temp(drag_id)).flatten();
            if let (Some(h_id), Some(pos)) = (active, response.interact_pointer_pos()) {
                let (new_f, new_v) = from_screen(pos);
                if h_id == 0 {
                    self.freq_gfx[0] = new_f.clamp(300, (self.freq_gfx[1] - 50).max(300));
                    if self.volt_gfx[0] > 0 { self.volt_gfx[0] = new_v.clamp(650, self.volt_gfx[1].max(650)); }
                } else {
                    let ceiling = if self.oc_gfx_clock[1] > 0 { self.oc_gfx_clock[1] } else { 3200 };
                    self.freq_gfx[1] = new_f.clamp(self.freq_gfx[0] + 50, ceiling);
                    if self.volt_gfx[1] > 0 { self.volt_gfx[1] = new_v.clamp(self.volt_gfx[0].max(650), 1200); }
                }
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
            }
        }
        if response.drag_stopped() { ui.ctx().data_mut(|d| d.insert_temp(drag_id, None::<usize>)); }

        for (_, hp, tag) in &handles {
            painter.circle_filled(*hp, 5.5, egui::Color32::from_rgb(255, 195, 70));
            painter.circle_stroke(*hp, 5.5, egui::Stroke::new(1.2, egui::Color32::from_rgb(18, 23, 33)));
            painter.text(egui::pos2(hp.x, hp.y - 9.0), egui::Align2::CENTER_BOTTOM, *tag, egui::FontId::proportional(10.0), egui::Color32::from_rgb(225, 235, 250));
        }
    }
}
