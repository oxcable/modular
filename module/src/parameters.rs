use std::sync::atomic::{AtomicU8, Ordering};

use eurorack::utils::Duration;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum SerializedParameter {
    Num(f32),
    List(Vec<SerializedParameter>),
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
