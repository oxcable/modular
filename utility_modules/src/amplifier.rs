use std::sync::Arc;

use atomic_float::AtomicF32;
use eurorack::{Voltage, CV_VOLTS};
use module::{AudioUnit, Module, Panel, Parameter};
use rack::ModuleIO;
use widgets::{
    egui::{self, Align, Layout},
    jack::{self, Jack},
    knob::Knob,
};

#[derive(Default)]
pub struct Vca {
    params: Arc<VcaParams>,
}

impl Module for Vca {
    fn inputs(&self) -> usize {
        VcaUnit::INPUTS
    }

    fn outputs(&self) -> usize {
        VcaUnit::OUTPUTS
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

pub struct VcaUnit(Arc<VcaParams>);

impl ModuleIO for VcaUnit {
    const INPUTS: usize = 2;
    const OUTPUTS: usize = 1;
}

impl VcaUnit {
    pub const AUDIO_IN: usize = 0;
    pub const CV_IN: usize = 1;

    pub const AUDIO_OUT: usize = 0;

    fn new(gain_db: f32) -> Self {
        VcaUnit(Arc::new(VcaParams {
            gain: AtomicF32::new(10f32.powf(gain_db / 20.0)),
            gain_atten: AtomicF32::new(0.0),
        }))
    }
}

impl Default for VcaUnit {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl AudioUnit for VcaUnit {
    fn reset(&mut self, _sample_rate: usize) {}

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let mut gain = self.0.gain.read();
        if let Some(cv_in) = inputs[Self::CV_IN] {
            gain *= self.0.gain_atten.read() * cv_in / CV_VOLTS;
        }
        outputs[Self::AUDIO_OUT] = gain * inputs[Self::AUDIO_IN].unwrap_or(0.0);
    }
}

pub struct VcaPanel(Arc<VcaParams>);

impl Panel for VcaPanel {
    fn width(&self) -> usize {
        4
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("VCA");
        ui.add_space(20.0);
        ui.add(Knob::new(&self.0.gain).range(0.0..=1.0));
        ui.label("Gain");
        ui.add(Knob::attenuverter(&self.0.gain_atten));
        ui.add(Jack::input(handle.input(VcaUnit::CV_IN)));
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            jack::outputs(ui, |ui| {
                ui.add(Jack::output(handle.output(VcaUnit::AUDIO_OUT)));
                ui.label("Out");
            });
            jack::inputs(ui, |ui| {
                ui.add(Jack::input(handle.input(VcaUnit::AUDIO_IN)));
                ui.label("In");
            });
        });
    }
}
