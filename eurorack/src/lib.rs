pub mod utils;

/// The type for a single sample.
pub type Voltage = f32;

/// The maximum voltage level for audio signals (±5V).
pub const AUDIO_VOLTS: f32 = 5.0;

/// The maximum voltage level for control signals (0-10V).
pub const CV_VOLTS: f32 = 10.0;

/// The threshold voltage for activating gates.
pub const GATE_THRESHOLD_VOLTS: f32 = 2.0;

/// The base frequency for 1V/Octave CV signals.
pub const V_OCT_F0: f32 = 261.6256; // C4

/// Converts a midi note number to a 1V/Octave voltage.
pub fn midi_to_voltage(midi_note: u8) -> f32 {
    (midi_note as f32 - 60.0) / 12.0
}
