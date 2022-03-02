use rack::{
    voltage::{Voltage, AUDIO_VOLTS, V_OCT_F0},
    Module, ModuleIO,
};

pub struct VCO {
    phase: f32,
    phase_delta: f32,
}

impl ModuleIO for VCO {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 3;
}

impl VCO {
    pub const V_OCT_IN: usize = 0;

    pub const SAW_OUT: usize = 0;
    pub const SQUARE_OUT: usize = 1;
    pub const TRI_OUT: usize = 2;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        VCO {
            phase: 0.0,
            phase_delta: 0.0,
        }
    }
}

impl Module for VCO {
    fn reset(&mut self, sample_rate: usize) {
        self.phase_delta = V_OCT_F0 / sample_rate as f32;
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        // Compute the new waveform phase using the input voltage.
        let v_oct = inputs[Self::V_OCT_IN].unwrap_or(0.0);
        self.phase += self.phase_delta * 2f32.powf(v_oct);
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }

        outputs[Self::SAW_OUT] = AUDIO_VOLTS * (2.0 * self.phase - 1.0);
        outputs[Self::SQUARE_OUT] = if self.phase > 0.5 { AUDIO_VOLTS } else { 0.0 };
        outputs[Self::TRI_OUT] = if self.phase > 0.5 {
            AUDIO_VOLTS * (4.0 * self.phase - 1.0)
        } else {
            AUDIO_VOLTS * (1.0 - 4.0 * (self.phase - 0.5))
        };
    }
}
