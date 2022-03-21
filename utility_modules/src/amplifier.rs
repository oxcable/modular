use std::sync::Arc;

use atomic_float::AtomicF32;
use eurorack::{Voltage, CV_VOLTS};
use module::{AudioUnit, Module, Panel, Parameter};
use widgets::{
    egui::{self, Align, Layout},
    jack::{self, Jack},
    knob::Knob,
    signal::SignalFlow,
};

#[derive(Default)]
pub struct Vca {
    params: Arc<VcaParams>,
}

impl Vca {
    pub const AUDIO_IN: usize = 0;
    pub const CV_IN: usize = 1;

    pub const AUDIO_OUT: usize = 0;
}

impl Module for Vca {
    fn inputs(&self) -> usize {
        2
    }

    fn outputs(&self) -> usize {
        1
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit + Send> {
        Box::new(VcaUnit(self.params.clone()))
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(VcaPanel(self.params.clone()))
    }
}

struct VcaParams {
    gain: AtomicF32,
    gain_atten: AtomicF32,
}

impl Default for VcaParams {
    fn default() -> Self {
        VcaParams {
            gain: AtomicF32::new(1.0),
            gain_atten: AtomicF32::new(0.0),
        }
    }
}

struct VcaUnit(Arc<VcaParams>);

impl AudioUnit for VcaUnit {
    fn reset(&mut self, _sample_rate: usize) {}

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let mut gain = self.0.gain.read();
        if let Some(cv_in) = inputs[Vca::CV_IN] {
            gain *= self.0.gain_atten.read() * cv_in / CV_VOLTS;
        }
        outputs[Vca::AUDIO_OUT] = gain * inputs[Vca::AUDIO_IN].unwrap_or(0.0);
    }
}

struct VcaPanel(Arc<VcaParams>);

impl Panel for VcaPanel {
    fn width(&self) -> usize {
        4
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("VCA");
        ui.add_space(20.0);
        ui.add(Knob::new(&self.0.gain).range(0.0..=1.0));
        ui.add(SignalFlow::down_arrow());
        ui.label("Gain");
        ui.add(SignalFlow::up_arrow());
        ui.add(Knob::attenuverter(&self.0.gain_atten));
        ui.add(SignalFlow::join_vertical());
        ui.add(Jack::input(handle.input(Vca::CV_IN)));
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            jack::outputs(ui, |ui| {
                ui.add(Jack::output(handle.output(Vca::AUDIO_OUT)));
            });
            ui.add(SignalFlow::join_vertical());
            ui.add(Jack::input(handle.input(Vca::AUDIO_IN)));
            ui.label("Audio");
        });
    }
}
