use std::sync::mpsc;

use midir::{MidiInput, MidiInputConnection};
use midly::{live::LiveEvent, MidiMessage};
use rack::{
    utils::midi_to_voltage,
    voltage::{Voltage, CV_VOLTS},
    Module, ModuleIO,
};

pub struct MidiIn {
    _connection: MidiInputConnection<()>,
    rx: mpsc::Receiver<MidiMessage>,
    active: bool,
    key: u8,
    voltage: f32,
}

impl ModuleIO for MidiIn {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 2;
}

impl MidiIn {
    pub const V_OCT_OUT: usize = 0;
    pub const GATE_OUT: usize = 1;

    pub fn new() -> Result<MidiIn, Error> {
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
                Ok(MidiIn {
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

impl Module for MidiIn {
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
        outputs[Self::V_OCT_OUT] = self.voltage;
        outputs[Self::GATE_OUT] = if self.active { CV_VOLTS } else { 0.0 };
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
