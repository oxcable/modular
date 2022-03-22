use module::registry::ModuleRegistry;

pub mod amplifier;
pub mod clock;
pub mod envelope;
pub mod filters;
pub mod lfo;
pub mod midi;
pub mod oscillators;
pub mod sequencer;

pub fn builtin_modules() -> ModuleRegistry {
    let mut registry = ModuleRegistry::default();
    registry.register::<amplifier::Vca>("builtins::Vca", "VCA");
    registry.register::<clock::Clock>("builtins::Clock", "Clock");
    registry.register::<envelope::Adsr>("builtins::Adsr", "Adsr");
    registry.register::<filters::Vcf>("builtins::Vcf", "VCF");
    registry.register::<lfo::Lfo>("builtins::Lfo", "LFO");
    registry.register::<midi::MidiIn>("builtins::MidiIn", "MidiIn");
    registry.register::<oscillators::Vco>("builtins::Vco", "VCO");
    registry.register::<sequencer::Sequencer>("builtins::Sequencer", "Sequencer");
    registry
}
