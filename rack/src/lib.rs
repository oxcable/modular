// Temporarily re-export refactored interfaces:
pub use eurorack as voltage;
pub use eurorack::Voltage;
pub mod utils {
    pub use eurorack::midi_to_voltage;
    pub use eurorack::utils::*;
}
pub use module::AudioUnit as Module;
pub use module::{ModuleHandle, ModuleInput, ModuleOutput};

pub trait ModuleIO {
    const INPUTS: usize;
    const OUTPUTS: usize;
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

    pub fn audio_output() -> ModuleInput {
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
        if dst.module == AUDIO_OUTPUT_HANDLE {
            self.output_channel = Some(src);
            Ok(())
        } else if src.module.0 >= self.modules.len() || dst.module.0 >= self.modules.len() {
            Err(RackError::InvalidModule)
        } else if src.channel >= self.modules[src.module.0].outputs.len()
            || dst.channel >= self.modules[dst.module.0].inputs.len()
        {
            Err(RackError::InvalidChannel)
        } else {
            self.patch_cables.push((src, dst));
            Ok(())
        }
    }

    pub fn reset(&mut self, sample_rate: usize) {
        for module in &mut self.modules {
            module.module.reset(sample_rate);
        }
    }

    pub fn tick(&mut self) -> Voltage {
        // First propogate voltages through all patch cables. All signals take 1 sample to
        // propogate. This simplifies routing and enables feedback and circular patches.
        for (src, dst) in &self.patch_cables {
            let v = self.modules[src.module.0].outputs[src.channel];
            self.modules[dst.module.0].inputs[dst.channel] = Some(v);
        }

        for module in &mut self.modules {
            module.module.tick(&module.inputs, &mut module.outputs);
        }

        self.output_channel
            .map_or(0.0, |src| self.modules[src.module.0].outputs[src.channel])
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
