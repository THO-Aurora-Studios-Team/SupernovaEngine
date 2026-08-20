//! Input system for Supernova Engine.
//!
//! Provides keyboard, mouse, and gamepad input handling with
//! cross-platform support.

use std::collections::HashMap;

/// Key enumeration for input handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Shift,
    Control,
    Alt,
    /// Gamepad buttons
    GamepadA,
    GamepadB,
    GamepadX,
    GamepadY,
    GamepadStart,
    GamepadBack,
    GamepadLeftBumper,
    GamepadRightBumper,
    GamepadLeftTrigger,
    GamepadRightTrigger,
}

/// Mouse button enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Input event type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    KeyPressed(Key),
    KeyReleased(Key),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseMoved(f32, f32),
    MouseScrolled(f32),
    GamepadConnected(u32),
    GamepadDisconnected(u32),
}

/// Input state
#[derive(Debug, Clone)]
pub struct InputState {
    pub keys: HashMap<Key, bool>,
    pub mouse_pos: (f32, f32),
    pub mouse_buttons: HashMap<MouseButton, bool>,
    pub scroll_offset: f32,
    pub gamepad_axes: HashMap<u32, Vec<f32>>,
    pub gamepad_buttons: HashMap<u32, HashMap<u32, bool>>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            mouse_pos: (0.0, 0.0),
            mouse_buttons: HashMap::new(),
            scroll_offset: 0.0,
            gamepad_axes: HashMap::new(),
            gamepad_buttons: HashMap::new(),
        }
    }

    pub fn set_key(&mut self, key: Key, pressed: bool) {
        self.keys.insert(key, pressed);
    }

    pub fn set_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        self.mouse_buttons.insert(button, pressed);
    }

    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_pos = (x, y);
    }

    pub fn set_scroll(&mut self, offset: f32) {
        self.scroll_offset = offset;
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        *self.keys.get(&key).unwrap_or(&false)
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        *self.mouse_buttons.get(&button).unwrap_or(&false)
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_pos
    }

    pub fn get_scroll_offset(&self) -> f32 {
        self.scroll_offset
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Input handler for keyboard and mouse input
pub struct InputHandler {
    state: InputState,
    events: Vec<InputEvent>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
            events: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.state.scroll_offset = 0.0;
        self.events.clear();
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.state.is_key_down(key)
    }

    pub fn is_key_just_pressed(&self, key: Key) -> bool {
        // Would track previous frame state
        false
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.state.is_mouse_button_down(button)
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        self.state.mouse_position()
    }

    pub fn scroll_offset(&self) -> f32 {
        self.state.get_scroll_offset()
    }

    pub fn reset_scroll(&mut self) {
        self.state.set_scroll(0.0);
    }

    pub fn set_key(&mut self, key: Key, pressed: bool) {
        let event = if pressed {
            InputEvent::KeyPressed(key)
        } else {
            InputEvent::KeyReleased(key)
        };
        self.events.push(event);
        self.state.set_key(key, pressed);
    }

    pub fn set_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        let event = if pressed {
            InputEvent::MouseButtonPressed(button)
        } else {
            InputEvent::MouseButtonReleased(button)
        };
        self.events.push(event);
        self.state.set_mouse_button(button, pressed);
    }

    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.events.push(InputEvent::MouseMoved(x, y));
        self.state.set_mouse_pos(x, y);
    }

    pub fn get_events(&self) -> &[InputEvent] {
        &self.events
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
