// We must manually implement hash on Module{Input,Output} to include their type ID; otherwise they
// hash to the same value. This implemention is still consistent with the derived PartialEq.
#![allow(clippy::derive_hash_xor_eq)]

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::atomic::{AtomicU8, Ordering},
};

use eurorack::{utils::Duration, Voltage};

pub mod registry;

pub trait AudioUnit: Send {
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

    fn serialize(&self) -> HashMap<String, SerializedParameter>;
    fn deserialize(&self, params: &HashMap<String, SerializedParameter>);
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum SerializedParameter {
    Num(f32),
    List(Vec<Box<SerializedParameter>>),
}

impl SerializedParameter {
    fn as_num(&self) -> f32 {
        match self {
            SerializedParameter::Num(value) => *value,
            _ => panic!("SerializedParameter is not f32"),
        }
    }
}

pub trait Parameter {
    type Value;
    fn read(&self) -> Self::Value;
    fn write(&self, value: Self::Value);
    fn serialize(&self) -> SerializedParameter;
    fn deserialize(&self, serialized: &SerializedParameter);
}

impl Parameter for AtomicU8 {
    type Value = u8;
    fn read(&self) -> Self::Value {
        self.load(Ordering::Relaxed)
    }
    fn write(&self, value: Self::Value) {
        self.store(value, Ordering::Relaxed)
    }
    fn serialize(&self) -> SerializedParameter {
        SerializedParameter::Num(self.read() as f32)
    }
    fn deserialize(&self, serialized: &SerializedParameter) {
        self.write(serialized.as_num() as u8);
    }
}

impl Parameter for portable_atomic::AtomicF32 {
    type Value = f32;
    fn read(&self) -> Self::Value {
        self.load(Ordering::Relaxed)
    }
    fn write(&self, value: Self::Value) {
        self.store(value, Ordering::Relaxed)
    }
    fn serialize(&self) -> SerializedParameter {
        SerializedParameter::Num(self.read())
    }
    fn deserialize(&self, serialized: &SerializedParameter) {
        self.write(serialized.as_num());
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
    fn serialize(&self) -> SerializedParameter {
        SerializedParameter::Num(self.read())
    }
    fn deserialize(&self, serialized: &SerializedParameter) {
        self.write(serialized.as_num());
    }
}
