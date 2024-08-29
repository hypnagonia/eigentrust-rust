use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use console_log;

#[cfg(not(target_arch = "wasm32"))]
use env_logger;

use log::Level;

pub fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(Level::Info).expect("Failed to initialize logger");
        log::debug!("Logger initialized for WebAssembly");
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
        log::debug!("Logger initialized for native environment");
    }
}

pub struct PeersMap {
    pub map: HashMap<String, usize>,
    pub map_reversed: HashMap<usize, String>,
    pub max_value: usize,
}

impl PeersMap {
    pub fn new() -> Self {
        PeersMap {
            map_reversed: HashMap::new(),
            map: HashMap::new(),
            max_value: 0,
        }
    }

    pub fn insert_or_get(&mut self, key: String) -> usize {
        if let Some(&existing_value) = self.map.get(&key) {
            return existing_value;
        }

        self.map.insert(key.clone(), self.max_value);
        self.map_reversed.insert(self.max_value, key.clone());

        self.max_value += 1;
        self.max_value - 1
    }

    pub fn get_max_value(&self) -> usize {
        self.max_value
    }
}
