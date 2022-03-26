use std::collections::HashMap;

use crate::{Module, ModuleHandle};

#[derive(Clone, Debug)]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
}

#[derive(Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, RegisteredModule>,
    next_handle: usize,
}

impl ModuleRegistry {
    pub fn all_modules(&self) -> Vec<ModuleManifest> {
        self.modules.values().map(|e| e.manifest.clone()).collect()
    }

    pub fn register<M>(&mut self, id: &str, name: &str)
    where
        M: 'static + Module + Default,
    {
        self.modules.insert(
            id.to_owned(),
            RegisteredModule {
                manifest: ModuleManifest {
                    id: id.to_owned(),
                    name: name.to_owned(),
                },
                factory: Box::new(|| Box::new(M::default())),
            },
        );
    }

    pub fn create_module(
        &mut self,
        id: &str,
    ) -> Result<(ModuleHandle, Box<dyn Module>), RegistryError> {
        if let Some(entry) = self.modules.get(id) {
            let handle = ModuleHandle(self.next_handle);
            self.next_handle += 1;
            Ok((handle, (entry.factory)()))
        } else {
            Err(RegistryError::NotRegistered(id.to_owned()))
        }
    }
}

struct RegisteredModule {
    manifest: ModuleManifest,
    factory: Box<dyn Fn() -> Box<dyn Module>>,
}

#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    #[error("no module with id '{0}' exists")]
    NotRegistered(String),
}
