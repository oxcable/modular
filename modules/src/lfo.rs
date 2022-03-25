use std::{f32::consts::PI, sync::Arc};

use atomic_float::AtomicF32;
use eurorack::{Voltage, CV_VOLTS};
use module::{AudioUnit, Module, Panel, Parameter};
use widgets::{
    egui::{self, Align, Layout},
    icons::Icon,
    jack::{self, Jack},
    knob::Knob,
};

#[derive(Default)]
pub struct Lfo {
    params: Arc<LfoParams>,
}

impl Lfo {
    pub const FREQ_IN: usize = 0;

    pub const SINE_OUT: usize = 0;
    pub const SAW_OUT: usize = 1;
    pub const SQUARE_OUT: usize = 2;
    pub const TRI_OUT: usize = 3;
}

impl Module for Lfo {
    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        4
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit> {
        Box::new(LfoUnit {
            params: self.params.clone(),
            sample_rate: 0.0,
            phase: 0.0,
        })
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

struct LfoUnit {
    params: Arc<LfoParams>,
    sample_rate: f32,
    phase: f32,
}

impl AudioUnit for LfoUnit {
    fn reset(&mut self, sample_rate: usize) {
        self.sample_rate = sample_rate as f32;
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let mut freq = self.params.frequency.read();
        freq += 20.0 * inputs[Lfo::FREQ_IN].unwrap_or(0.0) / CV_VOLTS;

        self.phase = (self.phase + freq / self.sample_rate) % 1.0;
        outputs[Lfo::SINE_OUT] = CV_VOLTS * ((2.0 * PI * self.phase).sin() + 1.0) / 2.0;

        outputs[Lfo::SAW_OUT] = CV_VOLTS * self.phase;
        outputs[Lfo::SQUARE_OUT] = CV_VOLTS * if self.phase < 0.5 { 1.0 } else { 0.0 };
        outputs[Lfo::TRI_OUT] = CV_VOLTS
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
        ui.add(Knob::new(&self.params.frequency).logarithmic(0.0..=100.0));
        ui.label("Freq");
        ui.add(Jack::input(handle.input(Lfo::FREQ_IN)));
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            jack::outputs(ui, |ui| {
                ui.add(Jack::output(handle.output(Lfo::TRI_OUT)));
                ui.add(Icon::triangle_wave());
                ui.add(Jack::output(handle.output(Lfo::SQUARE_OUT)));
                ui.add(Icon::square_wave());
                ui.add(Jack::output(handle.output(Lfo::SAW_OUT)));
                ui.add(Icon::saw_wave());
                ui.add(Jack::output(handle.output(Lfo::SINE_OUT)));
                ui.add(Icon::sine_wave());
            });
        });
    }
}
