use audio_host::AudioHost;
use oscillators::VCO;
use rack::Rack;
use utility_modules::{
    amplifier::VcaUnit as VCA, envelope::AdsrUnit as ADSR, midi::MidiInUnit as MidiIn,
};

fn main() -> anyhow::Result<()> {
    let mut rack = Rack::new();

    let midi = rack.add_module_old(MidiIn::new()?);
    let osc = rack.add_module_old(VCO::new());
    let adsr = rack.add_module_old(ADSR::default());
    let amp = rack.add_module_old(VCA::default());

    rack.connect(midi.output(MidiIn::V_OCT_OUT), osc.input(VCO::V_OCT_IN))?;
    rack.connect(midi.output(MidiIn::GATE_OUT), adsr.input(ADSR::GATE_IN))?;
    rack.connect(adsr.output(ADSR::CV_OUT), amp.input(VCA::CV_IN))?;
    rack.connect(osc.output(VCO::SAW_OUT), amp.input(VCA::AUDIO_IN))?;
    rack.connect(amp.output(VCA::AUDIO_OUT), Rack::audio_output())?;

    AudioHost::default().run_forever(rack)?;
    Ok(())
}
