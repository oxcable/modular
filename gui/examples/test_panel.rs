use eframe::egui;

use gui::{jack::Jack, knob::Knob, ModularSynth, Panel};

struct TestPanel {
    knob1: f32,
    knob2: f32,
}

impl Default for TestPanel {
    fn default() -> Self {
        TestPanel {
            knob1: 0.5,
            knob2: 0.0,
        }
    }
}

impl Panel for TestPanel {
    fn width(&self) -> usize {
        12
    }

    fn update(&mut self, ui: &mut egui::Ui) {
        ui.heading("Test Panel");
        ui.add_space(20.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                ui.add(Knob::new(&mut self.knob1));
                ui.label("Knob 1");
                ui.small(format!("{:0.2}", self.knob1));
            });
            columns[1].vertical_centered(|ui| {
                ui.add(Knob::new(&mut self.knob2).range(-1.0..=1.0).scale(2.0));
                ui.label("Knob 2");
                ui.small(format!("{:0.2}", self.knob2));
            });
        });
        ui.add_space(20.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                ui.add(Jack::input());
                ui.label("Input");
            });
            columns[1].vertical_centered(|ui| {
                ui.add(Jack::output());
                ui.label("Output");
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(255.0, 515.0)),
        ..Default::default()
    };
    let app = ModularSynth::new(vec![Box::new(TestPanel::default())]);
    eframe::run_native(Box::new(app), options);
}
