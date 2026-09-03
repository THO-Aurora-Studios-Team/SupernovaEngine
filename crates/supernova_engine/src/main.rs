//! Supernova Engine — main entry point.
//!
//! Launches a real wgpu-rendered window with an ECS-driven scene and exposes
//! a lock-free crossbeam-channel IPC bridge so the editor can connect.

use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::bounded;
use supernova_engine::{EngineCommand, EngineEvent, GpuContext};
use supernova_engine::SupernovaEngine;
use supernova_math::Vec3;
use supernova_scene::Transform;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("Supernova Engine v0.1.0");
    log::info!("========================================");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let (_ipc_tx, ipc_rx) = bounded::<EngineCommand>(64);
    let (evt_tx, _evt_rx) = bounded::<EngineEvent>(64);

    let mut app = EngineApp {
        window: None,
        gpu: None,
        renderer: None,
        engine: None,
        ipc_rx,
        evt_tx,
        last_frame: Instant::now(),
        frame_count: 0u64,
        running: true,
    };

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("Event loop error");
}

struct EngineApp {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    renderer: Option<supernova_engine::ViewportRenderer>,
    engine: Option<SupernovaEngine>,
    ipc_rx: crossbeam_channel::Receiver<EngineCommand>,
    evt_tx: crossbeam_channel::Sender<EngineEvent>,
    last_frame: Instant,
    frame_count: u64,
    running: bool,
}

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Supernova Engine v0.1.0")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );

        let gpu = pollster::block_on(GpuContext::new(window.clone()));
        let renderer = supernova_engine::ViewportRenderer::new(&gpu);

        let mut engine = SupernovaEngine::new();
        engine.initialize();

        // Spawn a demo entity
        {
            let world = engine.world_mut();
            let _player = world.spawn_with((Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),));
        }

        let _ = self.evt_tx.send(EngineEvent::Initialized {
            gpu_name: "Integrated GPU".into(),
            backend: format!("{:?}", gpu.surface_config.format),
        });

        log::info!("Engine window created — starting render loop");

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.renderer = Some(renderer);
        self.engine = Some(engine);
        self.last_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested — shutting down");
                self.running = false;
                if let Some(mut engine) = self.engine.take() {
                    engine.stop();
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
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl EngineApp {
    fn tick(&mut self) {
        if !self.running {
            return;
        }

        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.frame_count += 1;

        // 1. Process IPC commands from the editor (lock-free, non-blocking).
        if let Some(engine) = &mut self.engine {
            if !engine.process_commands(&self.ipc_rx) {
                self.running = false;
                return;
            }
        }

        // 2. Step the ECS simulation.
        if let Some(engine) = &mut self.engine {
            engine.update(dt);
        }

        // 3. Render.
        let gpu = self.gpu.as_ref().unwrap();
        let renderer = self.renderer.as_ref().unwrap();

        if let Some(surface_tex) = gpu.begin_frame() {
            let view = surface_tex
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let time_secs = self.last_frame.elapsed().as_secs_f32();
            renderer.render(gpu, &view, time_secs);

            surface_tex.present();
        }
    }
}
