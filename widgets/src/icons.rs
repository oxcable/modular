use std::f32::consts::PI;

use egui::*;

const ICON_SIZE: f32 = 18.0;

pub struct Icon {
    size: f32,
    type_: IconType,
}

impl Icon {
    pub fn sine_wave() -> Self {
        Icon {
            size: ICON_SIZE,
            type_: IconType::Sine,
        }
    }

    pub fn saw_wave() -> Self {
        Icon {
            size: ICON_SIZE,
            type_: IconType::Saw,
        }
    }

    pub fn square_wave() -> Self {
        Icon {
            size: ICON_SIZE,
            type_: IconType::Square,
        }
    }

    pub fn triangle_wave() -> Self {
        Icon {
            size: ICON_SIZE,
            type_: IconType::Triangle,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Widget for Icon {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = vec2(self.size, self.size);
        let (rect, response) = ui.allocate_exact_size(size, Sense::focusable_noninteractive());
        if ui.is_rect_visible(rect) {
            let origin = rect.left_center();
            let xmax = self.size;
            let ymax = self.size / 4.0;
            let pts = match self.type_ {
                IconType::Sine => sine_wave(origin, xmax, ymax),
                IconType::Saw => vec![
                    origin + vec2(0.0, ymax),
                    origin + vec2(self.size, -ymax),
                    origin + vec2(self.size, ymax),
                ],
                IconType::Square => vec![
                    origin + vec2(0.0, ymax),
                    origin + vec2(0.0, -ymax),
                    origin + vec2(0.5 * self.size, -ymax),
                    origin + vec2(0.5 * self.size, ymax),
                    origin + vec2(self.size, ymax),
                    origin + vec2(self.size, -ymax),
                ],
                IconType::Triangle => vec![
                    origin + vec2(0.0, 0.0),
                    origin + vec2(0.25 * self.size, -ymax),
                    origin + vec2(0.75 * self.size, ymax),
                    origin + vec2(self.size, 0.0),
                ],
            };
            ui.painter()
                .add(Shape::line(pts, ui.visuals().noninteractive().fg_stroke));
        }
        response
    }
}

fn sine_wave(origin: Pos2, xmax: f32, ymax: f32) -> Vec<Pos2> {
    let mut pts = Vec::new();
    for i in 0..=xmax as usize {
        let x = i as f32;
        let y = (2.0 * PI * i as f32 / xmax).sin();
        pts.push(origin + vec2(x, -ymax * y));
    }
    pts
}

#[derive(Copy, Clone, Debug)]
enum IconType {
    Sine,
    Saw,
    Square,
    Triangle,
}
