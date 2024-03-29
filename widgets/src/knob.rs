use std::{f32::consts::PI, ops::RangeInclusive};

use egui::*;
use module::Parameter;

pub struct Knob<'a> {
    param: &'a dyn Parameter<Value = f32>,
    scale: f32,
    range: Range,
    snap_to_center: bool,
    hover_text: Box<dyn Fn(f32) -> String>,
}

impl<'a> Knob<'a> {
    pub fn new(param: &'a dyn Parameter<Value = f32>) -> Self {
        Knob {
            param,
            scale: 1.0,
            range: Range::Linear(0.0..=1.0),
            snap_to_center: false,
            hover_text: Box::new(|v| format!("{:0.3}", v)),
        }
    }

    pub fn attenuverter(param: &'a dyn Parameter<Value = f32>) -> Self {
        Knob::new(param)
            .range(-1.0..=1.0)
            .scale(0.5)
            .snap_to_center()
    }

    pub fn frequency(param: &'a dyn Parameter<Value = f32>) -> Self {
        Knob::new(param)
            .logarithmic(20.0..=20_000.0)
            .hover_text(|v| format!("{:.0} Hz", v))
    }

    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = Range::Linear(range);
        self
    }

    pub fn logarithmic(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = Range::Logarithmic(range);
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn snap_to_center(mut self) -> Self {
        self.snap_to_center = true;
        self
    }

    pub fn hover_text<F>(mut self, format: F) -> Self
    where
        F: 'static + Fn(f32) -> String,
    {
        self.hover_text = Box::new(format);
        self
    }
}

impl<'a> Widget for Knob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let radius = self.scale * 1.5 * ui.spacing().interact_size.y;
        let drag_height = 5.0 * radius;

        let desired_size = vec2(2.0 * radius, 2.0 * radius);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        let mut response = response.on_hover_cursor(CursorIcon::Grab);

        // Interact:
        let mut value = self.param.read();
        let mut normalized_value = self.range.to_normal(value);
        if response.dragged() {
            ui.output().cursor_icon = CursorIcon::Grabbing;
            let precise = ui.ctx().input().modifiers.shift;
            let delta = -response.drag_delta().y / drag_height * if precise { 0.1 } else { 1.0 };
            if delta != 0.0 {
                normalized_value = (normalized_value + delta).clamp(0.0, 1.0);
                if self.snap_to_center && !precise && value_near(normalized_value, 0.5) {
                    normalized_value = 0.5;
                }
                value = self.range.from_normal(normalized_value);
                self.param.write(value);
                response.mark_changed();
            }
        }

        if response.dragged() || response.hovered() {
            show_tooltip_for(ui.ctx(), Id::null(), &rect, |ui| {
                ui.small((self.hover_text)(value));
            });
        }

        // Draw:
        if ui.is_rect_visible(rect) {
            const LEFT_ANGLE: f32 = 0.75 * PI;
            const RIGHT_ANGLE: f32 = 2.25 * PI;
            const ARC_LENGTH: f32 = RIGHT_ANGLE - LEFT_ANGLE;

            let origin = rect.center();
            let painter = ui.painter();
            let widget = ui.style().interact(&response);

            // Knob background:
            painter.circle_filled(origin, radius, widget.bg_fill);

            // Selectable range:
            let mut arc_pts = Vec::new();
            for i in 0..=100 {
                let angle = LEFT_ANGLE + ARC_LENGTH * (i as f32 / 100.0);
                arc_pts.push(origin + radius * Vec2::angled(angle));
            }
            painter.add(Shape::line(arc_pts, widget.fg_stroke));

            // Snap points:
            if self.snap_to_center {
                let dir = vec2(0.0, -radius);
                painter.line_segment([origin + 0.75 * dir, origin + dir], widget.fg_stroke);
            }

            // Needle:
            let dir = radius * Vec2::angled(LEFT_ANGLE + ARC_LENGTH * normalized_value);
            let mut stroke = widget.fg_stroke;
            stroke.width = (3.0 * self.scale).max(1.5).max(stroke.width);
            painter.line_segment([origin + 0.4 * dir, origin + dir], stroke);
        }

        response
    }
}

#[derive(Clone, Debug)]
enum Range {
    Linear(RangeInclusive<f32>),
    Logarithmic(RangeInclusive<f32>),
}

impl Range {
    fn to_normal(&self, value: f32) -> f32 {
        match self {
            Self::Linear(range) => remap_clamp(value, range.clone(), 0.0..=1.0),
            Self::Logarithmic(range) => {
                remap_clamp(value, range.clone(), 1.0..=100.0).log10() / 2.0
            }
        }
    }

    fn from_normal(&self, normal: f32) -> f32 {
        match self {
            Self::Linear(range) => remap(normal, 0.0..=1.0, range.clone()),
            Self::Logarithmic(range) => remap(10f32.powf(2.0 * normal), 1.0..=100.0, range.clone()),
        }
    }
}

fn value_near(value: f32, target: f32) -> bool {
    (value - target).abs() < 0.015
}
