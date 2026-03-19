use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single saved server entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
    pub last_connected: Option<String>,
}

impl ServerEntry {
    pub fn new(name: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            address: address.into(),
            last_connected: None,
        }
    }
}

/// The full list persisted to servers.toml.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServerList {
    #[serde(default)]
    pub servers: Vec<ServerEntry>,
}

impl ServerList {
    /// Returns the path to servers.toml using platform directories.
    fn config_path() -> PathBuf {
        // Use dirs crate for platform-appropriate config dir.
        // Falls back to current directory if dirs is unavailable.
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("alembic").join("servers.toml")
    }

    /// Loads from disk. Returns an empty list (with localhost default) if
    /// the file doesn't exist yet.
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            // First run — seed with a localhost entry so there's something
            // to click immediately during development.
            ServerList {
                servers: vec![ServerEntry::new("Local Dev Server", "127.0.0.1:7777")],
            }
        }
    }

    /// Saves to disk. Creates directories if they don't exist.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Adds a server if the address isn't already in the list.
    pub fn add(&mut self, name: impl Into<String>, address: impl Into<String>) {
        let address = address.into();
        if !self.servers.iter().any(|s| s.address == address) {
            self.servers.push(ServerEntry::new(name, address));
            let _ = self.save();
        }
    }

    /// Removes a server by index.
    pub fn remove(&mut self, index: usize) {
        if index < self.servers.len() {
            self.servers.remove(index);
            let _ = self.save();
        }
    }

    /// Updates the last_connected timestamp for a server by address.
    pub fn touch(&mut self, address: &str) {
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        if let Some(entry) = self.servers.iter_mut().find(|s| s.address == address) {
            entry.last_connected = Some(now);
            let _ = self.save();
        }
    }
}
