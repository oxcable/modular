use eframe::egui;

use atomic_float::AtomicF32;
use audio_host::AudioHost;
use gui::ModularSynth;
use module::{ModuleHandle, Panel, Parameter};
use widgets::{
    icons::Icon,
    jack::{self, Jack},
    knob::Knob,
};

struct TestPanel {
    knob1: AtomicF32,
    knob2: AtomicF32,
}

impl Default for TestPanel {
    fn default() -> Self {
        TestPanel {
            knob1: AtomicF32::new(0.0),
            knob2: AtomicF32::new(1800.0),
        }
    }
}

impl Panel for TestPanel {
    fn width(&self) -> usize {
        10
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
                ui.add(Knob::frequency(&self.knob2));
                ui.label("Knob 2");
                ui.small(format!("{:.0} Hz", self.knob2.read()));
            });
        });
        ui.add_space(20.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                jack::outputs(ui, |ui| {
                    ui.add(Jack::output(handle.output(0)));
                    ui.add_space(5.0);
                    ui.add(Jack::output(handle.output(1)));
                    ui.label("Outputs");
                });
            });
            columns[1].vertical_centered(|ui| {
                jack::inputs(ui, |ui| {
                    ui.add(Jack::input(handle.input(0)));
                    ui.add_space(5.0);
                    ui.add(Jack::input(handle.input(1)));
                    ui.label("Inputs");
                });
            });
        });
        ui.add_space(20.0);
        ui.group(|ui| {
            ui.label("Icons");
            ui.columns(4, |columns| {
                columns[0].vertical_centered(|ui| ui.add(Icon::sine_wave()));
                columns[1].vertical_centered(|ui| ui.add(Icon::saw_wave()));
                columns[2].vertical_centered(|ui| ui.add(Icon::square_wave()));
                columns[3].vertical_centered(|ui| ui.add(Icon::triangle_wave()));
            });
        });

        egui::TopBottomPanel::bottom("dark_mode").show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                let mut debug_on_hover = ui.ctx().debug_on_hover();
                ui.checkbox(
                    &mut debug_on_hover,
                    egui::RichText::small("Debug mode".into()),
                );
                ui.ctx().set_debug_on_hover(debug_on_hover);

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
        initial_window_size: Some(egui::Vec2::new(215.0, 540.0)),
        ..Default::default()
    };
    let app = ModularSynth::new(
        AudioHost::default(),
        vec![(ModuleHandle(0), Box::new(TestPanel::default()))],
    );
    eframe::run_native(Box::new(app), options);
}
