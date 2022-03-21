use std::collections::HashMap;

use crate::Module;

#[derive(Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleFactory>,
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
}

type ModuleFactory = Box<dyn Fn() -> Box<dyn Module>>;
