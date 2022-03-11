use std::{f32::consts::PI, sync::Arc};

use atomic_float::AtomicF32;
use eurorack::{Voltage, CV_VOLTS};
use module::{AudioUnit, Module, Panel, Parameter};
use widgets::{
    egui::{self, Align, Layout},
    jack::{self, Jack},
    knob::Knob,
};

#[derive(Default)]
pub struct Lfo {
    params: Arc<LfoParams>,
}

impl Module for Lfo {
    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        4
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit + Send> {
        Box::new(LfoUnit::with_params(self.params.clone()))
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(LfoPanel {
            params: self.params.clone(),
        })
    }
}

struct LfoParams {
    frequency: AtomicF32,
}

impl Default for LfoParams {
    fn default() -> Self {
        LfoParams {
            frequency: AtomicF32::new(1.0),
        }
    }
}

pub struct LfoUnit {
    params: Arc<LfoParams>,
    sample_rate: f32,
    phase: f32,
}

impl rack::ModuleIO for LfoUnit {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 4;
}

impl LfoUnit {
    pub const FREQ_IN: usize = 0;

    pub const SINE_OUT: usize = 0;
    pub const SAW_OUT: usize = 1;
    pub const SQUARE_OUT: usize = 2;
    pub const TRI_OUT: usize = 3;

    pub fn new(frequency: f32) -> Self {
        LfoUnit::with_params(Arc::new(LfoParams {
            frequency: AtomicF32::new(frequency),
        }))
    }

    fn with_params(params: Arc<LfoParams>) -> Self {
        LfoUnit {
            params,
            sample_rate: 0.0,
            phase: 0.0,
        }
    }
}

impl AudioUnit for LfoUnit {
    fn reset(&mut self, sample_rate: usize) {
        self.sample_rate = sample_rate as f32;
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let mut freq = self.params.frequency.read();
        freq += 20.0 * inputs[Self::FREQ_IN].unwrap_or(0.0) / CV_VOLTS;

        self.phase = (self.phase + freq / self.sample_rate) % 1.0;
        outputs[Self::SINE_OUT] = CV_VOLTS * ((2.0 * PI * self.phase).sin() + 1.0) / 2.0;
        outputs[Self::SAW_OUT] = CV_VOLTS * self.phase;
        outputs[Self::SQUARE_OUT] = CV_VOLTS * if self.phase < 0.5 { 1.0 } else { 0.0 };
        outputs[Self::TRI_OUT] = CV_VOLTS
            * if self.phase < 0.5 {
                2.0 * self.phase
            } else {
                1.0 - 2.0 * (self.phase - 0.5)
            };
    }
}

struct LfoPanel {
    params: Arc<LfoParams>,
}

impl Panel for LfoPanel {
    fn width(&self) -> usize {
        4
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("LFO");
        ui.add_space(20.0);
        ui.add(Knob::new(&self.params.frequency).range(0.0..=20.0));
        ui.label("Freq");
        ui.add(Jack::input(handle.input(LfoUnit::FREQ_IN)));
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            jack::outputs(ui, |ui| {
                ui.add(Jack::output(handle.output(LfoUnit::TRI_OUT)));
                ui.label("Tri");
                ui.add(Jack::output(handle.output(LfoUnit::SQUARE_OUT)));
                ui.label("Square");
                ui.add(Jack::output(handle.output(LfoUnit::SAW_OUT)));
                ui.label("Saw");
                ui.add(Jack::output(handle.output(LfoUnit::SINE_OUT)));
                ui.label("Sine");
            });
        });
    }
}
