use std::collections::HashMap;

use crate::{Module, ModuleHandle};

#[derive(Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleFactory>,
    next_handle: usize,
}

impl ModuleRegistry {
    pub fn all_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    pub fn register<M>(&mut self, name: String)
    where
        M: 'static + Module + Default,
    {
        self.modules
            .insert(name, Box::new(|| Box::new(M::default())));
    }

    pub fn create_module(
        &mut self,
        name: String,
    ) -> Result<(ModuleHandle, Box<dyn Module>), RegistryError> {
        if let Some(factory) = self.modules.get(&name) {
            let handle = ModuleHandle(self.next_handle);
            self.next_handle += 1;
            Ok((handle, factory()))
        } else {
            Err(RegistryError::NotRegistered(name))
        }
    }
}

type ModuleFactory = Box<dyn Fn() -> Box<dyn Module>>;

#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    #[error("no module with this name '{0}' exists")]
    NotRegistered(String),
}
