use crate::math::Vec2;
use std::collections::HashSet;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Input manager that tracks keyboard state, mouse state, and scroll.
#[derive(Default, Debug)]
pub struct Input {
    keys: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    mouse_buttons: HashSet<MouseButton>,
    mouse_just_pressed: HashSet<MouseButton>,
    mouse_just_released: HashSet<MouseButton>,
    pub mouse_pos: Vec2,
    pub mouse_delta: Vec2,
    pub scroll: f32,
    last_mouse_pos: Option<Vec2>,
    pub text_input: String,
}

impl Input {
    /// Returns true while the key is held down.
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }

    /// Returns true on the frame the key was pressed.
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    /// Returns true on the frame the key was released.
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    /// Returns true while the mouse button is held down.
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    /// Returns true on the frame the mouse button was pressed.
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_just_pressed.contains(&button)
    }

    /// Returns true on the frame the mouse button was released.
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_just_released.contains(&button)
    }

    /// Returns the text typed since the last frame clear.
    pub fn text(&self) -> &str {
        &self.text_input
    }

    pub(crate) fn process_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    match key_event.state {
                        ElementState::Pressed => {
                            if !self.keys.contains(&code) {
                                self.keys_just_pressed.insert(code);
                            }
                            self.keys.insert(code);
                        }
                        ElementState::Released => {
                            self.keys.remove(&code);
                            self.keys_just_released.insert(code);
                        }
                    }
                }
                // Handle text input
                if let Some(text) = key_event.text.as_ref() {
                    self.text_input.push_str(text);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    if !self.mouse_buttons.contains(button) {
                        self.mouse_just_pressed.insert(*button);
                    }
                    self.mouse_buttons.insert(*button);
                }
                ElementState::Released => {
                    self.mouse_buttons.remove(button);
                    self.mouse_just_released.insert(*button);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                self.mouse_delta = match self.last_mouse_pos {
                    Some(last) => new_pos - last,
                    None => Vec2::ZERO,
                };
                self.last_mouse_pos = Some(new_pos);
                self.mouse_pos = new_pos;
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => self.scroll += y,
                MouseScrollDelta::PixelDelta(pos) => self.scroll += pos.y as f32 / 100.0,
            },
            _ => {}
        }
    }

    pub(crate) fn clear_just_pressed(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_just_pressed.clear();
        self.mouse_just_released.clear();
        self.scroll = 0.0;
        self.mouse_delta = Vec2::ZERO;
        self.text_input.clear();
    }
}
