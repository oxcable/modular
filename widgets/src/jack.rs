use std::{f32::consts::FRAC_PI_3, hash::Hash};

use egui::*;
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
        let desired_size = 2.0 * radius * vec2(1.0, FRAC_PI_3.sin());
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Update our position, for cable drawing:
        let origin = rect.center();
        match self.type_ {
            JackType::Input(input) => update_position(ui, input, origin),
            JackType::Output(output) => update_position(ui, output, origin),
        }

        // Interact:
        if response.clicked_by(PointerButton::Primary) {
            use JackInteraction::*;
            match (self.type_, JackInteraction::get(ui)) {
                (JackType::Output(output), Some(PendingInput(input)))
                | (JackType::Input(input), Some(PendingOutput(output))) => {
                    CreateConnection(output, input).update(ui);
                }
                (JackType::Output(output), None) => PendingOutput(output).update(ui),
                (JackType::Input(input), None) => PendingInput(input).update(ui),
                _ => (),
            }
        } else if response.clicked_by(PointerButton::Secondary)
            || (response.hovered() && ui.input().key_pressed(Key::Backspace))
        {
            match self.type_ {
                JackType::Output(output) => JackInteraction::ClearOutput(output).update(ui),
                JackType::Input(input) => JackInteraction::ClearInput(input).update(ui),
            }
        }

        // Draw:
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let widget = ui.style().interact(&response);

            // Hexagon container:
            let mut hex_pts = Vec::new();
            for i in 0..=6 {
                let angle = i as f32 * FRAC_PI_3;
                let dir = radius * vec2(angle.cos(), angle.sin());
                hex_pts.push(origin + dir);
            }
            painter.add(Shape::convex_polygon(
                hex_pts,
                widget.bg_fill,
                widget.fg_stroke,
            ));

            // Jack interior:
            painter.circle(
                origin,
                0.6 * radius,
                ui.visuals().faint_bg_color,
                widget.fg_stroke,
            );
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
pub enum JackInteraction {
    PendingInput(ModuleInput),
    PendingOutput(ModuleOutput),
    CreateConnection(ModuleOutput, ModuleInput),
    ClearInput(ModuleInput),
    ClearOutput(ModuleOutput),
}

impl JackInteraction {
    pub fn get(ui: &Ui) -> Option<Self> {
        ui.memory().data.get_temp::<Self>(Id::null())
    }

    pub fn update(self, ui: &Ui) {
        ui.memory().data.insert_temp::<Self>(Id::null(), self);
    }

    pub fn clear(ui: &Ui) {
        ui.memory().data.remove::<Self>(Id::null());
    }
}

fn update_position<T>(ui: &Ui, io: T, position: Pos2)
where
    T: Hash,
{
    let id = Id::new(io);
    ui.memory().data.remove::<Pos2>(id);
    ui.memory().data.insert_temp(id, position);
}
