use std::{
    f32::consts::PI,
    io::{stdin, Read},
};

use anyhow::anyhow;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize,
};
use rack::{Module, ModuleIO, Rack, Voltage};

struct SineModule {
    phase: f32,
    dphase: f32,
}

impl SineModule {
    fn new() -> Self {
        SineModule {
            phase: 0.0,
            dphase: 0.0,
        }
    }
}

impl Module for SineModule {
    fn reset(&mut self, sample_rate: usize) {
        self.dphase = 2.0 * PI * 220.0 / sample_rate as f32;
    }

    fn tick(&mut self, _inputs: &[Option<Voltage>], outputs: &mut [Voltage]) {
        outputs[0] = self.phase.sin();
        self.phase += self.dphase;
    }
}

impl ModuleIO for SineModule {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 1;
}

fn main() -> anyhow::Result<()> {
    let mut rack = Rack::new();
    let sine = rack.add_module(SineModule::new());
    rack.connect(sine.output(0), rack.audio_output())?;

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
                *s = rack.tick();
            }
        },
        move |err| println!("cpal error: {:?}", err),
    )?;
    stream.play()?;

    let mut buf = [0];
    let _ = stdin().read(&mut buf);
    Ok(())
}
