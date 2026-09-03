//! Supernova Editor — main entry point.
//!
//! Launches the editor UI window with wgpu + egui, spawns the engine
//! on a background thread, and connects via lock-free IPC channels.

use supernova_editor::EditorApp;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("Supernova Editor v0.1.0");
    log::info!("===========================");

    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = EditorApp::new();
    event_loop
        .run_app(&mut app)
        .expect("Event loop error");
}
