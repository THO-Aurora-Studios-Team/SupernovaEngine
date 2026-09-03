//! Supernova Engine — main engine crate.
//!
//! Aggregates all subsystem crates into a unified engine facade with
//! real-time wgpu rendering and a lock-free IPC bridge for editor integration.

pub use supernova_assets as assets;
pub use supernova_audio as audio;
pub use supernova_core as core;
pub use supernova_input as input;
pub use supernova_math as math;
pub use supernova_network as network;
pub use supernova_physics as physics;
pub use supernova_plugin as plugin;
pub use supernova_renderer as renderer;
pub use supernova_scene as scene;
pub use supernova_scripting as scripting;

use std::sync::Arc;

use crossbeam_channel::{bounded, Receiver, Sender};
use supernova_assets::AssetManager;
use supernova_audio::AudioEngine;
use supernova_core::App;
use supernova_input::InputHandler;
use supernova_network::NetworkStack;
use supernova_physics::PhysicsEngine;
use supernova_plugin::PluginManager;
use supernova_renderer::Renderer;
use supernova_scene::SceneManager;
use supernova_scripting::ScriptingEngine;

// ────────────────────────────────────────────────────────────────────────────
// IPC Protocol — lock-free channel bridge between editor ↔ engine
// ────────────────────────────────────────────────────────────────────────────

/// Commands the editor sends to the engine.
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Spawn an entity with an optional name.
    SpawnEntity { name: String },
    /// Despawn an entity by index.
    DespawnEntity { entity_index: u32 },
    /// Set an entity's translation.
    SetTranslation {
        entity_index: u32,
        x: f32,
        y: f32,
        z: f32,
    },
    /// Set an entity's scale.
    SetScale {
        entity_index: u32,
        x: f32,
        y: f32,
        z: f32,
    },
    /// Set an entity's rotation (Euler angles in radians).
    SetRotation {
        entity_index: u32,
        pitch: f32,
        yaw: f32,
        roll: f32,
    },
    /// Play / pause the simulation.
    SetPaused(bool),
    /// Step a single frame (only meaningful while paused).
    StepFrame,
    /// Set gravity vector.
    SetGravity { x: f32, y: f32, z: f32 },
    /// Request the engine to shut down.
    Shutdown,
    /// Send a console command string.
    ConsoleCommand(String),
    /// Resize the render viewport (width, height).
    ResizeViewport { width: u32, height: u32 },
}

/// Events the engine sends back to the editor.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// A frame was rendered.  Contains RGBA pixel data, dimensions, and timing.
    Frame {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        frame_number: u64,
        dt_ms: f32,
    },
    /// Entity list snapshot.
    EntityList {
        entities: Vec<EntitySnapshot>,
    },
    /// Console log line.
    ConsoleLog {
        level: LogLevel,
        message: String,
    },
    /// Engine has fully initialized.
    Initialized {
        gpu_name: String,
        backend: String,
    },
    /// Engine is shutting down.
    Shutdown,
    /// Physics step completed.
    PhysicsStepDone {
        bodies_active: u32,
        collisions: u32,
    },
}

/// Lightweight entity snapshot for the hierarchy panel.
#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    pub index: u32,
    pub name: String,
    pub has_transform: bool,
    pub has_rigid_body: bool,
    pub has_collider: bool,
}

/// Log level for console messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Channels connecting the editor to the engine.
pub struct IpcChannels {
    /// Editor → Engine command sender (editor holds the Receiver side).
    pub command_tx: Sender<EngineCommand>,
    pub command_rx: Receiver<EngineCommand>,
    /// Engine → Editor event sender (editor holds the Receiver side).
    pub event_tx: Sender<EngineEvent>,
    pub event_rx: Receiver<EngineEvent>,
}

impl IpcChannels {
    /// Create a new pair of bounded channels (capacity 64 each).
    pub fn new() -> Self {
        let (command_tx, command_rx) = bounded(64);
        let (event_tx, event_rx) = bounded(64);
        Self {
            command_tx,
            command_rx,
            event_tx,
            event_rx,
        }
    }
}

