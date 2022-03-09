use std::sync::mpsc;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Stream,
};
use rack::{voltage::AUDIO_VOLTS, Rack};

pub struct AudioHost {
    buffer_size: u32,
    stream: Option<Stream>,
}

impl AudioHost {
    pub fn new(buffer_size: u32) -> Self {
        AudioHost {
            buffer_size,
            stream: None,
        }
    }

    pub fn start(&mut self, mut rack: Rack) -> Result<(), AudioHostError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioHostError::NoOutputDevice)?;
        let mut config = device.default_output_config()?.config();
        config.channels = 1;
        config.buffer_size = BufferSize::Fixed(self.buffer_size);

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
        self.stream = Some(stream);

        Ok(())
    }

    pub fn run_forever(&mut self, rack: Rack) -> Result<(), AudioHostError> {
        self.start(rack)?;

        let (tx, rx) = mpsc::channel();
        ctrlc::set_handler(move || {
            tx.send(()).unwrap();
        })
        .unwrap();
        rx.recv().unwrap();

        Ok(())
    }
}

impl Default for AudioHost {
    fn default() -> Self {
        AudioHost::new(64)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AudioHostError {
    #[error("no output device was found")]
    NoOutputDevice,
    #[error("")]
    StreamConfig(#[from] cpal::DefaultStreamConfigError),
    #[error("")]
    BuildStream(#[from] cpal::BuildStreamError),
    #[error("")]
    PlayStream(#[from] cpal::PlayStreamError),
}
