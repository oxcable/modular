use eframe::egui;
use module::{ModuleHandle, Panel};

use crate::jack::Jack;

pub(crate) struct AudioOutputPanel;

impl Panel for AudioOutputPanel {
    fn width(&self) -> usize {
        5
    }

    fn update(&mut self, handle: &ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("Audio\nOut");
        ui.add_space(370.0);
        ui.add(Jack::input(handle.input(0)));
    }
}
