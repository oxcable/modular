use eframe::egui::*;
use eframe::epaint::QuadraticBezierShape;
use rack::{ModuleInput, ModuleOutput};

use crate::jack::JackInteraction;

pub(crate) struct Connections {
    connections: Vec<Connection>,
}

impl Connections {
    pub(crate) fn new() -> Self {
        Connections {
            connections: Vec::new(),
        }
    }

    pub(crate) fn update(&mut self, ui: &mut Ui) {
        // Handle any interactions from Jack widgets:
        let mut pending_source = None;
        if let Some(interaction) = JackInteraction::get(ui) {
            match interaction {
                JackInteraction::PendingInput(_, pos) => pending_source = Some(pos),
                JackInteraction::PendingOutput(_, pos) => pending_source = Some(pos),
                JackInteraction::CreateConnection(output, output_pos, input, input_pos) => {
                    self.connections.push(Connection {
                        _output: output,
                        _input: input,
                        output_pos,
                        input_pos,
                    });
                    JackInteraction::clear(ui);
                }
            }
        }

        // Draw existing connections:
        for c in &self.connections {
            Cable::new(c.output_pos, c.input_pos).draw(ui);
        }

        // Handle ongoing new connection:
        if let Some(src_pos) = pending_source {
            // Use <esc> to cancel new connection.
            if ui.ctx().input().key_pressed(Key::Escape) {
                JackInteraction::clear(ui);
            } else {
                let hover_pos = ui.ctx().input().pointer.hover_pos();
                if let Some(pos) = hover_pos {
                    Cable::new(src_pos, pos).draw(ui);
                }
            }
        }
    }
}

struct Cable {
    src: Pos2,
    dst: Pos2,
}

impl Cable {
    fn new(src: Pos2, dst: Pos2) -> Self {
        Cable { src, dst }
    }

    fn draw(self, ui: &mut Ui) {
        let stroke = if ui.visuals().dark_mode {
            Stroke::new(5.0, Color32::from_white_alpha(64))
        } else {
            Stroke::new(5.0, Color32::from_black_alpha(128))
        };

        let sag = vec2(0.0, 0.3 * self.src.distance(self.dst));
        let midpoint = self.src + 0.5 * (self.dst - self.src) + sag;

        ui.painter()
            .add(Shape::from(QuadraticBezierShape::from_points_stroke(
                [self.src, midpoint, self.dst],
                false,
                Color32::default(),
                stroke,
            )));
    }
}

#[derive(Copy, Clone, Debug)]
struct Connection {
    _output: ModuleOutput,
    _input: ModuleInput,
    output_pos: Pos2,
    input_pos: Pos2,
}
