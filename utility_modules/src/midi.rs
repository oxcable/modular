use std::sync::mpsc;

use eurorack::{midi_to_voltage, Voltage, CV_VOLTS};
use midir::{MidiInput, MidiInputConnection};
use midly::{live::LiveEvent, MidiMessage};
use module::{AudioUnit, Module, Panel};
use widgets::{
    egui::{self, Align, Layout},
    jack::{self, Jack},
};

#[derive(Default)]
pub struct MidiIn {}

impl MidiIn {
    pub const V_OCT_OUT: usize = 0;
    pub const GATE_OUT: usize = 1;
}

impl Module for MidiIn {
    fn inputs(&self) -> usize {
        0
    }

    fn outputs(&self) -> usize {
        2
    }

    fn create_audio_unit(&self) -> Box<dyn AudioUnit + Send> {
        // TODO: We should figure out how to actually do error handling for this; we probably don't
        // want panics in this function.
        Box::new(MidiInUnit::new().expect("couldn't connect to midi device"))
    }

    fn create_panel(&self) -> Box<dyn Panel> {
        Box::new(MidiInPanel {})
    }
}

struct MidiInUnit {
    _connection: MidiInputConnection<()>,
    rx: mpsc::Receiver<MidiMessage>,
    active: bool,
    key: u8,
    voltage: f32,
}

impl MidiInUnit {
    fn new() -> Result<MidiInUnit, Error> {
        let midi_input = MidiInput::new("utility_modules::midi")?;
        match midi_input.ports().first() {
            Some(port) => {
                let (tx, rx) = mpsc::channel();
                let connection = midi_input
                    .connect(
                        port,
                        "utility_modules::midi",
                        move |_stamp, msg, _| {
                            if let Ok(LiveEvent::Midi { message, .. }) = LiveEvent::parse(msg) {
                                tx.send(message).unwrap();
                            }
                        },
                        (),
                    )
                    .map_err(|_| Error::ConnectError)?;
                Ok(MidiInUnit {
                    _connection: connection,
                    rx,
                    active: false,
                    key: 0,
                    voltage: 0.0,
                })
            }
            None => Err(Error::NoMidiDevice),
        }
    }
}

impl AudioUnit for MidiInUnit {
    fn reset(&mut self, _sample_rate: usize) {}

    fn tick(&mut self, _inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        // Listen for midi updates.
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                MidiMessage::NoteOn { key, .. } => {
                    self.active = true;
                    self.key = key.as_int();
                    self.voltage = midi_to_voltage(self.key);
                }
                MidiMessage::NoteOff { key, .. } if key.as_int() == self.key => self.active = false,
                _ => (),
            }
        }

        // Write out the current midi note.
        outputs[MidiIn::V_OCT_OUT] = self.voltage;
        outputs[MidiIn::GATE_OUT] = if self.active { CV_VOLTS } else { 0.0 };
    }
}

struct MidiInPanel;

impl Panel for MidiInPanel {
    fn width(&self) -> usize {
        4
    }

    fn update(&mut self, handle: &module::ModuleHandle, ui: &mut egui::Ui) {
        ui.heading("MIDI");
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            jack::outputs(ui, |ui| {
                ui.add(Jack::output(handle.output(MidiIn::V_OCT_OUT)));
                ui.small("V/Oct");
                ui.add(Jack::output(handle.output(MidiIn::GATE_OUT)));
                ui.small("Gate");
            });
        });
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("midir init error")]
    InitError(#[from] midir::InitError),
    #[error("midir connection error")]
    ConnectError,
    #[error("no midi device found")]
    NoMidiDevice,
}
