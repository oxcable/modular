use std::{f32::consts::PI, ops::RangeInclusive};

use eframe::egui::*;
use module::Parameter;

pub struct Knob<'a> {
    param: &'a dyn Parameter<Value = f32>,
    scale: f32,
    range: RangeInclusive<f32>,
}

impl<'a> Knob<'a> {
    pub fn new(param: &'a dyn Parameter<Value = f32>) -> Self {
        Knob {
            param,
            scale: 1.0,
            range: 0.0..=1.0,
        }
    }

    pub fn attenuverter(param: &'a dyn Parameter<Value = f32>) -> Self {
        Knob::new(param).range(-1.0..=1.0)
    }

    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

impl<'a> Widget for Knob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let radius = self.scale * ui.spacing().interact_size.y;
        let drag_height = 5.0 * radius;

        let desired_size = vec2(2.0 * radius, 2.0 * radius);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        let mut response = response.on_hover_cursor(CursorIcon::Grab);

        // Interact:
        let mut relative_value = remap_clamp(self.param.read(), self.range.clone(), 0.0..=1.0);
        if response.dragged() {
            ui.output().cursor_icon = CursorIcon::Grabbing;
            let delta = -response.drag_delta().y / drag_height;
            if delta != 0.0 {
                relative_value = (relative_value + delta).clamp(0.0, 1.0);
                self.param
                    .write(remap(relative_value, 0.0..=1.0, self.range.clone()));
                response.mark_changed();
            }
        }

        // Draw:
        if ui.is_rect_visible(rect) {
            const LEFT_ANGLE: f32 = 0.75 * PI;
            const RIGHT_ANGLE: f32 = 2.25 * PI;
            const ARC_LENGTH: f32 = RIGHT_ANGLE - LEFT_ANGLE;

            let origin = rect.center();
            let widget = ui.style().interact(&response);

            // Knob background:
            ui.painter().circle_filled(origin, radius, widget.bg_fill);

            // Selectable range:
            let mut arc_pts = Vec::new();
            for i in 0..=100 {
                let angle = LEFT_ANGLE + ARC_LENGTH * (i as f32 / 100.0);
                let dir = radius * vec2(angle.cos(), angle.sin());
                arc_pts.push(origin + dir);
            }
            ui.painter().add(Shape::line(arc_pts, widget.fg_stroke));

            // Needle:
            let angle = LEFT_ANGLE + ARC_LENGTH * relative_value;
            let dir = radius * vec2(angle.cos(), angle.sin());
            let stroke = Stroke::new(2.0 * self.scale, widget.fg_stroke.color);
            ui.painter()
                .line_segment([origin + 0.5 * dir, origin + dir], stroke);
        }

        response
    }
}
