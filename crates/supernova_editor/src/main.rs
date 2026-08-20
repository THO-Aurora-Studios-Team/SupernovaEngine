//! Supernova Editor — main entry point.
//!
//! The editor provides tools for creating and modifying game scenes
//! and assets. It integrates with the core engine and renderer.

use supernova_editor::EditorApp;

fn main() {
    env_logger::init();
    println!("Supernova Editor v0.1.0");
    println!("===========================");

    let mut app = EditorApp::new();
    app.run();
}
