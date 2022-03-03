use std::io::{stdin, Read};

use anyhow::anyhow;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize,
};
use oscillators::VCO;
use rack::{voltage::AUDIO_VOLTS, Rack};
use sequencer::Sequencer;
use utility_modules::clock::Clock;

fn main() -> anyhow::Result<()> {
    let mut rack = Rack::new();
    let clock = rack.add_module(Clock::new(180.0));
    let sequencer = rack.add_module(Sequencer::new(&[
        61, 65, 68, 77, 61, 65, 68, 75, 61, 65, 68, 78, 78, 73, 73, 73,
    ]));
    let vco = rack.add_module(VCO::new());
    rack.connect(
        clock.output(Clock::TRIGGER_OUT),
        sequencer.input(Sequencer::TRIGGER_IN),
    )?;
    rack.connect(
        sequencer.output(Sequencer::V_OCT_OUT),
        vco.input(VCO::V_OCT_IN),
    )?;
    rack.connect(vco.output(VCO::SAW_OUT), rack.audio_output())?;

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
    let _ = stdin().read(&mut buf);
    Ok(())
}