impl Default for IpcChannels {
    fn default() -> Self {
        Self::new()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// GPU Context — wgpu device + surface management
// ────────────────────────────────────────────────────────────────────────────

/// Persistent wgpu state shared between engine subsystems.
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
}

impl GpuContext {
    /// Create a GPU context from a winit window.
    ///
    /// Prefers integrated GPUs for power efficiency.
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Supernova Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            }, None)
            .await
            .expect("Failed to create device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Self {
            device,
            queue,
            surface,
            surface_config,
            surface_format: format,
        }
    }

    /// Reconfigure the surface after a resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    /// Begin a frame — returns the output texture view.
    pub fn begin_frame(&self) -> Option<wgpu::SurfaceTexture> {
        self.surface.get_current_texture().ok()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Render pipeline — simple colored geometry for the engine viewport
// ────────────────────────────────────────────────────────────────────────────

/// GPU-side render state (vertex buffers, pipelines, etc.).
pub struct ViewportRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    _pad: [f32; 3],
}

const VERTEX_SHADER_SRC: &str = r#"
struct Uniforms {
    time: f32,
};
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let angle = uniforms.time * 0.5;
    let s = sin(angle);
    let c = cos(angle);
    let rot = mat3x3<f32>(
        vec3<f32>(c, s, 0.0),
        vec3<f32>(-s, c, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    );
    out.clip_position = vec4<f32>(rot * in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

impl ViewportRenderer {
    pub fn new(gpu: &GpuContext) -> Self {
        use wgpu::util::DeviceExt;

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Engine Shader"),
                source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER_SRC.into()),
            });

        let uniform_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                time: 0.0,
                _pad: [0.0; 3],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Engine Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Engine Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        // Triangle + a rotating quad to make it visually interesting.
        let vertices = vec![
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [1.0, 0.2, 0.2],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [0.2, 1.0, 0.2],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [0.2, 0.2, 1.0],
            },
            // Quad
            Vertex {
                position: [-0.8, 0.8, 0.0],
                color: [1.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.4, 0.8, 0.0],
                color: [0.0, 1.0, 1.0],
            },
            Vertex {
                position: [-0.4, 0.4, 0.0],
                color: [1.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.8, 0.4, 0.0],
                color: [1.0, 1.0, 1.0],
            },
        ];

        let indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 3, 5, 6];

        let vertex_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            bind_group,
            uniform_buffer,
        }
    }

    /// Encode a render pass into the given texture view.
    pub fn render(
        &self,
        gpu: &GpuContext,
        view: &wgpu::TextureView,
        time: f32,
    ) {
        gpu.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                time,
                _pad: [0.0; 3],
            }]),
        );

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Engine Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Engine Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Engine state
// ────────────────────────────────────────────────────────────────────────────

/// Engine statistics reported each frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct EngineStats {
    pub frame_number: u64,
    pub fps: f32,
    pub dt_ms: f32,
    pub entities_active: u32,
}

/// Main engine structure that holds all subsystems.
pub struct SupernovaEngine {
    app: App,
    renderer: Renderer,
    physics: PhysicsEngine,
    input: InputHandler,
    audio: AudioEngine,
    scene_manager: SceneManager,
    asset_manager: AssetManager,
    network: NetworkStack,
    scripting: ScriptingEngine,
    plugin_manager: PluginManager,
    running: bool,
    paused: bool,
    dt: f32,
    stats: EngineStats,
}

impl SupernovaEngine {
    pub fn new() -> Self {
        Self {
            app: App::new(),
            renderer: Renderer::new(),
            physics: PhysicsEngine::new(),
            input: InputHandler::new(),
            audio: AudioEngine::new(),
            scene_manager: SceneManager::new(),
            asset_manager: AssetManager::new(),
            network: NetworkStack::new(),
            scripting: ScriptingEngine::new(),
            plugin_manager: PluginManager::new(),
            running: false,
            paused: false,
            dt: 0.0,
            stats: EngineStats::default(),
        }
    }

    /// Get mutable access to the world
    pub fn world_mut(&mut self) -> &mut supernova_core::World {
        &mut self.app.world
    }

    /// Get mutable access to the renderer
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    /// Get mutable access to the input handler
    pub fn input_mut(&mut self) -> &mut InputHandler {
        &mut self.input
    }

    /// Get mutable access to the physics engine
    pub fn physics_mut(&mut self) -> &mut PhysicsEngine {
        &mut self.physics
    }

    pub fn initialize(&mut self) {
        let time = supernova_core::Time::default();
        self.app.insert_resource(time);
        self.plugin_manager.initialize(&mut self.app.world);
        self.running = true;
        log::info!("Supernova Engine initialized");
    }

    /// Run one simulation step (physics, scripting, audio, ECS).
    pub fn update(&mut self, dt: f32) {
        self.dt = dt;
        self.stats.frame_number += 1;
        self.stats.dt_ms = dt * 1000.0;
        self.stats.fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
        self.stats.entities_active = self.app.world.entity_count() as u32;

        if !self.paused {
            self.input.update();
            self.plugin_manager.update(&mut self.app.world, dt);
            self.scripting.update(&mut self.app.world, dt);
            self.physics.update(&mut self.app.world, dt);
            self.audio.update();
            self.app.update(dt);
            self.network.update(dt);
        }
    }

