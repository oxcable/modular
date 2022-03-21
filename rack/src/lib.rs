use std::collections::HashMap;

use eurorack::Voltage;
use module::{AudioUnit, Module, ModuleHandle, ModuleInput, ModuleOutput};

pub struct Rack {
    sample_rate: usize,
    modules: HashMap<ModuleHandle, AudioUnitFacade>,
    patch_cables: Vec<(ModuleOutput, ModuleInput)>,
    output_channel: Option<ModuleOutput>,
}

impl Rack {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Rack {
            sample_rate: 0,
            modules: HashMap::new(),
            patch_cables: Vec::new(),
            output_channel: None,
        }
    }

    pub fn audio_output() -> ModuleInput {
        AUDIO_OUTPUT_HANDLE.input(0)
    }

    pub fn add_audio_unit(
        &mut self,
        handle: ModuleHandle,
        inputs: usize,
        outputs: usize,
        mut audio_unit: Box<dyn AudioUnit + Send>,
    ) {
        audio_unit.reset(self.sample_rate);
        self.modules.insert(
            handle,
            AudioUnitFacade {
                audio_unit,
                inputs: vec![None; inputs],
                outputs: vec![0.0; outputs],
            },
        );
    }

    pub fn add_module<M: Module>(&mut self, module: &M) -> ModuleHandle {
        let handle = ModuleHandle(self.modules.len());
        self.add_audio_unit(
            handle,
            module.inputs(),
            module.outputs(),
            module.create_audio_unit(),
        );
        handle
    }

    pub fn take_module<M: Module>(&mut self, module: M) -> ModuleHandle {
        self.add_module(&module)
    }

    pub fn connect(&mut self, src: ModuleOutput, dst: ModuleInput) -> Result<(), RackError> {
        if dst.module == AUDIO_OUTPUT_HANDLE {
            self.output_channel = Some(src);
            Ok(())
        } else if !self.modules.contains_key(&src.module) || !self.modules.contains_key(&dst.module)
        {
            Err(RackError::InvalidModule)
        } else if src.channel >= self.modules[&src.module].outputs.len()
            || dst.channel >= self.modules[&dst.module].inputs.len()
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
            self.modules.get_mut(&dst.module).unwrap().inputs[dst.channel] = None;
            Ok(())
        }
    }

    pub fn reset(&mut self, sample_rate: usize) {
        self.sample_rate = sample_rate;
        for module in self.modules.values_mut() {
            module.audio_unit.reset(sample_rate);
        }
    }

    pub fn tick(&mut self) -> Voltage {
        // First propogate voltages through all patch cables. All signals take 1 sample to
        // propogate. This simplifies routing and enables feedback and circular patches.
        for (src, dst) in &self.patch_cables {
            let v = self.modules[&src.module].outputs[src.channel];
            self.modules.get_mut(&dst.module).unwrap().inputs[dst.channel] = Some(v);
        }

        for module in self.modules.values_mut() {
            module.audio_unit.tick(&module.inputs, &mut module.outputs);
        }

        self.output_channel
            .map_or(0.0, |src| self.modules[&src.module].outputs[src.channel])
    }
}

pub const AUDIO_OUTPUT_HANDLE: ModuleHandle = ModuleHandle(usize::MAX);

struct AudioUnitFacade {
    audio_unit: Box<dyn AudioUnit + Send>,
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
