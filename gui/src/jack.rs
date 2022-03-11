use eframe::egui::*;
use module::{ModuleInput, ModuleOutput};

pub struct Jack {
    type_: JackType,
}

impl Jack {
    pub fn input(input: ModuleInput) -> Self {
        Jack {
            type_: JackType::Input(input),
        }
    }

    pub fn output(output: ModuleOutput) -> Self {
        Jack {
            type_: JackType::Output(output),
        }
    }
}

impl Widget for Jack {
    fn ui(self, ui: &mut Ui) -> Response {
        let radius = ui.spacing().interact_size.y;
        let desired_size = radius * vec2(3.5, 2.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Interact:
        let origin = rect.center();
        if response.clicked_by(PointerButton::Primary) {
            use JackInteraction::*;
            use JackType::*;
            match (self.type_, JackInteraction::get(ui)) {
                (Output(output), Some(PendingInput(input, input_pos))) => {
                    CreateConnection(output, origin, input, input_pos).update(ui);
                }
                (Input(input), Some(PendingOutput(output, output_pos))) => {
                    CreateConnection(output, output_pos, input, origin).update(ui);
                }
                (Output(output), None) => {
                    PendingOutput(output, origin).update(ui);
                }
                (Input(input), None) => {
                    PendingInput(input, origin).update(ui);
                }
                _ => (),
            }
        } else if response.clicked_by(PointerButton::Secondary)
            || (response.hovered() && ui.input().key_pressed(Key::Backspace))
        {
            use JackInteraction::*;
            match self.type_ {
                JackType::Output(output) => ClearOutput(output).update(ui),
                JackType::Input(input) => ClearInput(input).update(ui),
            }
        }

        // Draw:
        if ui.is_rect_visible(rect) {
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
            let arrow_dir = vec2(0.75 * radius, 0.0);
            let arrow_origin = match self.type_ {
                JackType::Input(_) => rect.left_center(),
                JackType::Output(_) => rect.right_center() - arrow_dir,
            };
            ui.painter()
                .arrow(arrow_origin, arrow_dir, widget.fg_stroke);
        }

        response
    }
}

#[derive(Copy, Clone, Debug)]
enum JackType {
    Input(ModuleInput),
    Output(ModuleOutput),
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum JackInteraction {
    PendingInput(ModuleInput, Pos2),
    PendingOutput(ModuleOutput, Pos2),
    CreateConnection(ModuleOutput, Pos2, ModuleInput, Pos2),
    ClearInput(ModuleInput),
    ClearOutput(ModuleOutput),
}

impl JackInteraction {
    pub(crate) fn get(ui: &Ui) -> Option<Self> {
        ui.memory().data.get_temp::<Self>(Id::null())
    }

    pub(crate) fn update(self, ui: &Ui) {
        ui.memory().data.insert_temp::<Self>(Id::null(), self);
    }

    pub(crate) fn clear(ui: &Ui) {
        ui.memory().data.remove::<Self>(Id::null());
    }
}
