use rack::{
    voltage::{Voltage, CV_VOLTS},
    Module, ModuleIO,
};

pub struct VCA {
    gain: f32,
}

impl ModuleIO for VCA {
    const INPUTS: usize = 2;
    const OUTPUTS: usize = 1;
}

impl VCA {
    pub const AUDIO_IN: usize = 0;
    pub const CV_IN: usize = 1;

    pub const AUDIO_OUT: usize = 0;

    fn new(gain_db: f32) -> Self {
        VCA {
            gain: 10f32.powf(gain_db / 20.0),
        }
    }
}

impl Default for VCA {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Module for VCA {
    fn reset(&mut self, _sample_rate: usize) {}

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let gain = self.gain * inputs[Self::CV_IN].unwrap_or(0.0) / CV_VOLTS;
        outputs[Self::AUDIO_OUT] = gain * inputs[Self::AUDIO_IN].unwrap_or(0.0);
    }
}