    /// Process incoming IPC commands. Returns false when Shutdown is received.
    pub fn process_commands(&mut self, rx: &Receiver<EngineCommand>) -> bool {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                EngineCommand::Shutdown => {
                    self.stop();
                    return false;
                }
                EngineCommand::SpawnEntity { name } => {
                    let e = self.app.world.spawn();
                    self.app.world.insert(
                        e,
                        supernova_scene::Transform::identity(),
                    );
                    log::info!("Spawned entity '{}' at {:?}", name, e);
                }
                EngineCommand::DespawnEntity { entity_index } => {
                    let e = supernova_core::Entity::new(entity_index, 0);
                    self.app.world.despawn(e);
                }
                EngineCommand::SetTranslation {
                    entity_index,
                    x,
                    y,
                    z,
                } => {
                    let e = supernova_core::Entity::new(entity_index, 0);
                    if let Some(t) = self.app.world.get_mut::<supernova_scene::Transform>(e) {
                        t.translation = supernova_math::Vec3::new(x, y, z);
                    }
                }
                EngineCommand::SetScale {
                    entity_index,
                    x,
                    y,
                    z,
                } => {
                    let e = supernova_core::Entity::new(entity_index, 0);
                    if let Some(t) = self.app.world.get_mut::<supernova_scene::Transform>(e) {
                        t.scale = supernova_math::Vec3::new(x, y, z);
                    }
                }
                EngineCommand::SetRotation {
                    entity_index,
                    pitch,
                    yaw,
                    roll,
                } => {
                    let e = supernova_core::Entity::new(entity_index, 0);
                    if let Some(t) = self.app.world.get_mut::<supernova_scene::Transform>(e) {
                        use supernova_math::Quat;
                        t.rotation = Quat::from_rotation_y(yaw)
                            * Quat::from_rotation_x(pitch)
                            * Quat::from_rotation_z(roll);
                    }
                }
                EngineCommand::SetPaused(p) => {
                    self.paused = p;
                }
                EngineCommand::StepFrame => {
                    if self.paused {
                        self.update(1.0 / 60.0);
                    }
                }
                EngineCommand::SetGravity { x, y, z } => {
                    self.physics
                        .set_gravity(supernova_math::Vec3::new(x, y, z));
                }
                EngineCommand::ConsoleCommand(cmd) => {
                    log::info!("Console: {}", cmd);
                }
                EngineCommand::ResizeViewport { width, height } => {
                    // Handled externally via GPU context resize.
                    log::debug!("Viewport resize requested: {}x{}", width, height);
                }
            }
        }
        true
    }

    /// Read frame data from the GPU back-buffer and send it through the IPC channel.
    pub fn readback_and_send_frame(
        &self,
        gpu: &GpuContext,
        texture: &wgpu::Texture,
        event_tx: &Sender<EngineEvent>,
    ) {
        let width = texture.width();
        let height = texture.height();

        let output_buffer_size = (width * height * 4) as wgpu::BufferAddress;
        let output_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Frame Readback Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Copy Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        gpu.queue.submit(std::iter::once(encoder.finish()));

        let slice = output_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).ok();
        });
        gpu.device.poll(wgpu::Maintain::Wait);

        if receiver.recv().ok().and_then(|r| r.ok()).is_some() {
            let data = slice.get_mapped_range().to_vec();
            let _ = event_tx.try_send(EngineEvent::Frame {
                rgba: data,
                width,
                height,
                frame_number: self.stats.frame_number,
                dt_ms: self.stats.dt_ms,
            });
            drop(output_buffer);
        }
    }

    /// Collect all living entities into a snapshot list.
    pub fn entity_snapshots(&self) -> Vec<EntitySnapshot> {
        self.app
            .world
            .entities()
            .map(|e| EntitySnapshot {
                index: e.index(),
                name: format!("Entity_{}", e.index()),
                has_transform: self.app.world.has::<supernova_scene::Transform>(e),
                has_rigid_body: false,
                has_collider: false,
            })
            .collect()
    }

    pub fn stats(&self) -> EngineStats {
        self.stats
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.plugin_manager.shutdown(&mut self.app.world);
        log::info!("Supernova Engine stopped");
    }
}

impl Default for SupernovaEngine {
    fn default() -> Self {
        Self::new()
    }
}
