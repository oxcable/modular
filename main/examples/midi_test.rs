use audio_host::AudioHost;
use oscillators::vco::Vco;
use rack::Rack;
use utility_modules::{amplifier::Vca, envelope::Adsr, midi::MidiIn};

fn main() -> anyhow::Result<()> {
    let mut rack = Rack::new();

    let midi = rack.take_module(MidiIn::default());
    let osc = rack.take_module(Vco::default());
    let adsr = rack.take_module(Adsr::default());
    let amp = rack.take_module(Vca::default());

    rack.connect(midi.output(MidiIn::V_OCT_OUT), osc.input(Vco::V_OCT_IN))?;
    rack.connect(midi.output(MidiIn::GATE_OUT), adsr.input(Adsr::GATE_IN))?;
    rack.connect(adsr.output(Adsr::CV_OUT), amp.input(Vca::CV_IN))?;
    rack.connect(osc.output(Vco::SAW_OUT), amp.input(Vca::AUDIO_IN))?;
    rack.connect(amp.output(Vca::AUDIO_OUT), Rack::audio_output())?;

    AudioHost::default().run_forever(rack)?;
    Ok(())
}
