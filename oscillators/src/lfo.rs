use std::f32::consts::PI;

use rack::{
    voltage::{Voltage, CV_VOLTS},
    Module, ModuleIO,
};

pub struct LFO {
    sample_rate: f32,
    frequency: f32,
    phase: f32,
}

impl ModuleIO for LFO {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 4;
}

impl LFO {
    pub const FREQ_IN: usize = 0;

    pub const SINE_OUT: usize = 0;
    pub const SAW_OUT: usize = 1;
    pub const SQUARE_OUT: usize = 2;
    pub const TRI_OUT: usize = 3;

    pub fn new(frequency: f32) -> Self {
        LFO {
            sample_rate: 0.0,
            frequency,
            phase: 0.0,
        }
    }
}

impl Module for LFO {
    fn reset(&mut self, sample_rate: usize) {
        self.sample_rate = sample_rate as f32;
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let freq = self.frequency + 20.0 * inputs[Self::FREQ_IN].unwrap_or(0.0) / CV_VOLTS;
        self.phase = (self.phase + freq / self.sample_rate) % 1.0;
        outputs[Self::SINE_OUT] = CV_VOLTS * ((2.0 * PI * self.phase).sin() + 1.0);
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
