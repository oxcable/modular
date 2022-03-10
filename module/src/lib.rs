use eurorack::Voltage;

pub trait AudioUnit {
    fn reset(&mut self, sample_rate: usize);
    fn tick(&mut self, inputs: &[Option<Voltage>], outputs: &mut [Voltage]);
}

pub trait Panel {
    fn width(&self) -> usize;
    fn update(&mut self, handle: &ModuleHandle, ui: &mut egui::Ui);
}

pub trait Module {
    fn inputs(&self) -> usize;
    fn outputs(&self) -> usize;

    fn create_audio_unit(&self) -> Box<dyn AudioUnit>;
    fn create_panel(&self) -> Box<dyn Panel>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModuleHandle(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModuleInput {
    pub module: ModuleHandle,
    pub channel: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModuleOutput {
    pub module: ModuleHandle,
    pub channel: usize,
}

impl ModuleHandle {
    pub fn input(&self, channel: usize) -> ModuleInput {
        ModuleInput {
            module: *self,
            channel,
        }
    }

    pub fn output(&self, channel: usize) -> ModuleOutput {
        ModuleOutput {
            module: *self,
            channel,
        }
    }
}
