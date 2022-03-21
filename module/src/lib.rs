// We must manually implement hash on Module{Input,Output} to include their type ID; otherwise they
// hash to the same value. This implemention is still consistent with the derived PartialEq.
#![allow(clippy::derive_hash_xor_eq)]

use std::{
    hash::{Hash, Hasher},
    sync::atomic::{AtomicU8, Ordering},
};

use eurorack::{utils::Duration, Voltage};

pub mod registry;

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

    fn create_audio_unit(&self) -> Box<dyn AudioUnit + Send>;
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

impl Hash for ModuleHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.0.hash(state);
    }
}

impl Hash for ModuleInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.module.hash(state);
        self.channel.hash(state);
    }
}

impl Hash for ModuleOutput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.module.hash(state);
        self.channel.hash(state);
    }
}

pub trait Parameter {
    type Value;
    fn read(&self) -> Self::Value;
    fn write(&self, value: Self::Value);
}

impl Parameter for AtomicU8 {
    type Value = u8;
    fn read(&self) -> Self::Value {
        self.load(Ordering::Relaxed)
    }
    fn write(&self, value: Self::Value) {
        self.store(value, Ordering::Relaxed)
    }
}

impl Parameter for atomic_float::AtomicF32 {
    type Value = f32;
    fn read(&self) -> Self::Value {
        self.load(Ordering::Relaxed)
    }
    fn write(&self, value: Self::Value) {
        self.store(value, Ordering::Relaxed)
    }
}

impl Parameter for Duration {
    type Value = f32;
    fn read(&self) -> Self::Value {
        self.seconds()
    }
    fn write(&self, value: Self::Value) {
        self.set_seconds(value);
    }
}
