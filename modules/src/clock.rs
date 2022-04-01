use std::sync::Arc;

use eurorack::{Voltage, CV_VOLTS};
use module::*;
use portable_atomic::AtomicF32;
use widgets::{
    egui::{self, Align, Layout},
    jack::{self, Jack},
    knob::Knob,
};

#[derive(Default)]
pub struct Clock {
    params: Arc<ClockParams>,
}

impl Clock {
    pub const TRIGGER_OUT: usize = 0;
}

impl Module for Clock {
    fn inputs(&self) -> usize {
        0
    }

    fn outputs(&self) -> usize {
        1
    }

    fn params(&self) -> Option<&dyn Parameters> {
        Some(self.params.as_ref())
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit> {
        Box::new(ClockUnit {
            params: self.params.clone(),
            sample_rate: None,
            ticks: 0,
        })
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(ClockPanel(self.params.clone()))
    }
}

#[derive(Parameters)]
struct ClockParams {
    bpm: AtomicF32,
    pulse_width: AtomicF32,
}

impl Default for ClockParams {
    fn default() -> Self {
        ClockParams {
            bpm: AtomicF32::new(120.0),
            pulse_width: AtomicF32::new(0.5),
        }
    }
}

struct ClockUnit {
    params: Arc<ClockParams>,
    sample_rate: Option<f32>,
    ticks: usize,
}

impl AudioUnit for ClockUnit {
    fn reset(&mut self, sample_rate: usize) {
        self.sample_rate = Some(sample_rate as f32);
    }

    fn tick(&mut self, _inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let sample_rate = self.sample_rate.expect("clock not initialized");
        let period = (60.0 / self.params.bpm.read() * sample_rate) as usize;
        let width = (self.params.pulse_width.read() * period as f32) as usize;

        self.ticks = (self.ticks + 1) % period;
        outputs[Clock::TRIGGER_OUT] = if self.ticks < width.clamp(1, period - 2) {
            CV_VOLTS
        } else {
            0.0
        };
    }
}

struct ClockPanel(Arc<ClockParams>);

impl Panel for ClockPanel {
    fn width(&self) -> usize {
        4
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("Clock");
        ui.add_space(20.0);
        ui.add(
            Knob::new(&self.0.bpm)
                .range(40.0..=200.0)
                .hover_text(|v| format!("{:.0} bpm", v)),
        );
        ui.label("BPM");
        ui.add_space(20.0);
        ui.add(
            Knob::new(&self.0.pulse_width)
                .scale(0.5)
                .range(0.0..=1.0)
                .snap_to_center(),
        );
        ui.small("Pulse Width");
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            jack::outputs(ui, |ui| {
                ui.add(Jack::output(handle.output(Clock::TRIGGER_OUT)));
            });
            ui.label("TRIG");
        });
    }
}
