// This file defines the various objects, resources, and components used in the game.
use bevy::prelude::*;
use rand_chacha::rand_core::SeedableRng;
use std::time::Duration;

use crate::utils::constants::game_constants::SEED;
use rand_chacha::ChaCha8Rng;

/// An enum representing the different types of pyramids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyramidType {
    Type1,
    Type2,
}

impl Default for PyramidType {
    fn default() -> Self {
        PyramidType::Type1
    }
}

/// An enum representing the possible shapes for decorations on the pyramid faces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationShape {
    Circle,
    Square,
    Star,
    Triangle,
}

/// A single decoration on a pyramid face.
/// Stored using barycentric coordinates relative to the triangle vertices (top, corner1, corner2).
#[derive(Clone, Debug)]
pub struct Decoration {
    /// Position can be reconstructed as: position = w0*top + w1*corner1 + w2*corner2
    pub barycentric: Vec3,
    /// Size of the decoration
    pub size: f32,
}

/// A set of decorations for a pyramid face.
/// All decorations on a face share the same shape and color.
#[derive(Clone, Debug)]
pub struct DecorationSet {
    /// The shape used for all decorations on this face
    pub shape: DecorationShape,
    /// The color used for all decorations on this face
    pub color: Color,
    /// The list of individual decorations with their positions and sizes
    pub decorations: Vec<Decoration>,
}

/// Game state enum representing the different states the game can be in
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GamePhase {
    #[default]
    Menu,
    Instructions,
    Settings,
    Playing,
    Won,
}

/// A resource that holds game settings that can be configured by the player.
#[derive(Resource, Clone, Debug)]
pub struct GameSettings {
    // Pyramid settings
    pub pyramid_base_radius_min: f32,
    pub pyramid_base_radius_max: f32,
    pub pyramid_height_min: f32,
    pub pyramid_height_max: f32,

    // Decoration settings
    pub decoration_count_min: usize,
    pub decoration_count_max: usize,
    pub decoration_size_min: f32,
    pub decoration_size_max: f32,

    // Camera settings
    pub camera_speed_rotation: f32,
    pub camera_speed_zoom: f32,
    pub camera_min_radius: f32,
    pub camera_max_radius: f32,

    // Game settings
    pub alignment_threshold: f32,
    pub pyramid_colors: [Color; 3],
    pub random_seed: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        use crate::utils::constants::{
            camera_3d_constants::*, game_constants::*, pyramid_constants::*,
        };

        Self {
            pyramid_base_radius_min: PYRAMID_BASE_RADIUS_MIN,
            pyramid_base_radius_max: PYRAMID_BASE_RADIUS_MAX,
            pyramid_height_min: PYRAMID_HEIGHT_MIN,
            pyramid_height_max: PYRAMID_HEIGHT_MAX,

            decoration_count_min: DECORATION_COUNT_MIN,
            decoration_count_max: DECORATION_COUNT_MAX,
            decoration_size_min: DECORATION_SIZE_MIN,
            decoration_size_max: DECORATION_SIZE_MAX,

            camera_speed_rotation: CAMERA_3D_SPEED_X,
            camera_speed_zoom: CAMERA_3D_SPEED_Z,
            camera_min_radius: CAMERA_3D_MIN_RADIUS,
            camera_max_radius: CAMERA_3D_MAX_RADIUS,

            alignment_threshold: COSINE_ALIGNMENT_CAMERA_FACE_THRESHOLD,
            pyramid_colors: PYRAMID_COLORS,
            random_seed: SEED,
        }
    }
}

/// A resource that holds the current state of the game.
#[derive(Resource, Clone, Default, Debug)]
pub struct GameState {
    /// The current phase of the game
    pub phase: GamePhase,
    // Game values
    // The seed used for random number generation.
    pub random_seed: u64,
    // The type of pyramid in the current game.
    pub pyramid_type: PyramidType,
    // The base radius of the pyramid.
    pub pyramid_base_radius: f32,
    // The height of the pyramid.
    pub pyramid_height: f32,
    // The index of the target face of the pyramid.
    pub pyramid_target_face_index: usize,
    // The starting orientation radius of the pyramid.
    pub pyramid_start_orientation_radius: f32,
    // The colors of the pyramid faces.
    pub pyramid_color_faces: [Color; 3],

    // State change tracking
    /// A flag indicating whether the game state has changed.
    pub is_changed: bool,

    // Timing
    // The time when the game started.
    pub start_time: Option<Duration>,
    // The time when the game ended.
    pub end_time: Option<Duration>,

    // Metrics
    // The number of attempts the player has made.
    pub attempts: u32,
    // The cosine alignment of the camera with the target face when the player wins.
    pub cosine_alignment: Option<f32>,
}

/// A resource for random number generation.
#[derive(Resource)]
pub struct RandomGen {
    pub random_gen: ChaCha8Rng,
}

impl RandomGen {
    // Creates a new `RandomGen` from a given seed.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            random_gen: ChaCha8Rng::seed_from_u64(seed),
        }
    }
}

impl Default for RandomGen {
    // Creates a new `RandomGen` with the default seed.
    fn default() -> Self {
        use rand_chacha::rand_core::SeedableRng;
        Self {
            random_gen: ChaCha8Rng::seed_from_u64(SEED),
        }
    }
}

/// A component that marks an entity as a pyramid.
#[derive(Component)]
pub struct Pyramid;

/// A component that marks an entity as a face of a pyramid.
#[derive(Component)]
pub struct FaceMarker {
    pub face_index: usize,
    pub color: Color,
    pub normal: Vec3,
    /// The decorations on this face (if any)
    pub decorations: Option<DecorationSet>,
}

/// A component that marks an entity as a game entity, which can be cleared during setup.
#[derive(Component)]
pub struct GameEntity;

/// A component that marks an entity as a UI entity.
#[derive(Component)]
pub struct UIEntity;
