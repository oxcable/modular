use std::sync::{atomic::AtomicU8, Arc};

use eurorack::{midi_to_voltage, utils::SchmittTrigger, Voltage};
use module::{AudioUnit, Module, Panel, Parameter};
use widgets::{
    egui::{self, Layout, Slider},
    jack::{self, Jack},
    signal::SignalFlow,
};

pub const SEQUENCE_LENGTH: usize = 8;

#[derive(Default)]
pub struct Sequencer {
    params: Arc<SequencerParams>,
}

impl Module for Sequencer {
    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit + Send> {
        Box::new(SequencerUnit {
            params: self.params.clone(),
            trigger: SchmittTrigger::default(),
            position: SEQUENCE_LENGTH - 1,
        })
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(SequencerPanel(self.params.clone()))
    }
}

struct SequencerParams {
    notes: [AtomicU8; SEQUENCE_LENGTH],
}

impl Default for SequencerParams {
    fn default() -> Self {
        SequencerParams {
            notes: [
                69.into(),
                69.into(),
                69.into(),
                69.into(),
                69.into(),
                69.into(),
                69.into(),
                69.into(),
            ],
        }
    }
}

pub struct SequencerUnit {
    params: Arc<SequencerParams>,
    trigger: SchmittTrigger,
    position: usize,
}

impl rack::ModuleIO for SequencerUnit {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 1;
}

impl SequencerUnit {
    pub const TRIGGER_IN: usize = 0;

    pub const V_OCT_OUT: usize = 0;

    pub fn new(notes: [u8; SEQUENCE_LENGTH]) -> Self {
        let mut atomic_notes = <[AtomicU8; 8]>::default();
        for (i, n) in notes.into_iter().enumerate() {
            atomic_notes[i] = AtomicU8::new(n);
        }

        SequencerUnit {
            params: Arc::new(SequencerParams {
                notes: atomic_notes,
            }),
            trigger: SchmittTrigger::default(),
            position: notes.len() - 1,
        }
    }
}

impl AudioUnit for SequencerUnit {
    fn reset(&mut self, _sample_rate: usize) {}

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        if self.trigger.detect(inputs[Self::TRIGGER_IN].unwrap_or(0.0)) {
            self.position = (self.position + 1) % SEQUENCE_LENGTH;
        }
        outputs[Self::V_OCT_OUT] = midi_to_voltage(self.params.notes[self.position].read());
    }
}

struct SequencerPanel(Arc<SequencerParams>);

impl Panel for SequencerPanel {
    fn width(&self) -> usize {
        8
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("Sequencer");
        ui.add_space(20.0);

        let mut notes: Vec<u8> = self.0.notes.iter().map(|p| p.read()).collect();
        ui.columns(SEQUENCE_LENGTH / 2, |columns| {
            for i in 0..SEQUENCE_LENGTH / 2 {
                let i2 = i + SEQUENCE_LENGTH / 2;
                columns[i].vertical(|ui| {
                    ui.add(
                        Slider::new(&mut notes[i], 21..=108)
                            .vertical()
                            .show_value(false),
                    );
                    ui.small(midi_note_name(notes[i]));
                    ui.add_space(10.0);
                    ui.add(
                        Slider::new(&mut notes[i2], 21..=108)
                            .vertical()
                            .show_value(false),
                    );
                    ui.small(midi_note_name(notes[i2]));
                });
            }
        });
        for (i, n) in notes.into_iter().enumerate() {
            self.0.notes[i].write(n);
        }

        ui.add_space(128.0);
        ui.with_layout(Layout::right_to_left(), |ui| {
            jack::outputs(ui, |ui| {
                ui.set_width(50.0);
                ui.vertical_centered(|ui| {
                    ui.small("V/Oct");
                    ui.add(Jack::output(handle.output(SequencerUnit::V_OCT_OUT)));
                });
            });
            ui.add(SignalFlow::join_horizontal());
            jack::inputs(ui, |ui| {
                ui.set_width(50.0);
                ui.vertical_centered(|ui| {
                    ui.small("Trig");
                    ui.add(Jack::input(handle.input(SequencerUnit::TRIGGER_IN)));
                });
            });
        });
    }
}

fn midi_note_name(note: u8) -> String {
    let name = match note % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => unreachable!(),
    };
    let octave = (note as i16) / 12 - 1;
    format!("{}{}", name, octave)
}
