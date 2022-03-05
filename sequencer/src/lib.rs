use rack::{utils::SchmittTrigger, voltage::Voltage, Module, ModuleIO};

pub struct Sequencer {
    trigger: SchmittTrigger,
    notes: Vec<f32>,
    position: usize,
}

impl ModuleIO for Sequencer {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 1;
}

impl Sequencer {
    pub const TRIGGER_IN: usize = 0;

    pub const V_OCT_OUT: usize = 0;

    pub fn new(notes: &[u8]) -> Self {
        assert!(!notes.is_empty());
        Sequencer {
            trigger: SchmittTrigger::default(),
            notes: notes.iter().copied().map(midi_to_cv).collect(),
            position: notes.len() - 1,
        }
    }
}

impl Module for Sequencer {
    fn reset(&mut self, _sample_rate: usize) {}

    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        if self.trigger.detect(inputs[Self::TRIGGER_IN].unwrap_or(0.0)) {
            self.position = (self.position + 1) % self.notes.len();
        }
        outputs[Self::V_OCT_OUT] = self.notes[self.position];
    }
}

fn midi_to_cv(midi_note: u8) -> f32 {
    (midi_note as f32 - 60.0) / 12.0
}
