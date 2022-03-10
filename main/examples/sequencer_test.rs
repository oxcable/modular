use audio_host::AudioHost;
use filters::VCF;
use oscillators::{LFO, VCO};
use rack::Rack;
use sequencer::Sequencer;
use utility_modules::{amplifier::VcaUnit as VCA, clock::Clock, envelope::ADSR};

fn main() -> anyhow::Result<()> {
    let mut rack = Rack::new();

    let clock = rack.add_module_old(Clock::new(180.0));
    let sequencer = rack.add_module_old(Sequencer::new(&[
        61, 65, 68, 77, 61, 65, 68, 75, 61, 65, 68, 78, 78, 73, 73, 73,
    ]));
    let lfo = rack.add_module_old(LFO::new(0.1));
    let osc = rack.add_module_old(VCO::new());
    let adsr = rack.add_module_old(ADSR::default());
    let amp = rack.add_module_old(VCA::default());
    let filter = rack.add_module_old(VCF::new(1000.0, 2.0));

    rack.connect(
        clock.output(Clock::TRIGGER_OUT),
        sequencer.input(Sequencer::TRIGGER_IN),
    )?;
    rack.connect(
        sequencer.output(Sequencer::V_OCT_OUT),
        osc.input(VCO::V_OCT_IN),
    )?;
    rack.connect(clock.output(Clock::TRIGGER_OUT), adsr.input(ADSR::GATE_IN))?;
    rack.connect(adsr.output(ADSR::CV_OUT), amp.input(VCA::CV_IN))?;
    rack.connect(osc.output(VCO::SAW_OUT), amp.input(VCA::AUDIO_IN))?;
    rack.connect(amp.output(VCA::AUDIO_OUT), filter.input(VCF::AUDIO_IN))?;
    rack.connect(lfo.output(LFO::TRI_OUT), filter.input(VCF::CUTOFF_IN))?;
    rack.connect(filter.output(VCF::LOWPASS_OUT), Rack::audio_output())?;

    AudioHost::default().run_forever(rack)?;
    Ok(())
}
