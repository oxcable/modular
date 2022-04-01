use std::sync::Arc;

use eurorack::{
    utils::{Duration, SchmittTrigger},
    Voltage, CV_VOLTS, GATE_THRESHOLD_VOLTS,
};
use module::*;
use portable_atomic::AtomicF32;
use widgets::{
    egui::{self, Align, Layout},
    jack::{self, Jack},
    knob::Knob,
    signal::SignalFlow,
};

#[derive(Default)]
pub struct Adsr {
    params: Arc<AdsrParams>,
}

impl Adsr {
    pub const GATE_IN: usize = 0;
    pub const CV_OUT: usize = 0;
}

impl Module for Adsr {
    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn params(&self) -> Option<&dyn Parameters> {
        Some(self.params.as_ref())
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit> {
        Box::new(AdsrUnit {
            params: self.params.clone(),
            trigger: SchmittTrigger::default(),
            state: State::Silent,
            samples_remaining: None,
            level: 0.0,
            step: 0.0,
        })
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(AdsrPanel(self.params.clone()))
    }
}

#[derive(Parameters)]
struct AdsrParams {
    attack: Duration,
    decay: Duration,
    sustain: AtomicF32,
    release: Duration,
}

impl Default for AdsrParams {
    fn default() -> Self {
        AdsrParams {
            attack: Duration::new(0.005),
            decay: Duration::new(0.1),
            sustain: AtomicF32::new(0.8),
            release: Duration::new(0.5),
        }
    }
}

struct AdsrUnit {
    params: Arc<AdsrParams>,
    trigger: SchmittTrigger,
    state: State,
    samples_remaining: Option<usize>,
    level: f32,
    step: f32,
}

impl AdsrUnit {
    fn attack(&mut self) {
        let attack = self.params.attack.to_samples();
        self.state = State::Attack;
        self.samples_remaining = Some(attack);
        self.step = (1.0 - self.level) / attack as f32;
    }

    fn decay(&mut self) {
        let decay = self.params.decay.to_samples();
        self.state = State::Decay;
        self.samples_remaining = Some(decay);
        self.step = (self.params.sustain.read() - self.level) / decay as f32;
    }

    fn sustain(&mut self) {
        self.state = State::Sustain;
        self.samples_remaining = None;
        self.level = self.params.sustain.read();
        self.step = 0.0;
    }

    fn release(&mut self) {
        let release = self.params.release.to_samples();
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

impl AudioUnit for AdsrUnit {
    fn reset(&mut self, sample_rate: usize) {
        self.params.attack.reset(sample_rate);
        self.params.decay.reset(sample_rate);
        self.params.release.reset(sample_rate);
    }

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        // Respond to input gate.
        let gate = inputs[Adsr::GATE_IN].unwrap_or(0.0);
        if self.trigger.detect(inputs[Adsr::GATE_IN].unwrap_or(0.0)) {
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
        outputs[Adsr::CV_OUT] = CV_VOLTS * self.level;
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

struct AdsrPanel(Arc<AdsrParams>);

impl Panel for AdsrPanel {
    fn width(&self) -> usize {
        4
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("ADSR");
        ui.add_space(20.0);
        ui.add(Knob::new(&self.0.attack).scale(0.75));
        ui.small("Attack");
        ui.add_space(10.0);
        ui.add(Knob::new(&self.0.decay).scale(0.75));
        ui.small("Decay");
        ui.add_space(10.0);
        ui.add(Knob::new(&self.0.sustain).scale(0.75));
        ui.small("Sustain");
        ui.add_space(10.0);
        ui.add(Knob::new(&self.0.release).scale(0.75));
        ui.small("Release");
        ui.add_space(10.0);
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            jack::outputs(ui, |ui| {
                ui.add(Jack::output(handle.output(Adsr::CV_OUT)));
            });
            ui.add(SignalFlow::join_vertical());
            ui.add(Jack::input(handle.input(Adsr::GATE_IN)));
            ui.label("Gate");
        });
    }
}
