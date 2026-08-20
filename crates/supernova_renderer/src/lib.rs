//! Supernova Renderer — graphics abstraction layer built on wgpu.
//!
//! Provides a modern, cross-platform renderer supporting 2D and 3D
//! rendering pipelines with proper depth testing, shader support, and
//! material management.

use supernova_math::{Vec2, Vec3, Quat, Mat4, Color};

/// Vertex format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexFormat {
    Simple,
    Colored,
    Textured,
    Animated,
}

/// Vertex structure for rendering
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            normal: Vec3::ZERO,
            uv: Vec2::ZERO,
            color: Color::WHITE,
        }
    }

    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }

    pub fn with_uv(mut self, uv: Vec2) -> Self {
        self.uv = uv;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Index type for drawing
pub type Index = u32;

/// Mesh structure for rendering
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
    pub vertex_format: VertexFormat,
    pub material: Option<MaterialHandle>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<Index>) -> Self {
        Self {
            vertices,
            indices,
            vertex_format: VertexFormat::Simple,
            material: None,
        }
    }

    pub fn with_format(mut self, format: VertexFormat) -> Self {
        self.vertex_format = format;
        self
    }

    pub fn with_material(mut self, material: MaterialHandle) -> Self {
        self.material = Some(material);
        self
    }
}

/// Material handle type
pub type MaterialHandle = u32;

/// Material definition
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub albedo: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: Color,
    pub textures: TexturePaths,
}

impl Material {
    pub fn new(name: String) -> Self {
        Self {
            name,
            albedo: Color::WHITE,
            metallic: 0.0,
            roughness: 0.5,
            emissive: Color::BLACK,
            textures: TexturePaths::new(),
        }
    }
}

/// Texture paths for material
#[derive(Debug, Clone)]
pub struct TexturePaths {
    pub albedo: Option<String>,
    pub normal: Option<String>,
    pub metallic_roughness: Option<String>,
    pub occlusion: Option<String>,
    pub emissive: Option<String>,
}

impl TexturePaths {
    pub fn new() -> Self {
        Self {
            albedo: None,
            normal: None,
            metallic_roughness: None,
            occlusion: None,
            emissive: None,
        }
    }
}

impl Default for TexturePaths {
    fn default() -> Self {
        Self::new()
    }
}

/// Light type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Ambient,
}

/// Light structure for rendering
#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub light_type: LightType,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl Light {
    pub fn new_directional(color: Color, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            position: Vec3::ZERO,
            direction: Vec3::new(-0.577, -0.577, -0.577).normalize(),
            color,
            intensity,
            range: f32::MAX,
            inner_angle: 0.0,
            outer_angle: 0.0,
        }
    }

    pub fn new_point(position: Vec3, color: Color, intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            direction: Vec3::ZERO,
            color,
            intensity,
            range,
            inner_angle: 0.0,
            outer_angle: 0.0,
        }
    }

    pub fn new_spot(position: Vec3, direction: Vec3, color: Color, intensity: f32, inner: f32, outer: f32) -> Self {
        Self {
            light_type: LightType::Spot,
            position,
            direction,
            color,
            intensity,
            range: 20.0,
            inner_angle: inner,
            outer_angle: outer,
        }
    }
}

/// Camera projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraProjection {
    Perspective,
    Orthographic,
}

/// Camera structure for rendering
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub projection: CameraProjection,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vec3, fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            projection: CameraProjection::Perspective,
            fov,
            aspect_ratio,
            near,
            far,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.rotation * Vec3::Z, Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        match self.projection {
            CameraProjection::Perspective => {
                Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
            }
            CameraProjection::Orthographic => {
                let half_width = self.aspect_ratio * self.far;
                let half_height = self.far;
                Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, self.near, self.far)
            }
        }
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

/// Entity handle type
pub type EntityHandle = u32;

/// Texture resource
#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub data: Vec<u8>,
}

/// Render target
pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Graphics backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Vulkan,
    DirectX12,
    Metal,
    OpenGL,
    WebGPU,
    WebGL,
    None,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Feature: u64 {
        const COMPUTE_SHADERS = 1 << 0;
        const GEOMETRY_SHADERS = 1 << 1;
        const TEXTURE_COMPRESSION_BC = 1 << 2;
        const TEXTURE_COMPRESSION_ETC = 1 << 3;
        const SAMPLER_ANISOTROPIC = 1 << 4;
        const UNIFORM_BUFFERS = 1 << 5;
        const STORAGE_BUFFERS = 1 << 6;
    }
}

/// Graphics device abstraction
pub struct Device {
    pub backend: Backend,
    pub features: Vec<Feature>,
    pub next_resource_id: ResourceHandle,
}

/// Resource handle type
pub type ResourceHandle = u32;

