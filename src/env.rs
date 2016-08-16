use std::collections::HashMap;

use module::Module;
use module::ModuleName;

#[derive(Debug)]
pub struct Env {
    pub modules: HashMap<ModuleName, Module>,
}
impl Env {
    pub fn new() -> Self {
        Env { modules: HashMap::new() }
    }
    pub fn add_module(&mut self, module: Module) {
        assert!(!self.modules.contains_key(&module.name));
        self.modules.insert(module.name.clone(), module);
    }
}
