use std::{collections::HashMap, f32::consts::PI, sync::Arc};

use eurorack::{Voltage, CV_VOLTS};
use module::*;
use portable_atomic::AtomicF32;
use widgets::{
    egui,
    jack::{self, Jack},
    knob::Knob,
    signal::SignalFlow,
};

#[derive(Default)]
pub struct Vcf {
    params: Arc<VcfParams>,
}

impl Vcf {
    pub const CUTOFF_IN: usize = 0;
    pub const RESONANCE_IN: usize = 1;
    pub const AUDIO_IN: usize = 2;

    pub const LOWPASS_OUT: usize = 0;
    pub const BANDPASS_OUT: usize = 1;
    pub const HIPASS_OUT: usize = 2;

    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Vcf {
            params: Arc::new(VcfParams {
                cutoff: AtomicF32::new(cutoff),
                cutoff_atten: AtomicF32::new(1.0),
                resonance: AtomicF32::new(resonance),
                resonance_atten: AtomicF32::new(1.0),
            }),
        }
    }
}

impl Module for Vcf {
    fn inputs(&self) -> usize {
        3
    }

    fn outputs(&self) -> usize {
        3
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit> {
        Box::new(VcfUnit {
            params: self.params.clone(),
            sample_rate: 0.0,
            last_out: [0.0; 3],
        })
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(VcfPanel(self.params.clone()))
    }

    fn serialize(&self) -> HashMap<String, SerializedParameter> {
        self.params.serialize()
    }

    fn deserialize(&self, params: &HashMap<String, SerializedParameter>) {
        self.params.deserialize(params);
    }
}

#[derive(Parameters)]
struct VcfParams {
    cutoff: AtomicF32,
    cutoff_atten: AtomicF32,
    resonance: AtomicF32,
    resonance_atten: AtomicF32,
}

impl Default for VcfParams {
    fn default() -> Self {
        VcfParams {
            cutoff: AtomicF32::new(20_000.0),
            cutoff_atten: AtomicF32::new(0.0),
            resonance: AtomicF32::new(1.0),
            resonance_atten: AtomicF32::new(0.0),
        }
    }
}

struct VcfUnit {
    params: Arc<VcfParams>,
    sample_rate: f32,
    last_out: [Voltage; 3],
}

impl AudioUnit for VcfUnit {
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
        let cutoff_in = inputs[Vcf::CUTOFF_IN].unwrap_or(0.0) / CV_VOLTS * self.sample_rate / 6.0;
        let resonance_in = inputs[Vcf::RESONANCE_IN].unwrap_or(0.0);

        let cutoff = (self.params.cutoff.read() + self.params.cutoff_atten.read() * cutoff_in)
            .clamp(0.0, self.sample_rate / 6.0);
        let resonance =
            self.params.resonance.read() + 0.5 * resonance_in * self.params.resonance_atten.read();

        // Implements a state variable multifilter.
        // Ref: DAFX, Section 2.2, pg 35
        let f1 = 2.0 * (PI * cutoff / self.sample_rate);
        let q1 = 1.0 / resonance;

        outputs[Vcf::HIPASS_OUT] = inputs[Vcf::AUDIO_IN].unwrap_or(0.0)
            - self.last_out[Vcf::LOWPASS_OUT]
            - q1 * self.last_out[Vcf::BANDPASS_OUT];
        outputs[Vcf::BANDPASS_OUT] =
            f1 * outputs[Vcf::HIPASS_OUT] + self.last_out[Vcf::BANDPASS_OUT];
        outputs[Vcf::LOWPASS_OUT] =
            f1 * outputs[Vcf::BANDPASS_OUT] + self.last_out[Vcf::LOWPASS_OUT];

        // Check for numerical stability in the filter. For now, we use an assert because I want to
        // bubble these up into crashes for testing. Eventually, this should be replaced with hard
        // clamping at the power limits.
        for o in outputs.iter() {
            assert!(o.is_finite());
        }

        self.last_out[Vcf::HIPASS_OUT] = outputs[Vcf::HIPASS_OUT];
        self.last_out[Vcf::BANDPASS_OUT] = outputs[Vcf::BANDPASS_OUT];
        self.last_out[Vcf::LOWPASS_OUT] = outputs[Vcf::LOWPASS_OUT];
    }
}

struct VcfPanel(Arc<VcfParams>);

impl Panel for VcfPanel {
    fn width(&self) -> usize {
        8
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("VCF");
        ui.add_space(20.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                ui.add(Knob::frequency(&self.0.cutoff));
                ui.add(SignalFlow::down_arrow());
                ui.small("Cutoff");
                ui.add(SignalFlow::up_arrow());
                ui.add(Knob::attenuverter(&self.0.cutoff_atten));
                ui.add(SignalFlow::join_vertical());
                ui.add(Jack::input(handle.input(Vcf::CUTOFF_IN)));
            });
            columns[1].vertical_centered(|ui| {
                ui.add(Knob::new(&self.0.resonance).range(0.5..=5.0));
                ui.add(SignalFlow::down_arrow());
                ui.small("Resonance");
                ui.add(SignalFlow::up_arrow());
                ui.add(Knob::attenuverter(&self.0.resonance_atten));
                ui.add(SignalFlow::join_vertical());
                ui.add(Jack::input(handle.input(Vcf::RESONANCE_IN)));
            });
        });
        ui.add_space(187.0);
        ui.add(Jack::input(handle.input(Vcf::AUDIO_IN)));
        ui.add(SignalFlow::join_vertical());
        jack::outputs(ui, |ui| {
            ui.columns(3, |columns| {
                columns[0].vertical_centered(|ui| {
                    ui.small("LO");
                    ui.add(Jack::output(handle.output(Vcf::LOWPASS_OUT)));
                });
                columns[1].vertical_centered(|ui| {
                    ui.small("BND");
                    ui.add(Jack::output(handle.output(Vcf::BANDPASS_OUT)));
                });
                columns[2].vertical_centered(|ui| {
                    ui.small("HI");
                    ui.add(Jack::output(handle.output(Vcf::HIPASS_OUT)));
                });
            });
        });
    }
}
