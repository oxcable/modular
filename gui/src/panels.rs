use eframe::egui;
use module::{ModuleHandle, Panel};

use crate::jack::{self, Jack};

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
                ui.label("Out");
            });
        });
    }
}
