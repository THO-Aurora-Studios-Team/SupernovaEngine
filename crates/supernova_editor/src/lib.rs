//! Supernova Editor module for Supernova Engine.
//!
//! The editor provides tools for creating and modifying game scenes
//! and assets. It integrates with the core engine and renderer.

use std::path::{Path, PathBuf};

/// Editor application state
pub struct EditorApp {
    /// Current scene being edited
    current_scene: Option<PathBuf>,
    /// Window state
    window_width: u32,
    window_height: u32,
    /// UI state
    menu_bar: bool,
    properties_panel: bool,
    hierarchy_panel: bool,
    inspector_panel: bool,
    /// Tool state
    selected_tool: EditorTool,
    /// Scene data
    scene_data: SceneData,
}

impl EditorApp {
    pub fn new() -> Self {
        Self {
            current_scene: None,
            window_width: 1920,
            window_height: 1080,
            menu_bar: true,
            properties_panel: true,
            hierarchy_panel: true,
            inspector_panel: true,
            selected_tool: EditorTool::Select,
            scene_data: SceneData {
                name: "Untitled".to_string(),
                entities: Vec::new(),
                materials: Vec::new(),
                lights: Vec::new(),
                cameras: Vec::new(),
            },
        }
    }

    pub fn run(&mut self) {
        println!("Supernova Editor v0.1.0");
        println!("===========================");
        // Main editor loop would go here
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Editor tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Select,
    Move,
    Rotate,
    Scale,
    Pan,
    Orbit,
    Brush,
    Eraser,
}

/// Scene data structure
pub struct SceneData {
    /// Scene name
    pub name: String,
    /// Root entities
    pub entities: Vec<EntityHandle>,
    /// Materials
    pub materials: Vec<Material>,
    /// Lighting
    pub lights: Vec<Light>,
    /// Camera
    pub cameras: Vec<Camera>,
}

/// Entity handle
pub type EntityHandle = u32;

/// Material definition
pub struct Material {
    /// Material name
    pub name: String,
    /// Albedo color
    pub albedo: [f32; 4],
    /// Metallic value
    pub metallic: f32,
    /// Roughness value
    pub roughness: f32,
    /// Emissive color
    pub emissive: [f32; 3],
    /// Texture paths
    pub textures: TexturePaths,
}

/// Texture paths for material
pub struct TexturePaths {
    /// Albedo texture
    pub albedo: Option<PathBuf>,
    /// Normal texture
    pub normal: Option<PathBuf>,
    /// Metallic roughness texture
    pub metallic_roughness: Option<PathBuf>,
    /// Occlusion texture
    pub occlusion: Option<PathBuf>,
    /// Emissive texture
    pub emissive: Option<PathBuf>,
}

/// Light definition
pub struct Light {
    /// Light type
    pub light_type: LightType,
    /// Position
    pub position: [f32; 3],
    /// Color
    pub color: [f32; 3],
    /// Intensity
    pub intensity: f32,
    /// Range
    pub range: f32,
    /// Inner angle
    pub inner_angle: f32,
    /// Outer angle
    pub outer_angle: f32,
}

/// Light type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Ambient,
}

/// Camera definition
pub struct Camera {
    /// Camera position
    pub position: [f32; 3],
    /// Camera rotation
    pub rotation: [f32; 4],
    /// Camera projection
    pub projection: CameraProjection,
    /// Camera field of view
    pub fov: f32,
    /// Camera aspect ratio
    pub aspect_ratio: f32,
    /// Camera near plane
    pub near: f32,
    /// Camera far plane
    pub far: f32,
}

/// Camera projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraProjection {
    Perspective,
    Orthographic,
}
