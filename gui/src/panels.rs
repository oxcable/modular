use eframe::egui;
use module::{ModuleHandle, Panel};

use widgets::jack::{self, Jack};

const HP_PIXELS: usize = 20;
const PANEL_HEIGHT: usize = 25 * HP_PIXELS;

pub(crate) fn panel_to_widget<'a>(
    handle: &'a ModuleHandle,
    panel: &'a mut dyn Panel,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let width = HP_PIXELS * panel.width();
        let desired_size = egui::vec2(width as f32, PANEL_HEIGHT as f32);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                rect,
                10.0,
                ui.visuals().faint_bg_color,
                ui.visuals().noninteractive().bg_stroke,
            );
            let mut panel_ui = ui.child_ui(
                rect.shrink(10.0),
                egui::Layout::top_down(egui::Align::Center),
            );
            panel.update(handle, &mut panel_ui);
        }

        response
    }
}

pub(crate) struct AudioOutputPanel;

impl Panel for AudioOutputPanel {
    fn width(&self) -> usize {
        4
    }

    fn update(&mut self, handle: &ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("Audio");
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            jack::inputs(ui, |ui| {
                ui.add(Jack::input(handle.input(0)));
            });
            ui.label("Out");
        });
    }
}
