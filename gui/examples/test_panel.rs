use eframe::egui;

use atomic_float::AtomicF32;
use audio_host::AudioHost;
use gui::{jack::Jack, knob::Knob, ModularSynth};
use module::{ModuleHandle, Panel, Parameter};

struct TestPanel {
    knob1: AtomicF32,
    knob2: AtomicF32,
}

impl Default for TestPanel {
    fn default() -> Self {
        TestPanel {
            knob1: AtomicF32::new(0.0),
            knob2: AtomicF32::new(2.5),
        }
    }
}

impl Panel for TestPanel {
    fn width(&self) -> usize {
        12
    }

    fn update(&mut self, handle: &ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("Test Panel");
        ui.add_space(20.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                ui.add(Knob::attenuverter(&self.knob1));
                ui.label("Knob 1");
                ui.small(format!("{:0.2}", self.knob1.read()));
            });
            columns[1].vertical_centered(|ui| {
                ui.add(
                    Knob::new(&self.knob2)
                        .range(0.0..=5.0)
                        .scale(2.0)
                        .hover_text(|v| format!("Value: {:0.1}", v)),
                );
                ui.label("Knob 2");
                ui.small(format!("{:0.2}", self.knob2.read()));
            });
        });
        ui.add_space(20.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                ui.add(Jack::input(handle.input(0)));
                ui.add_space(5.0);
                ui.add(Jack::input(handle.input(1)));
                ui.label("Inputs");
            });
            columns[1].vertical_centered(|ui| {
                ui.add(Jack::output(handle.output(0)));
                ui.add_space(5.0);
                ui.add(Jack::output(handle.output(1)));
                ui.label("Outputs");
            });
        });
        egui::TopBottomPanel::bottom("dark_mode").show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                egui::global_dark_light_mode_switch(ui);
                ui.small(if ui.visuals().dark_mode {
                    "Dark mode"
                } else {
                    "Light mode"
                });
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(255.0, 540.0)),
        ..Default::default()
    };
    let app = ModularSynth::new(
        AudioHost::default(),
        vec![(ModuleHandle(0), Box::new(TestPanel::default()))],
    );
    eframe::run_native(Box::new(app), options);
}
