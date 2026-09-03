//! Supernova Editor — full egui-based editor with wgpu rendering.
//!
//! Manages the editor UI (hierarchy, inspector, console, viewport),
//! communicates with the engine over lock-free crossbeam channels,
//! and renders an egui viewport fed by engine-produced frame data.

use std::sync::Arc;
use std::thread;
use std::time::Instant;

use crossbeam_channel::{bounded, Receiver, Sender};
use supernova_core::Entity;
use supernova_engine::{
    EngineCommand, EngineEvent, EntitySnapshot, LogLevel, SupernovaEngine,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

// ────────────────────────────────────────────────────────────────────────────
// Editor application
// ────────────────────────────────────────────────────────────────────────────

/// Top-level editor application.
pub struct EditorApp {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    egui_state: Option<EguiState>,

    // Editor panel state
    show_hierarchy: bool,
    show_inspector: bool,
    show_console: bool,
    show_toolbar: bool,
    selected_entity: Option<u32>,
    entities: Vec<EntitySnapshot>,
    console_log: Vec<(LogLevel, String)>,
    paused: bool,

    // Engine IPC
    engine_cmd_tx: Option<Sender<EngineCommand>>,
    engine_evt_rx: Option<Receiver<EngineEvent>>,
    viewport_rgba: Vec<u8>,
    viewport_size: (u32, u32),
    engine_frame: u64,
    engine_dt_ms: f32,
    engine_connected: bool,
    last_frame: Instant,
}

impl EditorApp {
    pub fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            egui_state: None,
            show_hierarchy: true,
            show_inspector: true,
            show_console: true,
            show_toolbar: true,
            selected_entity: None,
            entities: Vec::new(),
            console_log: Vec::new(),
            paused: false,
            engine_cmd_tx: None,
            engine_evt_rx: None,
            viewport_rgba: Vec::new(),
            viewport_size: (800, 600),
            engine_frame: 0,
            engine_dt_ms: 0.0,
            engine_connected: false,
            last_frame: Instant::now(),
        }
    }

    /// Spawn the engine on a background thread and wire up IPC channels.
    fn spawn_engine_thread(&mut self) {
        // Create the actual IPC channel pairs.
        let (cmd_tx, cmd_rx) = bounded::<EngineCommand>(64);
        let (evt_tx, evt_rx) = bounded::<EngineEvent>(64);

        // Store the sending/receiving ends that the editor will use.
        self.engine_cmd_tx = Some(cmd_tx);
        self.engine_evt_rx = Some(evt_rx);

        thread::spawn(move || {
            log::info!("[Engine Thread] Starting engine");
            let mut engine = SupernovaEngine::new();
            engine.initialize();

            // Create a demo scene
            {
                use supernova_math::Vec3;
                use supernova_scene::Transform;
                let world = engine.world_mut();
                let _player = world.spawn_with((Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),));
                let _floor = world.spawn_with((Transform::from_position(Vec3::new(0.0, -2.0, 0.0)),));
            }

            let _ = evt_tx.send(EngineEvent::Initialized {
                gpu_name: "Headless (editor-owned)".into(),
                backend: "None".into(),
            });

            let mut last_time = std::time::Instant::now();
            loop {
                let now = std::time::Instant::now();
                let dt = (now - last_time).as_secs_f32();
                last_time = now;

                // Process commands from the editor (lock-free, non-blocking).
                let mut alive = true;
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        EngineCommand::Shutdown => {
                            alive = false;
                            break;
                        }
                        EngineCommand::SpawnEntity { name } => {
                            let e = engine.world_mut().spawn();
                            engine.world_mut().insert(
                                e,
                                supernova_scene::Transform::identity(),
                            );
                            let _ = evt_tx.send(EngineEvent::ConsoleLog {
                                level: LogLevel::Info,
                                message: format!("Spawned entity '{}' ({})", name, e.index()),
                            });
                        }
                        EngineCommand::DespawnEntity { entity_index } => {
                            let e = Entity::new(entity_index, 0);
                            engine.world_mut().despawn(e);
                        }
                        EngineCommand::SetTranslation {
                            entity_index,
                            x,
                            y,
                            z,
                        } => {
                            let e = Entity::new(entity_index, 0);
                            if let Some(t) =
                                engine.world_mut().get_mut::<supernova_scene::Transform>(e)
                            {
                                t.translation = supernova_math::Vec3::new(x, y, z);
                            }
                        }
                        EngineCommand::SetScale {
                            entity_index,
                            x,
                            y,
                            z,
                        } => {
                            let e = Entity::new(entity_index, 0);
                            if let Some(t) =
                                engine.world_mut().get_mut::<supernova_scene::Transform>(e)
                            {
                                t.scale = supernova_math::Vec3::new(x, y, z);
                            }
                        }
                        EngineCommand::SetRotation {
                            entity_index,
                            pitch,
                            yaw,
                            roll,
                        } => {
                            let e = Entity::new(entity_index, 0);
                            if let Some(t) =
                                engine.world_mut().get_mut::<supernova_scene::Transform>(e)
                            {
                                use supernova_math::Quat;
                                t.rotation = Quat::from_rotation_y(yaw)
                                    * Quat::from_rotation_x(pitch)
                                    * Quat::from_rotation_z(roll);
                            }
                        }
                        EngineCommand::SetPaused(p) => {
                            // Engine thread doesn't maintain pause state; editor is the source of truth.
                            let _ = p;
                        }
                        EngineCommand::StepFrame => {
                            engine.update(1.0 / 60.0);
                        }
                        EngineCommand::SetGravity { x, y, z } => {
                            engine
                                .physics_mut()
                                .set_gravity(supernova_math::Vec3::new(x, y, z));
                        }
                        EngineCommand::ConsoleCommand(ref cmd) => {
                            let _ = evt_tx.send(EngineEvent::ConsoleLog {
                                level: LogLevel::Info,
                                message: format!("[Engine] {}", cmd),
                            });
                        }
                        EngineCommand::ResizeViewport { .. } => {}
                    }
                }

                if !alive {
                    let _ = evt_tx.send(EngineEvent::Shutdown);
                    break;
                }

                // Step simulation
                engine.update(dt);

                // Send entity list
                let snapshots = engine.entity_snapshots();
                let _ = evt_tx.send(EngineEvent::EntityList {
                    entities: snapshots,
                });
            }

            log::info!("[Engine Thread] Engine stopped");
        });
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// GPU + egui state (editor-side wgpu surface)
// ────────────────────────────────────────────────────────────────────────────

struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
}

impl GpuContext {
    async fn new(window: Arc<Window>) -> Self {
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
            .expect("No suitable GPU adapter found");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Editor Device"),
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

    fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.surface_config.width = w;
            self.surface_config.height = h;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

struct EguiState {
    ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
    texture: Option<egui::TextureHandle>,
}

impl EguiState {
    fn new(gpu: &GpuContext) -> Self {
        let ctx = egui::Context::default();
        let renderer =
            egui_wgpu::Renderer::new(&gpu.device, gpu.surface_format, None, 1, false);
        Self {
            ctx,
            renderer,
            texture: None,
        }
    }

    fn update_viewport_texture(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) {
        if rgba.is_empty() || width == 0 || height == 0 {
            return;
        }
        // Convert RGBA8 into egui texture format (packed u32 per pixel).
        let pixels: Vec<u8> = rgba.to_vec();

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &pixels,
        );

        // Replace the viewport texture each frame. Dropping the old handle
        // at frame start signals egui to free the previous GPU texture.
        self.texture = Some(self.ctx.load_texture(
            "viewport",
            image,
            egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                ..Default::default()
            },
        ));
    }
}

// ────────────────────────────────────────────────────────────────────────────
// UI Panels
// ────────────────────────────────────────────────────────────────────────────

impl EditorApp {
    fn build_ui(&mut self, ctx: &egui::Context) {
        self.build_toolbar(ctx);
        self.build_hierarchy(ctx);
        self.build_inspector(ctx);
        self.build_console(ctx);
        self.build_viewport(ctx);
    }

