// Temporarily re-export refactored interfaces:
pub use eurorack as voltage;
pub use eurorack::Voltage;
pub mod utils {
    pub use eurorack::midi_to_voltage;
    pub use eurorack::utils::*;
}
pub use module::AudioUnit as Module;
pub use module::{ModuleHandle, ModuleInput, ModuleOutput};

use module::Module as NewModule;

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

    pub fn add_module_old<M: Module + ModuleIO + Send + 'static>(
        &mut self,
        module: M,
    ) -> ModuleHandle {
        self.modules.push(ModuleFacade {
            module: Box::new(module),
            inputs: vec![None; M::INPUTS],
            outputs: vec![0.0; M::OUTPUTS],
        });
        ModuleHandle(self.modules.len() - 1)
    }

    pub fn add_module<M: NewModule>(&mut self, module: &M) -> ModuleHandle {
        self.modules.push(ModuleFacade {
            module: module.create_audio_unit(),
            inputs: vec![None; module.inputs()],
            outputs: vec![0.0; module.outputs()],
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

    pub fn disconnect(&mut self, src: ModuleOutput, dst: ModuleInput) -> Result<(), RackError> {
        if dst.module == AUDIO_OUTPUT_HANDLE {
            self.output_channel = None;
            Ok(())
        } else {
            // Find and remove the connection.
            let i = self
                .patch_cables
                .iter()
                .position(|c| c.0 == src && c.1 == dst)
                .ok_or(RackError::NotConnected)?;
            self.patch_cables.swap_remove(i);
            // Reset the destination input, as disconnected inputs do not get
            // updated every tick.
            self.modules[dst.module.0].inputs[dst.channel] = None;
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

pub const AUDIO_OUTPUT_HANDLE: ModuleHandle = ModuleHandle(usize::MAX);

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
    #[error("the referenced modules are not connected")]
    NotConnected,
}
