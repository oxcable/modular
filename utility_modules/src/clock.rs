use rack::{
    utils::Duration,
    voltage::{Voltage, CV_VOLTS},
    Module, ModuleIO,
};

pub struct Clock {
    period: Duration,
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
            period: Duration::new(60.0 / bpm),
            ticks: 0,
        }
    }
}

impl Module for Clock {
    fn reset(&mut self, sample_rate: usize) {
        self.period.reset(sample_rate);
    }

    fn tick(&mut self, _inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        let period = self.period.samples();
        self.ticks = (self.ticks + 1) % period;
        outputs[Self::TRIGGER_OUT] = if self.ticks < period / 2 {
            CV_VOLTS
        } else {
            0.0
        };
    }
}
