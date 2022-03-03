use crate::voltage::{Voltage, CV_VOLTS};

/// A utility for managing sample durations across resets.
#[derive(Debug)]
pub struct Duration {
    seconds: f32,
    samples: Option<usize>,
}

impl Duration {
    pub fn new(seconds: f32) -> Self {
        Duration {
            seconds,
            samples: None,
        }
    }

    pub fn reset(&mut self, sample_rate: usize) {
        self.samples = Some((sample_rate as f32 * self.seconds) as usize);
    }

    pub fn samples(&self) -> usize {
        self.samples.expect("Duration not initialized")
    }
}

/// A utility for generating pulse waves (e.g. for triggers).
#[derive(Debug)]
pub struct PulseGenerator {
    duration: Duration,
    samples_remaining: usize,
}

impl PulseGenerator {
    /// Creates a pulse generator that holds for the specified `duration`.
    pub fn with_duration(duration_ms: f32) -> Self {
        PulseGenerator {
            duration: Duration::new(duration_ms / 1000.0),
            samples_remaining: 0,
        }
    }

    /// Creates a pulse generator suitable gnerating trigger signals.
    pub fn for_triggers() -> Self {
        PulseGenerator::with_duration(1.0)
    }

    /// Resets the PulseGenerator.
    pub fn reset(&mut self, sample_rate: usize) {
        self.duration.reset(sample_rate);
    }

    /// Triggers a new pulse.
    pub fn trigger(&mut self) {
        self.samples_remaining = self.duration.samples();
    }

    /// Ticks the pulse generator, emitting a control voltage
    pub fn tick(&mut self) -> f32 {
        if self.samples_remaining > 0 {
            self.samples_remaining -= 1;
            CV_VOLTS
        } else {
            0.0
        }
    }
}

/// A utility for detecting triggers.
#[derive(Debug)]
pub struct SchmittTrigger {
    trigger_threshold: f32,
    reset_threshold: f32,
    active: bool,
}

impl SchmittTrigger {
    /// Creates a new trigger using the provided thresholds.
    pub fn new(trigger_threshold: f32, reset_threshold: f32) -> Self {
        Self {
            trigger_threshold,
            reset_threshold,
            active: false,
        }
    }

    /// Consumes the voltage, and indicates whether a new trigger was detected.
    pub fn detect(&mut self, v: Voltage) -> bool {
        if self.active {
            self.active = v >= self.reset_threshold;
            false
        } else {
            self.active = v >= self.trigger_threshold;
            self.active
        }
    }
}

impl Default for SchmittTrigger {
    fn default() -> Self {
        SchmittTrigger::new(2.0, 0.1)
    }
}
