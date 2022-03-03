pub mod utils;
pub mod voltage;

use voltage::Voltage;

pub trait ModuleIO {
    const INPUTS: usize;
    const OUTPUTS: usize;
}

pub trait Module {
    fn reset(&mut self, sample_rate: usize);
    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ModuleHandle(usize);

#[derive(Copy, Clone, Debug)]
pub struct ModuleInput {
    module: usize,
    channel: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct ModuleOutput {
    module: usize,
    channel: usize,
}

impl ModuleHandle {
    pub fn input(&self, channel: usize) -> ModuleInput {
        ModuleInput {
            module: self.0,
            channel,
        }
    }

    pub fn output(&self, channel: usize) -> ModuleOutput {
        ModuleOutput {
            module: self.0,
            channel,
        }
    }
}

pub struct Rack {
    modules: Vec<ModuleFacade>,
    patch_cables: Vec<(ModuleOutput, ModuleInput)>,
    output_channel: Option<ModuleOutput>,
}

impl Rack {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Rack {
            modules: Vec::new(),
            patch_cables: Vec::new(),
            output_channel: None,
        }
    }

    pub fn audio_output(&self) -> ModuleInput {
        AUDIO_OUTPUT_HANDLE.input(0)
    }

    pub fn add_module<M: Module + ModuleIO + Send + 'static>(&mut self, module: M) -> ModuleHandle {
        self.modules.push(ModuleFacade {
            module: Box::new(module),
            inputs: vec![None; M::INPUTS],
            outputs: vec![0.0; M::OUTPUTS],
        });
        ModuleHandle(self.modules.len() - 1)
    }

    pub fn connect(&mut self, src: ModuleOutput, dst: ModuleInput) -> Result<(), RackError> {
        if dst.module == AUDIO_OUTPUT_HANDLE.0 {
            self.output_channel = Some(src);
            Ok(())
        } else if src.module >= self.modules.len() || dst.module >= self.modules.len() {
            Err(RackError::InvalidModule)
        } else if src.channel >= self.modules[src.module].outputs.len()
            || dst.channel >= self.modules[dst.module].inputs.len()
        {
            Err(RackError::InvalidChannel)
        } else {
            self.patch_cables.push((src, dst));
            Ok(())
        }
    }

    pub fn reset(&mut self, sample_rate: usize) {
        for module in self.modules.iter_mut() {
            module.module.reset(sample_rate);
        }
    }

    pub fn tick(&mut self) -> Voltage {
        // First propogate voltages through all patch cables. All signals take 1 sample to
        // propogate. This simplifies routing and enables feedback and circular patches.
        for (src, dst) in self.patch_cables.iter() {
            let v = self.modules[src.module].outputs[src.channel];
            self.modules[dst.module].inputs[dst.channel] = Some(v);
        }

        for module in self.modules.iter_mut() {
            module.module.tick(&module.inputs, &mut module.outputs);
        }

        self.output_channel
            .map_or(0.0, |src| self.modules[src.module].outputs[src.channel])
    }
}

const AUDIO_OUTPUT_HANDLE: ModuleHandle = ModuleHandle(usize::MAX);

struct ModuleFacade {
    module: Box<dyn Module + Send>,
    inputs: Vec<Option<Voltage>>,
    outputs: Vec<Voltage>,
}

#[derive(thiserror::Error, Debug)]
pub enum RackError {
    #[error("the referenced module does not exist")]
    InvalidModule,
    #[error("the referenced module channel does not exist")]
    InvalidChannel,
}
