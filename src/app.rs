use crate::config::RuntimeConfig;

pub trait AppContext: Send + Sync {
    fn config(&self) -> &RuntimeConfig;
}

pub struct App {
    config: RuntimeConfig,
}

impl App {
    pub fn new(config: RuntimeConfig) -> Self {
        Self { config }
    }
}

impl AppContext for App {
    fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}
