use std::io::{stdin, Read};

use anyhow::anyhow;
use clap::Parser;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize,
};
use filters::VCF;
use oscillators::{LFO, VCO};
use rack::{voltage::AUDIO_VOLTS, Rack};
use sequencer::Sequencer;
use utility_modules::{amplifier::VCA, clock::Clock, envelope::ADSR, midi::MidiIn};

/// Test main function for the oxable modular synth.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Use midi controller as input.
    #[clap(long)]
    midi: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut rack = Rack::new();

    let lfo = rack.add_module(LFO::new(0.1));
    let osc = rack.add_module(VCO::new());
    let adsr = rack.add_module(ADSR::default());
    let amp = rack.add_module(VCA::default());
    let filter = rack.add_module(VCF::new(1000.0, 2.0));

    // Switch between MIDI and sequenced input for easier testing.
    if args.midi {
        let midi = rack.add_module(MidiIn::new()?);
        rack.connect(midi.output(MidiIn::V_OCT_OUT), osc.input(VCO::V_OCT_IN))?;
        rack.connect(midi.output(MidiIn::GATE_OUT), adsr.input(ADSR::GATE_IN))?;
    } else {
        let clock = rack.add_module(Clock::new(180.0));
        let sequencer = rack.add_module(Sequencer::new(&[
            61, 65, 68, 77, 61, 65, 68, 75, 61, 65, 68, 78, 78, 73, 73, 73,
        ]));
        rack.connect(
            clock.output(Clock::TRIGGER_OUT),
            sequencer.input(Sequencer::TRIGGER_IN),
        )?;
        rack.connect(
            sequencer.output(Sequencer::V_OCT_OUT),
            osc.input(VCO::V_OCT_IN),
        )?;
        rack.connect(clock.output(Clock::TRIGGER_OUT), adsr.input(ADSR::GATE_IN))?;
    }

    rack.connect(adsr.output(ADSR::CV_OUT), amp.input(VCA::CV_IN))?;
    rack.connect(osc.output(VCO::SAW_OUT), amp.input(VCA::AUDIO_IN))?;
    rack.connect(amp.output(VCA::AUDIO_OUT), filter.input(VCF::AUDIO_IN))?;
    rack.connect(lfo.output(LFO::TRI_OUT), filter.input(VCF::CUTOFF_IN))?;
    rack.connect(filter.output(VCF::LOWPASS_OUT), Rack::audio_output())?;

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or(anyhow!("no output device"))?;
    let mut config = device.default_output_config()?.config();
    config.channels = 1;
    config.buffer_size = BufferSize::Fixed(64);

    rack.reset(config.sample_rate.0 as usize);
    let stream = device.build_output_stream(
        &config,
        move |samples: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for s in samples.iter_mut() {
                *s = rack.tick() / AUDIO_VOLTS;
            }
        },
        move |err| println!("cpal error: {:?}", err),
    )?;
    stream.play()?;

    let mut buf = [0];
    stdin().read_exact(&mut buf)?;
    Ok(())
}
