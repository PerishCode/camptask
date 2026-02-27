#[derive(Debug, Clone)]
pub struct App {
    name: String,
}

impl App {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