    fn build_toolbar(&mut self, ctx: &egui::Context) {
        if !self.show_toolbar {
            return;
        }
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Supernova Editor v0.1.0");
                ui.separator();

                let play_icon = if self.paused { "\u{25b6} Play" } else { "\u{23f8} Pause" };
                if ui.button(play_icon).clicked() {
                    self.paused = !self.paused;
                    if let Some(tx) = &self.engine_cmd_tx {
                        let _ = tx.send(EngineCommand::SetPaused(self.paused));
                    }
                }
                if ui.button("\u{23f9} Stop").clicked() {
                    if let Some(tx) = &self.engine_cmd_tx {
                        let _ = tx.send(EngineCommand::Shutdown);
                    }
                }
                if ui.button("\u{27a1} Step").clicked() {
                    if let Some(tx) = &self.engine_cmd_tx {
                        let _ = tx.send(EngineCommand::StepFrame);
                    }
                }

                ui.separator();

                if ui.button("+ Entity").clicked() {
                    if let Some(tx) = &self.engine_cmd_tx {
                        let name = format!("Entity_{}", self.entities.len());
                        let _ = tx.send(EngineCommand::SpawnEntity { name });
                    }
                }

                ui.separator();
                ui.label(format!(
                    "Frame: {} | {:.1} ms | Entities: {}",
                    self.engine_frame,
                    self.engine_dt_ms,
                    self.entities.len()
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("\u{2630}").clicked() {
                        self.show_toolbar = !self.show_toolbar;
                    }
                });
            });
        });
    }

    fn build_hierarchy(&mut self, ctx: &egui::Context) {
        if !self.show_hierarchy {
            return;
        }
        egui::SidePanel::left("hierarchy")
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.separator();

                if ui.button("+ Create Entity").clicked() {
                    if let Some(tx) = &self.engine_cmd_tx {
                        let name = format!("Entity_{}", self.entities.len());
                        let _ = tx.send(EngineCommand::SpawnEntity { name });
                    }
                }

                ui.separator();

                for entity in &self.entities {
                    let label = if entity.name.is_empty() {
                        format!("Entity {}", entity.index)
                    } else {
                        entity.name.clone()
                    };

                    let selected = self.selected_entity == Some(entity.index);
                    let response = ui.selectable_label(selected, &label);

                    if response.clicked() {
                        self.selected_entity = Some(entity.index);
                    }

                    response.context_menu(|ui| {
                        if ui.button("Delete").clicked() {
                            if let Some(tx) = &self.engine_cmd_tx {
                                let _ = tx.send(EngineCommand::DespawnEntity {
                                    entity_index: entity.index,
                                });
                            }
                            ui.close_menu();
                        }
                    });
                }
            });
    }

    fn build_inspector(&mut self, ctx: &egui::Context) {
        if !self.show_inspector {
            return;
        }
        egui::SidePanel::right("inspector")
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();

                match self.selected_entity {
                    Some(idx) => {
                        ui.label(format!("Entity {}", idx));
                        ui.separator();

                        ui.label("Transform");
                        let mut translation = [0.0f32; 3];
                        let mut scale = [1.0f32; 3];
                        let mut rotation = [0.0f32; 3];

                        // Find the entity in our snapshot list for display
                        if let Some(e) = self.entities.iter().find(|e| e.index == idx) {
                            ui.label(format!("Has Transform: {}", e.has_transform));
                        }

                        ui.horizontal(|ui| {
                            ui.label("Pos:");
                            ui.add(egui::DragValue::new(&mut translation[0]).prefix("X "));
                            ui.add(egui::DragValue::new(&mut translation[1]).prefix("Y "));
                            ui.add(egui::DragValue::new(&mut translation[2]).prefix("Z "));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Scl:");
                            ui.add(egui::DragValue::new(&mut scale[0]).prefix("X "));
                            ui.add(egui::DragValue::new(&mut scale[1]).prefix("Y "));
                            ui.add(egui::DragValue::new(&mut scale[2]).prefix("Z "));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Rot:");
                            ui.add(egui::DragValue::new(&mut rotation[0]).prefix("P "));
                            ui.add(egui::DragValue::new(&mut rotation[1]).prefix("Y "));
                            ui.add(egui::DragValue::new(&mut rotation[2]).prefix("R "));
                        });

                        if ui.button("Apply Transform").clicked() {
                            if let Some(tx) = &self.engine_cmd_tx {
                                let _ = tx.send(EngineCommand::SetTranslation {
                                    entity_index: idx,
                                    x: translation[0],
                                    y: translation[1],
                                    z: translation[2],
                                });
                                let _ = tx.send(EngineCommand::SetScale {
                                    entity_index: idx,
                                    x: scale[0],
                                    y: scale[1],
                                    z: scale[2],
                                });
                                let _ = tx.send(EngineCommand::SetRotation {
                                    entity_index: idx,
                                    pitch: rotation[0],
                                    yaw: rotation[1],
                                    roll: rotation[2],
                                });
                            }
                        }
                    }
                    None => {
                        ui.label("No entity selected");
                        ui.label("Click an entity in the Hierarchy to inspect it.");
                    }
                }
            });
    }

    fn build_console(&mut self, ctx: &egui::Context) {
        if !self.show_console {
            return;
        }
        egui::TopBottomPanel::bottom("console")
            .default_height(150.0)
            .show(ctx, |ui| {
                ui.heading("Console");
                ui.separator();

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for (level, msg) in &self.console_log {
                            let color = match level {
                                LogLevel::Trace => egui::Color32::GRAY,
                                LogLevel::Debug => egui::Color32::LIGHT_BLUE,
                                LogLevel::Info => egui::Color32::WHITE,
                                LogLevel::Warn => egui::Color32::YELLOW,
                                LogLevel::Error => egui::Color32::RED,
                            };
                            ui.label(egui::RichText::new(msg).color(color).monospace());
                        }
                    });

                ui.separator();
                let mut cmd_buf = String::new();
                ui.horizontal(|ui| {
                    ui.label(">");
                    let response = ui.text_edit_singleline(&mut cmd_buf);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !cmd_buf.is_empty() {
                            self.console_log
                                .push((LogLevel::Info, format!("> {}", cmd_buf)));
                            if let Some(tx) = &self.engine_cmd_tx {
                                let _ = tx.send(EngineCommand::ConsoleCommand(cmd_buf.clone()));
                            }
                            cmd_buf.clear();
                        }
                        response.request_focus();
                    }
                });
            });
    }

    fn build_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Viewport");

            let available = ui.available_size();
            let tex_handle = self.egui_state.as_ref().and_then(|s| s.texture.as_ref());

            match tex_handle {
                Some(tex) => {
                    let mut img = egui::Image::new(egui::load::SizedTexture::new(
                        tex.id(),
                        egui::vec2(
                            self.viewport_size.0 as f32,
                            self.viewport_size.1 as f32,
                        ),
                    ));
                    img = img.max_size(egui::vec2(available.x, available.y));
                    let _ = ui.add(img);
                }
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.label("No frame data received from engine yet...");
                    });
                }
            }
        });
    }
}

