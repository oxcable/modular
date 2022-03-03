use std::io::{stdin, Read};

use anyhow::anyhow;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize,
};
use oscillators::VCO;
use rack::{voltage::AUDIO_VOLTS, Rack};

fn main() -> anyhow::Result<()> {
    let mut rack = Rack::new();
    let vco = rack.add_module(VCO::new());
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
