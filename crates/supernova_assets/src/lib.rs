//! Supernova Assets — asset loading and management.
//!
//! Provides a flexible, type-safe asset manager with hot-reloading support.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Trait for deserializing assets from raw data.
pub trait FromBytes: Sized {
    fn from_bytes(data: &[u8]) -> Result<Self, AssetError>;
}

/// Error type for asset operations.
#[derive(Debug, Clone)]
pub enum AssetError {
    IOError(String),
    ParseError(String),
    NotFound(String),
    Locked,
    Other(String),
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::IOError(msg) => write!(f, "IO error: {}", msg),
            AssetError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AssetError::NotFound(msg) => write!(f, "Asset not found: {}", msg),
            AssetError::Locked => write!(f, "Asset is locked"),
            AssetError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AssetError {}

/// Asset handle for referencing loaded assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetHandle {
    id: u64,
    version: u32,
}

impl AssetHandle {
    pub fn new(id: u64) -> Self {
        Self { id, version: 0 }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl Default for AssetHandle {
    fn default() -> Self {
        Self { id: 0, version: 0 }
    }
}

/// Asset entry in the registry.
struct AssetEntry {
    data: Arc<RwLock<Vec<u8>>>,
    path: PathBuf,
    modified: std::time::SystemTime,
}

/// Asset manager for loading and managing game assets.
pub struct AssetManager {
    /// Assets registry (handle -> entry)
    assets: HashMap<AssetHandle, AssetEntry>,
    /// Path to asset handle mapping
    path_map: HashMap<PathBuf, AssetHandle>,
    /// Asset type names
    asset_types: HashMap<AssetHandle, &'static str>,
    /// Next available ID
    next_id: u64,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_map: HashMap::new(),
            asset_types: HashMap::new(),
            next_id: 1,
        }
    }

    /// Generate a new unique asset handle.
    fn next_handle(&mut self) -> AssetHandle {
        let id = self.next_id;
        self.next_id += 1;
        AssetHandle::new(id)
    }

    /// Load an asset from a file path.
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<AssetHandle, AssetError> {
        let path = path.as_ref().to_path_buf();
        if let Some(handle) = self.path_map.get(&path) {
            return Ok(*handle);
        }

        let data = std::fs::read(&path)
            .map_err(|e| AssetError::IOError(e.to_string()))?;

        let handle = self.next_handle();
        let entry = AssetEntry {
            data: Arc::new(RwLock::new(data)),
            path: path.clone(),
            modified: std::time::SystemTime::now(),
        };

        self.assets.insert(handle, entry);
        self.path_map.insert(path, handle);
        Ok(handle)
    }

    /// Load an asset from raw data.
    pub fn load_from_bytes(&mut self, data: Vec<u8>, name: &str) -> AssetHandle {
        let handle = self.next_handle();
        let entry = AssetEntry {
            data: Arc::new(RwLock::new(data)),
            path: PathBuf::from(name),
            modified: std::time::SystemTime::now(),
        };
        self.assets.insert(handle, entry);
        handle
    }

    /// Get raw asset data by handle.
    pub fn get_raw(&self, handle: AssetHandle) -> Option<Arc<RwLock<Vec<u8>>> > {
        self.assets.get(&handle).map(|e| e.data.clone())
    }

    /// Deserialize an asset from raw data.
    pub fn get<T: FromBytes>(&self, handle: AssetHandle) -> Result<T, AssetError> {
        let entry = self.assets.get(&handle).ok_or(AssetError::NotFound("Handle not found".into()))?;
        let data = entry.data.read().unwrap();
        T::from_bytes(&data)
    }

    /// Save asset data to a file.
    pub fn save<P: AsRef<Path>>(&mut self, path: P, data: &[u8]) -> Result<(), AssetError> {
        let path = path.as_ref().to_path_buf();
        std::fs::write(&path, data)
            .map_err(|e| AssetError::IOError(e.to_string()))?;

        let handle = self.next_handle();
        let entry = AssetEntry {
            data: Arc::new(RwLock::new(data.to_vec())),
            path: path.clone(),
            modified: std::time::SystemTime::now(),
        };
        self.assets.insert(handle, entry);
        self.path_map.insert(path, handle);
        Ok(())
    }

    /// Check if an asset exists.
    pub fn exists(&self, handle: AssetHandle) -> bool {
        self.assets.contains_key(&handle)
    }

    /// Get the number of loaded assets.
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Unload an asset.
    pub fn unload(&mut self, handle: AssetHandle) -> bool {
        if self.assets.remove(&handle).is_some() {
            // Remove from path map
            self.path_map.retain(|_, v| *v != handle);
            self.asset_types.remove(&handle);
            true
        } else {
            false
        }
    }

    /// Reload assets that have changed on disk.
    pub fn reload_changed(&mut self) -> Result<(), AssetError> {
        for (handle, entry) in &mut self.assets {
            let new_modified = std::fs::metadata(&entry.path)
                .map(|m| m.modified().unwrap_or(std::time::SystemTime::now()))
                .unwrap_or(std::time::SystemTime::now());

            if new_modified > entry.modified {
                let data = std::fs::read(&entry.path)
                    .map_err(|e| AssetError::IOError(e.to_string()))?;

                let mut write = entry.data.write().unwrap();
                write.clear();
                write.extend_from_slice(&data);
                entry.modified = new_modified;
            }
        }
        Ok(())
    }

    /// Get the path of an asset.
    pub fn path(&self, handle: AssetHandle) -> Option<&Path> {
        self.assets.get(&handle).map(|e| e.path.as_path())
    }

    /// Get the type name of an asset.
    pub fn type_name(&self, handle: AssetHandle) -> Option<&'static str> {
        self.asset_types.get(&handle).copied()
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
