use std::collections::HashMap;

pub fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        // the demo recomputes on every slider move, keep the browser console quiet
        console_log::init_with_level(log::Level::Warn).expect("Failed to initialize logger");
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // RUST_LOG overrides, e.g. RUST_LOG=trace
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();
        log::debug!("Logger initialized for native environment");
    }
}

pub struct PeersMap {
    pub map: HashMap<String, usize>,
    // index -> peer name
    pub names: Vec<String>,
}

impl Default for PeersMap {
    fn default() -> Self {
        Self::new()
    }
}

impl PeersMap {
    pub fn new() -> Self {
        PeersMap {
            map: HashMap::new(),
            names: Vec::new(),
        }
    }

    pub fn insert_or_get(&mut self, key: &str) -> usize {
        if let Some(&existing_value) = self.map.get(key) {
            return existing_value;
        }

        let index = self.names.len();
        self.map.insert(key.to_string(), index);
        self.names.push(key.to_string());
        index
    }

    pub fn get_max_value(&self) -> usize {
        self.names.len()
    }
}
