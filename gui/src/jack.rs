use eframe::egui::*;

pub struct Jack {
    type_: JackType,
}

impl Jack {
    pub fn input() -> Self {
        Jack {
            type_: JackType::Input,
        }
    }

    pub fn output() -> Self {
        Jack {
            type_: JackType::Output,
        }
    }
}

impl Widget for Jack {
    fn ui(self, ui: &mut Ui) -> Response {
        let radius = ui.spacing().interact_size.y;
        let desired_size = radius * vec2(3.5, 2.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Draw:
        if ui.is_rect_visible(rect) {
            let origin = rect.center();
            let widget = ui.style().interact(&response);

            // Hexagon container:
            let mut hex_pts = Vec::new();
            for i in 0..=6 {
                let angle = i as f32 * std::f32::consts::FRAC_PI_3;
                let dir = radius * vec2(angle.cos(), angle.sin());
                hex_pts.push(origin + dir);
            }
            ui.painter().add(Shape::convex_polygon(
                hex_pts,
                widget.bg_fill,
                widget.fg_stroke,
            ));

            // Jack interior:
            ui.painter()
                .circle_stroke(origin, 0.60 * radius, widget.fg_stroke);
            ui.painter()
                .circle_filled(origin, 0.45 * radius, widget.fg_stroke.color);

            // Arrow:
            let arrow_dir = vec2(0.75*radius, 0.0);
            let arrow_origin = match self.type_ {
                JackType::Input => rect.left_center(),
                JackType::Output => rect.right_center() - arrow_dir,
            };
            ui.painter()
                .arrow(arrow_origin, arrow_dir, widget.fg_stroke);
        }

        response
    }
}

enum JackType {
    Input,
    Output,
}
