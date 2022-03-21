use audio_host::AudioHost;
use filters::Vcf;
use oscillators::{lfo::Lfo, vco::Vco};
use rack::Rack;
use sequencer::Sequencer;
use utility_modules::{amplifier::Vca, clock::Clock, envelope::Adsr};

fn main() -> anyhow::Result<()> {
    let mut rack = Rack::new();

    let clock = rack.take_module(Clock::default());
    let sequencer = rack.take_module(Sequencer::with_sequence([
        61, 65, 68, 77, 61, 65, 68, 75, // 61, 65, 68, 78, 78, 73, 73, 73,
    ]));
    let lfo = rack.take_module(Lfo::default());
    let osc = rack.take_module(Vco::default());
    let adsr = rack.take_module(Adsr::default());
    let amp = rack.take_module(Vca::default());
    let filter = rack.take_module(Vcf::new(1000.0, 2.0));

    rack.connect(
        clock.output(Clock::TRIGGER_OUT),
        sequencer.input(Sequencer::TRIGGER_IN),
    )?;
    rack.connect(
        sequencer.output(Sequencer::V_OCT_OUT),
        osc.input(Vco::V_OCT_IN),
    )?;
    rack.connect(clock.output(Clock::TRIGGER_OUT), adsr.input(Adsr::GATE_IN))?;
    rack.connect(adsr.output(Adsr::CV_OUT), amp.input(Vca::CV_IN))?;
    rack.connect(osc.output(Vco::SAW_OUT), amp.input(Vca::AUDIO_IN))?;
    rack.connect(amp.output(Vca::AUDIO_OUT), filter.input(Vcf::AUDIO_IN))?;
    rack.connect(lfo.output(Lfo::TRI_OUT), filter.input(Vcf::CUTOFF_IN))?;
    rack.connect(filter.output(Vcf::LOWPASS_OUT), Rack::audio_output())?;

    AudioHost::default().run_forever(rack)?;
    Ok(())
}
