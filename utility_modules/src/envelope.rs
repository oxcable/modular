use rack::{
    utils::{Duration, SchmittTrigger},
    voltage::{Voltage, CV_VOLTS, GATE_THRESHOLD_VOLTS},
    Module, ModuleIO,
};

pub struct ADSR {
    attack: Duration,
    decay: Duration,
    sustain: f32,
    release: Duration,
    trigger: SchmittTrigger,
    state: State,
    samples_remaining: Option<usize>,
    level: f32,
    step: f32,
}

impl ModuleIO for ADSR {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 1;
}

impl ADSR {
    pub const GATE_IN: usize = 0;

    pub const CV_OUT: usize = 0;

    pub fn new(attack_secs: f32, decay_secs: f32, sustain: f32, release_secs: f32) -> Self {
        ADSR {
            attack: Duration::new(attack_secs),
            decay: Duration::new(decay_secs),
            sustain,
            release: Duration::new(release_secs),
            trigger: SchmittTrigger::default(),
            state: State::Silent,
            samples_remaining: None,
            level: 0.0,
            step: 0.0,
        }
    }

    fn attack(&mut self) {
        let attack = self.attack.samples();
        self.state = State::Attack;
        self.samples_remaining = Some(attack);
        self.step = (1.0 - self.level) / attack as f32;
    }

    fn decay(&mut self) {
        let decay = self.decay.samples();
        self.state = State::Decay;
        self.samples_remaining = Some(decay);
        self.step = (self.sustain - self.level) / decay as f32;
    }

    fn sustain(&mut self) {
        self.state = State::Sustain;
        self.samples_remaining = None;
        self.level = self.sustain;
        self.step = 0.0;
    }

    fn release(&mut self) {
        let release = self.release.samples();
        self.state = State::Release;
        self.samples_remaining = Some(release);
        self.step = -self.level / release as f32;
    }

    fn silence(&mut self) {
        self.state = State::Silent;
        self.samples_remaining = None;
        self.level = 0.0;
        self.step = 0.0;
    }
}

impl Default for ADSR {
    fn default() -> Self {
        ADSR::new(0.005, 0.1, 0.8, 0.5)
    }
}

impl Module for ADSR {
    fn reset(&mut self, sample_rate: usize) {
        self.attack.reset(sample_rate);
        self.decay.reset(sample_rate);
        self.release.reset(sample_rate);
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        // Respond to input gate.
        let gate = inputs[Self::GATE_IN].unwrap_or(0.0);
        if self.trigger.detect(inputs[Self::GATE_IN].unwrap_or(0.0)) {
            self.attack();
        } else if gate < GATE_THRESHOLD_VOLTS {
            match self.state {
                State::Attack | State::Decay | State::Sustain => self.release(),
                _ => (),
            }
        }

        // Tick current state.
        if let Some(samples) = self.samples_remaining {
            if samples == 0 {
                match self.state {
                    State::Attack => self.decay(),
                    State::Decay => self.sustain(),
                    State::Release => self.silence(),
                    _ => unreachable!(),
                }
            } else {
                self.samples_remaining = Some(samples - 1);
            }
        }

        // Compute final output.
        self.level += self.step;
        outputs[Self::CV_OUT] = CV_VOLTS * self.level;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    Attack,
    Decay,
    Sustain,
    Release,
    Silent,
}
