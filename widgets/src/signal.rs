use egui::*;

pub struct SignalFlow {
    direction: Direction,
    length: f32,
}

impl SignalFlow {
    pub fn join_vertical() -> Self {
        SignalFlow {
            direction: Direction::JoinVertical,
            length: 3.0,
        }
    }

    pub fn join_horizontal() -> Self {
        SignalFlow {
            direction: Direction::JoinHorizontal,
            length: 3.0,
        }
    }

    pub fn down_arrow() -> Self {
        SignalFlow {
            direction: Direction::DownArrow,
            length: 8.0,
        }
    }

    pub fn up_arrow() -> Self {
        SignalFlow {
            direction: Direction::UpArrow,
            length: 8.0,
        }
    }
}

impl Widget for SignalFlow {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(vec2(0.0, self.length), Sense::focusable_noninteractive());

        // Expand to consume any spacing added by egui, so that flow arrows can touch the actual
        // elements we're connecting.
        let rect = rect.expand2(ui.spacing().item_spacing);

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let stroke = ui.visuals().noninteractive().fg_stroke;
            match self.direction {
                Direction::JoinVertical => {
                    painter.line_segment([rect.center_bottom(), rect.center_top()], stroke);
                }
                Direction::JoinHorizontal => {
                    painter.line_segment([rect.left_center(), rect.right_center()], stroke);
                }
                Direction::UpArrow => {
                    let dir = vec2(0.0, -(self.length + ui.spacing().item_spacing.y));
                    painter.arrow(rect.center_bottom(), dir, stroke);
                }
                Direction::DownArrow => {
                    let dir = vec2(0.0, self.length + ui.spacing().item_spacing.y);
                    painter.arrow(rect.center_top(), dir, stroke);
                }
            };
        }
        response
    }
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    JoinVertical,
    JoinHorizontal,
    DownArrow,
    UpArrow,
}
