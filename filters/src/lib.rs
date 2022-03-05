use std::f32::consts::PI;

use rack::{
    voltage::{Voltage, CV_VOLTS},
    Module, ModuleIO,
};

pub struct VCF {
    sample_rate: f32,
    cutoff: f32,
    resonance: f32,
    last_out: [Voltage; 3],
}

impl ModuleIO for VCF {
    const INPUTS: usize = 3;
    const OUTPUTS: usize = 3;
}

impl VCF {
    pub const CUTOFF_IN: usize = 0;
    pub const RESONANCE_IN: usize = 1;
    pub const AUDIO_IN: usize = 2;

    pub const LOWPASS_OUT: usize = 0;
    pub const BANDPASS_OUT: usize = 1;
    pub const HIPASS_OUT: usize = 2;

    #[allow(clippy::new_without_default)]
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        VCF {
            sample_rate: 0.0,
            cutoff,
            resonance,
            last_out: [0.0; 3],
        }
    }
}

impl Module for VCF {
    fn reset(&mut self, sample_rate: usize) {
        self.sample_rate = sample_rate as f32;
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        // Compute modulated inputs.
        //
        // As state variable filters become unstable somewhere around Fs/6, we manually clamp the
        // max cutoff there. We could raise this limit by oversampling (and we probably should do
        // so later).
        //
        // Additionally, the current resonance mapping is arbitrary and could use more tuning.
        let cutoff_in = inputs[Self::CUTOFF_IN].unwrap_or(0.0) / CV_VOLTS * self.sample_rate / 6.0;
        let cutoff = (self.cutoff + cutoff_in).clamp(0.0, self.sample_rate / 6.0);
        let resonance = self.resonance + 0.5 * inputs[Self::RESONANCE_IN].unwrap_or(0.0);

        // Implements a state variable multifilter.
        // Ref: DAFX, Section 2.2, pg 35
        let f1 = 2.0 * (PI * cutoff / self.sample_rate);
        let q1 = 1.0 / resonance;

        outputs[Self::HIPASS_OUT] = inputs[Self::AUDIO_IN].unwrap_or(0.0)
            - self.last_out[Self::LOWPASS_OUT]
            - q1 * self.last_out[Self::BANDPASS_OUT];
        outputs[Self::BANDPASS_OUT] =
            f1 * outputs[Self::HIPASS_OUT] + self.last_out[Self::BANDPASS_OUT];
        outputs[Self::LOWPASS_OUT] =
            f1 * outputs[Self::BANDPASS_OUT] + self.last_out[Self::LOWPASS_OUT];

        // Check for numerical stability in the filter. For now, we use an assert because I want to
        // bubble these up into crashes for testing. Eventually, this should be replaced with hard
        // clamping at the power limits.
        for o in outputs.iter() {
            assert!(o.is_finite());
        }

        self.last_out[Self::HIPASS_OUT] = outputs[Self::HIPASS_OUT];
        self.last_out[Self::BANDPASS_OUT] = outputs[Self::BANDPASS_OUT];
        self.last_out[Self::LOWPASS_OUT] = outputs[Self::LOWPASS_OUT];
    }
}
