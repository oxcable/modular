use eframe::egui::*;
use module::{ModuleInput, ModuleOutput};

pub fn inputs<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Frame::group(ui.style())
        .margin(style::Margin::symmetric(1.0, 5.0))
        .rounding(5.0)
        .show(ui, |ui| add_contents(ui))
}

pub fn outputs<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Frame::group(ui.style())
        .margin(style::Margin::symmetric(1.0, 5.0))
        .fill(ui.visuals().code_bg_color)
        .rounding(5.0)
        .show(ui, |ui| add_contents(ui))
}

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
        let radius = 0.75 * ui.spacing().interact_size.y;
        let desired_size = radius * vec2(2.0, 2.0);
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
            let painter = ui.painter();
            let widget = ui.style().interact(&response);

            // Hexagon container:
            let mut hex_pts = Vec::new();
            for i in 0..=6 {
                let angle = i as f32 * std::f32::consts::FRAC_PI_3;
                let dir = radius * vec2(angle.cos(), angle.sin());
                hex_pts.push(origin + dir);
            }
            painter.add(Shape::convex_polygon(
                hex_pts,
                widget.bg_fill,
                widget.fg_stroke,
            ));

            // Jack interior:
            painter.circle_stroke(origin, 0.6 * radius, widget.fg_stroke);
            painter.circle_filled(origin, 0.4 * radius, widget.fg_stroke.color);
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
