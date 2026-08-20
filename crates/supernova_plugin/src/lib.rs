//! Supernova Plugin System.
//!
//! Provides dynamic loading and management of plugins that can extend
//! engine functionality at runtime.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod exports {
    pub use super::*;
}

/// Plugin trait for all Supernova plugins.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, world: &mut supernova_core::World);
    fn shutdown(&mut self);
    fn update(&mut self, world: &mut supernova_core::World, delta_time: f32);
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
    fn hot_reloadable(&self) -> bool {
        false
    }
}

/// Plugin manager for loading and managing plugins.
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_paths: Vec<PathBuf>,
    loaded_order: Vec<String>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_paths: Vec::new(),
            loaded_order: Vec::new(),
        }
    }

    pub fn add_plugin_path<P: AsRef<Path>>(&mut self, path: P) {
        self.plugin_paths.push(path.as_ref().to_path_buf());
    }

    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<String, PluginError> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or(PluginError::InvalidPath)?
            .to_string();

        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded);
        }

        let plugin = DummyPlugin::new(&name);
        self.plugins.insert(name.clone(), Box::new(plugin));
        self.loaded_order.push(name.clone());

        Ok(name)
    }

    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if self.plugins.remove(name).is_none() {
            return Err(PluginError::NotFound);
        }

        let index = self
            .loaded_order
            .iter()
            .position(|n| n == name)
            .ok_or(PluginError::NotFound)?;
        self.loaded_order.remove(index);

        Ok(())
    }

    pub fn get_plugin<'a>(&'a self, name: &str) -> Option<&'a dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref() as &'a dyn Plugin)
    }

    pub fn get_plugin_mut<'a>(&'a mut self, name: &str) -> Option<&'a mut dyn Plugin> {
        self.plugins
            .get_mut(name)
            .map(|p| p.as_mut() as &'a mut dyn Plugin)
    }

    pub fn update(&mut self, world: &mut supernova_core::World, delta_time: f32) {
        for name in self.loaded_order.iter() {
            if let Some(plugin) = self.plugins.get_mut(name) {
                plugin.update(world, delta_time);
            }
        }
    }

    pub fn initialize(&mut self, world: &mut supernova_core::World) {
        let mut initialized: Vec<String> = Vec::new();

        loop {
            let mut progress = false;
            let pending: Vec<String> = self
                .plugins
                .keys()
                .filter(|name| !initialized.contains(name))
                .cloned()
                .collect();

            for name in &pending {
                let deps = {
                    let plugin = self.plugins.get(name.as_str()).unwrap();
                    plugin.dependencies()
                };
                let can_init = deps
                    .iter()
                    .all(|dep| initialized.iter().any(|s| s.as_str() == *dep));

                if can_init {
                    if let Some(plugin_mut) = self.plugins.get_mut(name) {
                        plugin_mut.initialize(world);
                    }
                    initialized.push(name.clone());
                    progress = true;
                }
            }

            if !progress {
                break;
            }
        }
    }

    pub fn shutdown(&mut self, world: &mut supernova_core::World) {
        for name in self.loaded_order.iter() {
            if let Some(plugin) = self.plugins.get_mut(name) {
                plugin.shutdown();
            }
        }
        let _ = world;
        self.plugins.clear();
        self.loaded_order.clear();
    }

    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    NotFound,
    AlreadyLoaded,
    InvalidPath,
    LoadFailed,
    Incompatible,
    Other(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::NotFound => write!(f, "Plugin not found"),
            PluginError::AlreadyLoaded => write!(f, "Plugin already loaded"),
            PluginError::InvalidPath => write!(f, "Invalid plugin path"),
            PluginError::LoadFailed => write!(f, "Failed to load plugin"),
            PluginError::Incompatible => write!(f, "Plugin incompatible"),
            PluginError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

/// Dummy plugin for testing
struct DummyPlugin {
    name: String,
}

impl DummyPlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Plugin for DummyPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn initialize(&mut self, _world: &mut supernova_core::World) {}
    fn shutdown(&mut self) {}
    fn update(&mut self, _world: &mut supernova_core::World, _delta_time: f32) {}
}
