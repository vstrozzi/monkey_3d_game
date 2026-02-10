//! Declaration of the utils modules for monkey_3d_game.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::*,
};

/// Command handler for receiving commands from the Controller
pub mod command_handler;

/// State emitter for sending game state to the Controller
pub mod state_emitter;

/// Web adapter for WASM integration
pub mod web_adapter;

/// Various utility functions, constants, and objects
pub mod utils {
    pub mod camera;
    pub mod debug_functions;
    pub mod game_functions;
    pub mod macros;
    pub mod objects;
    pub mod pyramid;
    pub mod setup;
    pub mod systems_logic;
}