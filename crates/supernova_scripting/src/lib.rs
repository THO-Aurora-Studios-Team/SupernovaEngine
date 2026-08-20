//! Supernova Scripting Engine.
//!
//! Provides a multi-language scripting system supporting Lua (Luau), C#,
//! Python, and JavaScript. Scripts are executed in a sandboxed environment
//! and can interact with the ECS world through a safe API.

use std::collections::HashMap;

/// Entity handle type
pub type EntityHandle = u32;

/// Script handle for identifying loaded scripts
pub type ScriptHandle = u64;

/// Scripting error types
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptError {
    InvalidScript(String),
    ExecutionError(String),
    RuntimeError(String),
    CompileError(String),
    NotInitialized,
    UnknownLanguage(String),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::InvalidScript(msg) => write!(f, "Invalid script: {}", msg),
            ScriptError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            ScriptError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            ScriptError::CompileError(msg) => write!(f, "Compile error: {}", msg),
            ScriptError::NotInitialized => write!(f, "Scripting engine not initialized"),
            ScriptError::UnknownLanguage(lang) => write!(f, "Unknown scripting language: {}", lang),
        }
    }
}

impl std::error::Error for ScriptError {}

/// Supported scripting languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    Lua,
    Luau,
    CSharp,
    Python,
    JavaScript,
}

/// Script trait for all scripts
pub trait Script: Send + Sync {
    fn on_init(&mut self) {}
    fn on_update(&mut self, delta_time: f32) {}
    fn on_destroy(&mut self) {}
    fn language(&self) -> ScriptLanguage { ScriptLanguage::Lua }
}

/// Lua/Luau script implementation
pub struct LuaScript {
    code: String,
    active: bool,
    entity: Option<EntityHandle>,
}

impl LuaScript {
    pub fn new(code: String) -> Self {
        Self {
            code,
            active: true,
            entity: None,
        }
    }

    pub fn with_entity(code: String, entity: EntityHandle) -> Self {
        Self {
            code,
            active: true,
            entity: Some(entity),
        }
    }
}

impl Script for LuaScript {
    fn on_init(&mut self) {
        // Initialize Lua script
    }

    fn on_update(&mut self, delta_time: f32) {
        // Update Lua script
    }

    fn on_destroy(&mut self) {
        // Destroy Lua script
    }

    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::Lua
    }
}

/// C# script implementation
pub struct CSharpScript {
    assembly_path: String,
    active: bool,
    entity: Option<EntityHandle>,
}

impl CSharpScript {
    pub fn new(assembly_path: String) -> Self {
        Self {
            assembly_path,
            active: true,
            entity: None,
        }
    }

    pub fn with_entity(assembly_path: String, entity: EntityHandle) -> Self {
        Self {
            assembly_path,
            active: true,
            entity: Some(entity),
        }
    }
}

impl Script for CSharpScript {
    fn on_init(&mut self) {
        // Initialize C# script
    }

    fn on_update(&mut self, delta_time: f32) {
        // Update C# script
    }

    fn on_destroy(&mut self) {
        // Destroy C# script
    }

    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::CSharp
    }
}

/// Python script implementation
pub struct PythonScript {
    code: String,
    active: bool,
    entity: Option<EntityHandle>,
}

impl PythonScript {
    pub fn new(code: String) -> Self {
        Self {
            code,
            active: true,
            entity: None,
        }
    }

    pub fn with_entity(code: String, entity: EntityHandle) -> Self {
        Self {
            code,
            active: true,
            entity: Some(entity),
        }
    }
}

impl Script for PythonScript {
    fn on_init(&mut self) {
        // Initialize Python script
    }

    fn on_update(&mut self, delta_time: f32) {
        // Update Python script
    }

    fn on_destroy(&mut self) {
        // Destroy Python script
    }

    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::Python
    }
}

/// JavaScript script implementation
pub struct JavaScriptScript {
    code: String,
    active: bool,
    entity: Option<EntityHandle>,
}

impl JavaScriptScript {
    pub fn new(code: String) -> Self {
        Self {
            code,
            active: true,
            entity: None,
        }
    }

    pub fn with_entity(code: String, entity: EntityHandle) -> Self {
        Self {
            code,
            active: true,
            entity: Some(entity),
        }
    }
}

impl Script for JavaScriptScript {
    fn on_init(&mut self) {
        // Initialize JavaScript script
    }

    fn on_update(&mut self, delta_time: f32) {
        // Update JavaScript script
    }

    fn on_destroy(&mut self) {
        // Destroy JavaScript script
    }

    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::JavaScript
    }
}

/// Script instance
pub struct ScriptInstance {
    script: Box<dyn Script>,
    entity: EntityHandle,
    handle: ScriptHandle,
}

impl ScriptInstance {
    pub fn new(script: Box<dyn Script>, entity: EntityHandle, handle: ScriptHandle) -> Self {
        Self { script, entity, handle }
    }

    pub fn initialize(&mut self) {
        self.script.on_init();
    }

    pub fn update(&mut self, delta_time: f32) {
        self.script.on_update(delta_time);
    }

    pub fn destroy(&mut self) {
        self.script.on_destroy();
    }

    pub fn handle(&self) -> ScriptHandle {
        self.handle
    }

    pub fn entity(&self) -> EntityHandle {
        self.entity
    }
}

/// Scripting engine for running scripts
pub struct ScriptingEngine {
    scripts: Vec<ScriptInstance>,
    enabled: bool,
    next_handle: ScriptHandle,
    languages: HashMap<String, ScriptLanguage>,
}

impl ScriptingEngine {
    pub fn new() -> Self {
        Self {
            scripts: Vec::new(),
            enabled: true,
            next_handle: 0,
            languages: HashMap::new(),
        }
    }

    pub fn add_script(&mut self, script: Box<dyn Script>, entity: EntityHandle) -> ScriptHandle {
        let handle = self.next_handle;
        self.next_handle += 1;
        let instance = ScriptInstance::new(script, entity, handle);
        self.scripts.push(instance);
        handle
    }

    pub fn remove_script(&mut self, entity: EntityHandle) {
        self.scripts.retain(|instance| instance.entity != entity);
    }

    pub fn remove_script_by_handle(&mut self, handle: ScriptHandle) {
        self.scripts.retain(|instance| instance.handle != handle);
    }

    /// Update all script instances
    pub fn update(&mut self, _world: &mut supernova_core::World, delta_time: f32) {
        for instance in &mut self.scripts {
            instance.update(delta_time);
        }
    }

    pub fn initialize(&mut self) {
        for instance in &mut self.scripts {
            instance.initialize();
        }
    }

    pub fn destroy(&mut self) {
        for instance in &mut self.scripts {
            instance.destroy();
        }
        self.scripts.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Register a script language
    pub fn register_language(&mut self, name: &str, language: ScriptLanguage) {
        self.languages.insert(name.to_string(), language);
    }
}

impl Default for ScriptingEngine {
    fn default() -> Self {
        Self::new()
    }
}
