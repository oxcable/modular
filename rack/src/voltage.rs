/// The type for a single sample.
pub type Voltage = f32;

/// The maximum voltage level for audio signals (±5V).
pub const AUDIO_VOLTS: f32 = 5.0;

/// The maximum voltage level for control signals (0-10V).
pub const CV_VOLTS: f32 = 10.0;

/// The base frequency for 1V/Octave CV signals.
pub const V_OCT_F0: f32 = 261.6256; // C4
