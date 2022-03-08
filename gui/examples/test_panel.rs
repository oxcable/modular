use eframe::egui;

use gui::{ModularSynth, Panel};

struct TestPanel {}

impl Panel for TestPanel {
    fn width(&self) -> usize {
        12
    }

    fn update(&mut self, ui: &mut egui::Ui) {
        ui.heading("Test Panel");
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(255.0, 515.0)),
        ..Default::default()
    };
    let app = ModularSynth::new(vec![Box::new(TestPanel {})]);
    eframe::run_native(Box::new(app), options);
}
