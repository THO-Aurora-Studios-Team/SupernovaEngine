//! Supernova Scene — scene management and spatial components.

/// Entity handle type used throughout the scene system.
pub type EntityHandle = u32;

/// Transform component — position, rotation, and scale.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Translation (position)
    pub translation: supernova_math::Vec3,
    /// Rotation (quaternion)
    pub rotation: supernova_math::Quat,
    /// Scale
    pub scale: supernova_math::Vec3,
}

impl Transform {
    /// Create a transform at the origin.
    pub fn identity() -> Self {
        Self {
            translation: supernova_math::Vec3::ZERO,
            rotation: supernova_math::Quat::IDENTITY,
            scale: supernova_math::Vec3::ONE,
        }
    }

    /// Create a transform at the given position.
    pub fn from_position(translation: supernova_math::Vec3) -> Self {
        Self {
            translation,
            rotation: supernova_math::Quat::IDENTITY,
            scale: supernova_math::Vec3::ONE,
        }
    }

    /// Compute the 4x4 world matrix.
    pub fn matrix(&self) -> supernova_math::Mat4 {
        supernova_math::Mat4::from_translation(self.translation)
            * supernova_math::Mat4::from_quat(self.rotation)
            * supernova_math::Mat4::from_scale(self.scale)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Hierarchy parent component.
#[derive(Debug, Clone)]
pub struct Children {
    pub parent: u32,
    pub children: Vec<u32>,
}

impl Default for Children {
    fn default() -> Self {
        Self { parent: 0, children: Vec::new() }
    }
}

/// Tag component for grouping entities.
#[derive(Debug, Clone)]
pub struct Tag {
    pub tag: String,
}

impl Default for Tag {
    fn default() -> Self {
        Self { tag: String::new() }
    }
}

/// Scene structure for managing game objects.
#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<EntityHandle>,
    pub materials: Vec<supernova_renderer::Material>,
    pub lights: Vec<supernova_renderer::Light>,
    pub cameras: Vec<supernova_renderer::Camera>,
}

impl Scene {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entities: Vec::new(),
            materials: Vec::new(),
            lights: Vec::new(),
            cameras: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: EntityHandle) {
        self.entities.push(entity);
    }

    pub fn add_material(&mut self, material: supernova_renderer::Material) {
        self.materials.push(material);
    }

    pub fn add_light(&mut self, light: supernova_renderer::Light) {
        self.lights.push(light);
    }

    pub fn add_camera(&mut self, camera: supernova_renderer::Camera) {
        self.cameras.push(camera);
    }

    pub fn entity(&self, index: usize) -> Option<EntityHandle> {
        self.entities.get(index).copied()
    }

    pub fn material(&self, index: usize) -> Option<&supernova_renderer::Material> {
        self.materials.get(index)
    }

    pub fn light(&self, index: usize) -> Option<&supernova_renderer::Light> {
        self.lights.get(index)
    }

    pub fn camera(&self, index: usize) -> Option<&supernova_renderer::Camera> {
        self.cameras.get(index)
    }

    pub fn cameras(&self) -> &[supernova_renderer::Camera] {
        &self.cameras
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled".to_string())
    }
}

/// Scene manager for managing scenes.
#[derive(Debug, Default, Clone)]
pub struct SceneManager {
    current_scene: Option<Scene>,
    scene_map: std::collections::HashMap<String, EntityHandle>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, scene: Scene) {
        self.current_scene = Some(scene);
    }

    pub fn current_scene(&self) -> Option<&Scene> {
        self.current_scene.as_ref()
    }

    pub fn current_scene_mut(&mut self) -> Option<&mut Scene> {
        self.current_scene.as_mut()
    }

    pub fn unload(&mut self) {
        self.current_scene = None;
    }

    pub fn get_entity(&self, name: &str) -> Option<EntityHandle> {
        self.scene_map.get(name).copied()
    }

    pub fn add_entity(&mut self, name: String, handle: EntityHandle) {
        self.scene_map.insert(name, handle);
    }
}
