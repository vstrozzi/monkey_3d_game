// This file handles loading and saving game settings from/to a TOML file.
use crate::utils::objects::GameSettings;
use bevy::prelude::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The path to the settings file relative to the executable.
const SETTINGS_FILE_PATH: &str = "settings.toml";

/// Serializable version of GameSettings for TOML conversion.
/// Bevy's Color type is not directly serializable, so we convert to RGB arrays.
#[derive(Serialize, Deserialize, Debug)]
struct SerializableSettings {
    pyramid: PyramidSettings,
    decorations: DecorationSettings,
    camera: CameraSettings,
    game: GameSettingsData,
}

#[derive(Serialize, Deserialize, Debug)]
struct PyramidSettings {
    base_radius_min: f32,
    base_radius_max: f32,
    height_min: f32,
    height_max: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct DecorationSettings {
    count_min: usize,
    count_max: usize,
    size_min: f32,
    size_max: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CameraSettings {
    speed_rotation: f32,
    speed_zoom: f32,
    min_radius: f32,
    max_radius: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct GameSettingsData {
    alignment_threshold: f32,
    pyramid_colors: [[f32; 3]; 3], // RGB arrays for each color
    random_seed: u64,
}

impl From<&GameSettings> for SerializableSettings {
    fn from(settings: &GameSettings) -> Self {
        SerializableSettings {
            pyramid: PyramidSettings {
                base_radius_min: settings.pyramid_base_radius_min,
                base_radius_max: settings.pyramid_base_radius_max,
                height_min: settings.pyramid_height_min,
                height_max: settings.pyramid_height_max,
            },
            decorations: DecorationSettings {
                count_min: settings.decoration_count_min,
                count_max: settings.decoration_count_max,
                size_min: settings.decoration_size_min,
                size_max: settings.decoration_size_max,
            },
            camera: CameraSettings {
                speed_rotation: settings.camera_speed_rotation,
                speed_zoom: settings.camera_speed_zoom,
                min_radius: settings.camera_min_radius,
                max_radius: settings.camera_max_radius,
            },
            game: GameSettingsData {
                alignment_threshold: settings.alignment_threshold,
                pyramid_colors: [
                    color_to_rgb(&settings.pyramid_colors[0]),
                    color_to_rgb(&settings.pyramid_colors[1]),
                    color_to_rgb(&settings.pyramid_colors[2]),
                ],
                random_seed: settings.random_seed,
            },
        }
    }
}

impl From<SerializableSettings> for GameSettings {
    fn from(s: SerializableSettings) -> Self {
        GameSettings {
            pyramid_base_radius_min: s.pyramid.base_radius_min,
            pyramid_base_radius_max: s.pyramid.base_radius_max,
            pyramid_height_min: s.pyramid.height_min,
            pyramid_height_max: s.pyramid.height_max,

            decoration_count_min: s.decorations.count_min,
            decoration_count_max: s.decorations.count_max,
            decoration_size_min: s.decorations.size_min,
            decoration_size_max: s.decorations.size_max,

            camera_speed_rotation: s.camera.speed_rotation,
            camera_speed_zoom: s.camera.speed_zoom,
            camera_min_radius: s.camera.min_radius,
            camera_max_radius: s.camera.max_radius,

            alignment_threshold: s.game.alignment_threshold,
            pyramid_colors: [
                rgb_to_color(s.game.pyramid_colors[0]),
                rgb_to_color(s.game.pyramid_colors[1]),
                rgb_to_color(s.game.pyramid_colors[2]),
            ],
            random_seed: s.game.random_seed,
        }
    }
}

/// Converts a Bevy Color to an RGB array [r, g, b].
fn color_to_rgb(color: &Color) -> [f32; 3] {
    let linear = color.to_linear();
    [linear.red, linear.green, linear.blue]
}

/// Converts an RGB array [r, g, b] to a Bevy Color.
fn rgb_to_color(rgb: [f32; 3]) -> Color {
    Color::srgb(rgb[0], rgb[1], rgb[2])
}

/// Loads game settings from the TOML file.
/// If the file doesn't exist or can't be parsed, returns the default settings.
pub fn load_settings() -> GameSettings {
    let path = Path::new(SETTINGS_FILE_PATH);

    if !path.exists() {
        println!(
            "Settings file not found at '{}'. Using defaults.",
            SETTINGS_FILE_PATH
        );
        return GameSettings::default();
    }

    match fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<SerializableSettings>(&contents) {
            Ok(serializable) => {
                println!(
                    "Settings loaded successfully from '{}'.",
                    SETTINGS_FILE_PATH
                );
                GameSettings::from(serializable)
            }
            Err(e) => {
                eprintln!("Failed to parse settings file: {}. Using defaults.", e);
                GameSettings::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read settings file: {}. Using defaults.", e);
            GameSettings::default()
        }
    }
}

/// Saves game settings to the TOML file.
/// Returns Ok(()) on success, or an error message on failure.
pub fn save_settings(settings: &GameSettings) -> Result<(), String> {
    let serializable = SerializableSettings::from(settings);

    let toml_string = toml::to_string_pretty(&serializable)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(SETTINGS_FILE_PATH, toml_string)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    println!("Settings saved successfully to '{}'.", SETTINGS_FILE_PATH);
    Ok(())
}

/// Creates a default settings file if it doesn't exist.
pub fn create_default_settings_file() -> Result<(), String> {
    let path = Path::new(SETTINGS_FILE_PATH);

    if path.exists() {
        return Ok(()); // File already exists, nothing to do
    }

    let default_settings = GameSettings::default();
    save_settings(&default_settings)
}
