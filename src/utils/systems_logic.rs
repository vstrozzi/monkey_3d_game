//! Systems logic based on the gamephase.

use crate::utils::camera::camera_3d_fpov_inputs;
use crate::utils::game_functions::{
    check_loading_complete, despawn_ui, handle_door_animation, menu_inputs, playing_inputs,
    setup_intro_ui, setup_loading_ui, setup_playing_ui, setup_won_ui, update_score_bar_animation,
    update_ui_scale, won_inputs,
};
use crate::utils::objects::{GamePhase, GameState, LoadingState};
use crate::utils::setup::{despawn_setup, setup};
use bevy::prelude::*;

// Plugin for managing all the game systems based on the current game phase.
pub struct SystemsLogicPlugin;

impl Plugin for SystemsLogicPlugin {
    /// Builds the plugin by adding the systems to the app.
    fn build(&self, app: &mut App) {
        app.init_state::<GamePhase>()
            .init_resource::<LoadingState>()
            // Global UI responsiveness system (runs every frame)
            .add_systems(Update, update_ui_scale)
            // Intro State (Menu)
            .add_systems(OnEnter(GamePhase::MenuUI), setup_intro_ui)
            .add_systems(Update, menu_inputs.run_if(in_state(GamePhase::MenuUI)))
            .add_systems(OnExit(GamePhase::MenuUI), despawn_ui)
            // Loading State (black screen while scene loads)
            .add_systems(
                OnEnter(GamePhase::Loading),
                (setup, setup_loading_ui).chain(),
            )
            .add_systems(
                Update,
                check_loading_complete.run_if(in_state(GamePhase::Loading)),
            )
            .add_systems(OnExit(GamePhase::Loading), despawn_ui)
            // Playing State
            .add_systems(OnEnter(GamePhase::Playing), setup_playing_ui)
            .add_systems(
                Update,
                (
                    // Allow inputs only if not animating
                    (playing_inputs, camera_3d_fpov_inputs)
                        .chain()
                        .run_if(in_state(GamePhase::Playing).and(is_animating)), // STILL HERE FOR PERFORMANCE REASON BUT LOGIC INO INDIVID FUNCTIONS
                    // All the other systems can keep playing while we animate
                    (handle_door_animation, update_score_bar_animation)
                        .run_if(in_state(GamePhase::Playing)),
                ),
            )
            .add_systems(
                OnExit(GamePhase::Playing),
                (despawn_ui, despawn_setup).chain(),
            )
            // Won State
            .add_systems(OnEnter(GamePhase::Won), setup_won_ui)
            .add_systems(Update, won_inputs.run_if(in_state(GamePhase::Won)))
            .add_systems(OnExit(GamePhase::Won), despawn_ui);
    }
}

// Bevy needs a function for systems
fn is_animating(game_state: Res<GameState>) -> bool {
    !game_state.is_animating
}