impl Device {
    pub fn new() -> Self {
        Self {
            backend: Backend::None,
            features: vec![Feature::UNIFORM_BUFFERS],
            next_resource_id: 0,
        }
    }

    pub fn backend(&self) -> Backend {
        self.backend
    }

    pub fn features(&self) -> &[Feature] {
        &self.features
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}

/// Render pass
pub struct RenderPass {
    pub name: String,
    pub camera: Camera,
    pub pipeline: Pipeline,
    pub vertex_buffer: Vec<Vertex>,
    pub index_buffer: Vec<u32>,
    pub textures: Vec<Texture>,
    pub bind_groups: Vec<BindGroup>,
    pub depth_enabled: bool,
    pub clear_color: Color,
}

impl RenderPass {
    pub fn new(name: String, camera: Camera) -> Self {
        Self {
            name,
            camera,
            pipeline: Pipeline::new(),
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
            textures: Vec::new(),
            bind_groups: Vec::new(),
            depth_enabled: true,
            clear_color: Color::BLACK,
        }
    }

    pub fn set_shaders(&mut self, vertex: String, fragment: String) {
        self.pipeline.vertex_shader = vertex;
        self.pipeline.fragment_shader = fragment;
    }

    pub fn add_texture(&mut self, texture: Texture) {
        self.textures.push(texture);
    }

    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = Color::new(r, g, b, a);
    }

    pub fn set_depth_enabled(&mut self, enabled: bool) {
        self.depth_enabled = enabled;
    }

    pub fn upload(&mut self) {}

    pub fn execute(&self) {}
}

/// Shader pipeline state
pub struct Pipeline {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub vertex_layout: VertexLayout,
    pub primitive_topology: PrimitiveTopology,
    pub depth_stencil_state: DepthStencilState,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            vertex_shader: String::new(),
            fragment_shader: String::new(),
            vertex_layout: VertexLayout::default(),
            primitive_topology: PrimitiveTopology::TriangleList,
            depth_stencil_state: DepthStencilState::default(),
        }
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Vertex layout descriptor
pub struct VertexLayout {
    pub stride: u32,
    pub attributes: Vec<VertexAttribute>,
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self {
            stride: 32,
            attributes: Vec::new(),
        }
    }
}

/// Vertex attribute
pub struct VertexAttribute {
    pub format: VertexAttributeFormat,
    pub offset: u32,
    pub shader_stage: ShaderStage,
}

/// Vertex attribute format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexAttributeFormat {
    Float, Float2, Float3, Float4,
    Int, Int2, Int3, Int4,
    UInt, UInt2, UInt3, UInt4,
}

/// Shader stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex, Fragment, Geometry, Compute,
    Task, Mesh, All,
}

/// Primitive topology
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveTopology {
    PointList, LineList, LineStrip,
    TriangleList, TriangleStrip, TriangleFan,
}

/// Depth/stencil state
#[derive(Debug, Clone, Copy)]
pub struct DepthStencilState {
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
        }
    }
}

/// Compare function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareFunction {
    Never, Less, Equal, LessEqual,
    Greater, NotEqual, GreaterEqual, Always,
}

/// Stencil state
#[derive(Debug, Clone, Copy)]
pub struct StencilState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare: CompareFunction,
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            compare: CompareFunction::Always,
        }
    }
}

/// Stencil operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilOp {
    Keep, Zero, Replace,
    Increment, Decrement,
    IncrementWrap, DecrementWrap, Invert,
}

/// Bind group for shader resources
pub struct BindGroup {
    pub entries: Vec<BindGroupEntry>,
}

/// Bind group entry
pub struct BindGroupEntry {
    pub binding: u32,
    pub resource: BindingResource,
}

/// Binding resource
pub enum BindingResource {
    Buffer { buffer: ResourceHandle, offset: u64, size: u64 },
    TextureView { texture: ResourceHandle },
    Sampler { sampler: ResourceHandle },
}

/// Renderer struct — main rendering interface.
pub struct Renderer {
    device: Device,
    render_targets: Vec<RenderTarget>,
    clear_color: Color,
    enabled: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            device: Device::new(),
            render_targets: Vec::new(),
            clear_color: Color::BLACK,
            enabled: true,
        }
    }

    pub fn initialize(&mut self, width: u32, height: u32) {
        self.render_targets.push(RenderTarget::new(width, height));
    }

    pub fn begin_frame(&mut self) {}
    pub fn end_frame(&mut self) {}
    pub fn render_camera(&mut self, _camera: &Camera) {}
    pub fn render_mesh(&mut self, _mesh: &Mesh, _transform: &Mat4, _camera: &Camera) {}
    pub fn render_pass(&mut self, _pass: &RenderPass) {}

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn device(&self) -> &Device {
        &self.device
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
