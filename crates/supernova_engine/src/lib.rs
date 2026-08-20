//! Supernova Engine — main engine crate.
//!
//! Aggregates all subsystem crates into a unified engine facade.

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
    dt: f32,
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
            dt: 0.0,
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

    pub fn initialize(&mut self) {
        let time = supernova_core::Time::default();
        self.app.insert_resource(time);
        self.plugin_manager.initialize(&mut self.app.world);
        self.running = true;
        println!("Supernova Engine initialized");
    }

    pub fn update(&mut self, dt: f32) {
        self.dt = dt;
        self.input.update();
        self.plugin_manager.update(&mut self.app.world, dt);
        self.scripting.update(&mut self.app.world, dt);
        self.physics.update(&mut self.app.world, dt);
        self.audio.update();
        self.app.update(dt);
        self.renderer.begin_frame();
        self.renderer.end_frame();
        self.network.update(dt);
    }

    pub fn run(&mut self) {
        self.initialize();
        println!("Supernova Engine started");
        loop {
            let start = std::time::Instant::now();
            self.update(0.016);
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed < 0.016 {
                std::thread::sleep(std::time::Duration::from_secs_f32(0.016 - elapsed));
            }
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.plugin_manager.shutdown(&mut self.app.world);
        println!("Supernova Engine stopped");
    }
}

impl Default for SupernovaEngine {
    fn default() -> Self {
        Self::new()
    }
}
