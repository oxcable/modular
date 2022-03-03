use rack::{
    utils::{Duration, PulseGenerator},
    voltage::Voltage,
    Module, ModuleIO,
};

pub struct Clock {
    beat_duration: Duration,
    pulse: PulseGenerator,
    ticks: usize,
}

impl ModuleIO for Clock {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 1;
}

impl Clock {
    pub const TRIGGER_OUT: usize = 0;

    pub fn new(bpm: f32) -> Self {
        Clock {
            beat_duration: Duration::new(60.0 / bpm),
            pulse: PulseGenerator::for_triggers(),
            ticks: 0,
        }
    }
}

impl Module for Clock {
    fn reset(&mut self, sample_rate: usize) {
        self.beat_duration.reset(sample_rate);
        self.pulse.reset(sample_rate);
    }

    fn tick(&mut self, _inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        if self.ticks == 0 {
            self.pulse.trigger();
        }
        self.ticks = (self.ticks + 1) % self.beat_duration.samples();
        outputs[Self::TRIGGER_OUT] = self.pulse.tick();
    }
}
