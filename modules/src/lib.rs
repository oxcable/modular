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
    registry.register::<amplifier::Vca>("VCA".to_owned());
    registry.register::<clock::Clock>("Clock".to_owned());
    registry.register::<envelope::Adsr>("Adsr".to_owned());
    registry.register::<filters::Vcf>("VCF".to_owned());
    registry.register::<lfo::Lfo>("LFO".to_owned());
    registry.register::<midi::MidiIn>("MidiIn".to_owned());
    registry.register::<oscillators::Vco>("VCO".to_owned());
    registry.register::<sequencer::Sequencer>("Sequencer".to_owned());
    registry
}