// ────────────────────────────────────────────────────────────────────────────
// winit ApplicationHandler
// ────────────────────────────────────────────────────────────────────────────

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Supernova Editor v0.1.0")
            .with_inner_size(winit::dpi::LogicalSize::new(1400, 900));

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create editor window"),
        );

        let gpu = pollster::block_on(GpuContext::new(window.clone()));
        let egui_state = EguiState::new(&gpu);

        // Create IPC channels and spawn the engine.
        let (cmd_tx, _cmd_rx) = bounded::<EngineCommand>(64);
        let (_evt_tx, evt_rx) = bounded::<EngineEvent>(64);

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.egui_state = Some(egui_state);
        self.engine_cmd_tx = Some(cmd_tx);
        self.engine_evt_rx = Some(evt_rx);

        self.spawn_engine_thread();

        self.console_log
            .push((LogLevel::Info, "Editor initialized".into()));
        self.console_log.push((
            LogLevel::Info,
            "Engine thread started — waiting for connection...".into(),
        ));

        self.last_frame = std::time::Instant::now();

        log::info!("Editor window created");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Editor closing");
                if let Some(tx) = &self.engine_cmd_tx {
                    let _ = tx.send(EngineCommand::Shutdown);
                }
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.tick();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            other => {
                // Forward input events to egui.
                if let (Some(window), Some(egui_state)) =
                    (self.window.as_ref(), self.egui_state.as_mut())
                {
                    // egui 0.29 + winit 0.30 integration handled via
                    // egui_state.ctx.send_event or via PlatformInput.
                    // For now we use the simplified approach.
                    let _ = (window, egui_state, &other);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl EditorApp {
    fn tick(&mut self) {
        let now = std::time::Instant::now();
        let _dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        // 1. Drain engine events (lock-free).
        if let Some(rx) = &self.engine_evt_rx {
            while let Ok(evt) = rx.try_recv() {
                match evt {
                    EngineEvent::Initialized { gpu_name, backend } => {
                        self.engine_connected = true;
                        self.console_log.push((
                            LogLevel::Info,
                            format!("Engine connected: {} ({})", gpu_name, backend),
                        ));
                    }
                    EngineEvent::Frame {
                        rgba,
                        width,
                        height,
                        frame_number,
                        dt_ms,
                    } => {
                        self.engine_frame = frame_number;
                        self.engine_dt_ms = dt_ms;
                        self.viewport_rgba = rgba;
                        self.viewport_size = (width, height);

                        if let Some(egui_state) = &mut self.egui_state {
                            egui_state.update_viewport_texture(&self.viewport_rgba, width, height);
                        }
                    }
                    EngineEvent::EntityList { entities } => {
                        self.entities = entities;
                    }
                    EngineEvent::ConsoleLog { level, message } => {
                        self.console_log.push((level, message));
                        // Keep last 500 lines.
                        if self.console_log.len() > 500 {
                            let drain = self.console_log.len() - 500;
                            self.console_log.drain(..drain);
                        }
                    }
                    EngineEvent::Shutdown => {
                        self.engine_connected = false;
                        self.console_log
                            .push((LogLevel::Warn, "Engine disconnected".into()));
                    }
                    EngineEvent::PhysicsStepDone {
                        bodies_active,
                        collisions,
                    } => {
                        // Could display in status bar.
                        let _ = (bodies_active, collisions);
                    }
                }
            }
        }

        // 2. Render egui UI.
        //    Take egui_state out temporarily to avoid double mutable borrow with self.build_ui().
        let mut egui_state = match self.egui_state.take() {
            Some(s) => s,
            None => return,
        };

        let full_output = egui_state.ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(
                        self.viewport_size.0 as f32,
                        self.viewport_size.1 as f32,
                    ),
                )),
                ..Default::default()
            },
            |ctx| {
                self.build_ui(ctx);
            },
        );

        let paint_jobs = egui_state.ctx.tessellate(full_output.shapes, 1.0);

        let gpu = match self.gpu.as_ref() {
            Some(g) => g,
            None => {
                self.egui_state = Some(egui_state);
                return;
            }
        };

        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                gpu.surface_config.width,
                gpu.surface_config.height,
            ],
            pixels_per_point: self
                .window
                .as_ref()
                .map(|w| w.scale_factor() as f32)
                .unwrap_or(1.0),
        };

        // Upload textures.
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_state
                .renderer
                .update_texture(&gpu.device, &gpu.queue, *id, image_delta);
        }

        // Render.
        let surface_tex = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => {
                self.egui_state = Some(egui_state);
                return;
            }
        };

        let finished = {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Editor Encoder"),
                });

            egui_state
                .renderer
                .update_buffers(&gpu.device, &gpu.queue, &mut encoder, &paint_jobs, &screen_desc);

            let view = surface_tex
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Editor Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.12,
                                g: 0.12,
                                b: 0.15,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();

            egui_state
                .renderer
                .render(&mut render_pass, &paint_jobs, &screen_desc);

            drop(render_pass);
            drop(view);

            encoder.finish()
        };

        // Free textures.
        for id in &full_output.textures_delta.free {
            egui_state.renderer.free_texture(id);
        }

        gpu.queue.submit(std::iter::once(finished));
        surface_tex.present();

        // Put egui_state back.
        self.egui_state = Some(egui_state);
    }
}
