//! Systems logic based on the gamephase.
//!
//! Twin-Engine Architecture: The game no longer handles inputs directly.
//! All inputs are processed by the Controller which sends GameCommands.

use crate::command_handler::SharedMemResource;
use crate::command_handler::{PendingAnimation, PendingBlankScreen, PendingReset, RenderingPaused};
use crate::state_emitter::FrameCounterResource;
use crate::utils::camera::{apply_pending_rotation, apply_pending_zoom};
use crate::utils::game_functions::{
    apply_pending_check_alignment, handle_door_animation, spawn_score_bar,
    update_score_bar_animation, update_ui_scale,
};
use crate::utils::objects::{
    DoorWinEntities, GameEntity, PersistentCamera, RandomGen, RoundStartTimestamp,
    UIEntity,
};
use crate::utils::setup::setup_environment;
use bevy::prelude::*;
use crate::utils::setup::setup_round;
use core::sync::atomic::Ordering;
use shared::constants::camera_3d_constants::{
    CAMERA_3D_INITIAL_X, CAMERA_3D_INITIAL_Y, CAMERA_3D_INITIAL_Z,
};

// Plugin for managing all the game systems.config
pub struct SystemsLogicPlugin;

impl Plugin for SystemsLogicPlugin {
    /// Builds the plugin by adding the systems to the app.
    fn build(&self, app: &mut App) {
        app.init_resource::<BlankScreenState>()
            // Spawn persistent camera and static environment once at startup
            .add_systems(Startup, (spawn_persistent_camera, setup_environment))
            // Global UI responsiveness system (runs every frame)
            .add_systems(Update, update_ui_scale)
            // Global command-driven systems
            .add_systems(
                Update,
                (handle_reset_command, handle_animation_door_command),
            )
            // Rendering control systems (run any time)
            .add_systems(Update, (apply_blank_screen, handle_rendering_pause))
            // Input and Logic Systems
            .add_systems(
                Update,
                (
                    // Command-driven systems
                    // We removed is_not_animating check for now as checking SHM atomic every frame in run condition is OK but we can just simplify.
                    (
                        apply_pending_rotation,
                        apply_pending_zoom,
                        apply_pending_check_alignment,
                    )
                        .run_if(is_not_paused),
                    // Animation systems
                    (handle_door_animation, update_score_bar_animation).run_if(is_not_paused),
                    // Ensure local score bar exists (if cleared by reset)
                    // Note: In new flow, score bar spawning is handled by check_alignment or reset?
                    // Actually check_alignment spawns it. Reset clears it.
                ),
            );
    }
}

fn is_not_paused(rendering_paused: Res<RenderingPaused>) -> bool {
    !rendering_paused.0
}

/// This camera persists across resets to avoid artifacts.
fn spawn_persistent_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            CAMERA_3D_INITIAL_X,
            CAMERA_3D_INITIAL_Y,
            CAMERA_3D_INITIAL_Z,
        )
        .looking_at(Vec3::ZERO, Vec3::Y),
        PersistentCamera,
    ));
}

/// Resource tracking blank screen state
#[derive(Resource, Default)]
pub struct BlankScreenState {
    pub is_active: bool,
}

/// Marker component for the blank screen overlay entity
#[derive(Component)]
pub struct BlankScreenOverlay;

/// Helper function to spawn a fullscreen black overlay
fn spawn_blank_overlay(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(1000),
        BlankScreenOverlay,
    ));
}



/// Unified reset handler that works from any state.
/// Always transitions to Resetting state first, which then goes to Playing.
/// Unified reset handler.
fn handle_reset_command(
    mut pending_reset: ResMut<PendingReset>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    random_gen: ResMut<RandomGen>,
    time: Res<Time>,
    mut frame_counter: ResMut<FrameCounterResource>,
    camera_query: Query<&mut Transform, With<PersistentCamera>>,
    game_entities: Query<Entity, With<GameEntity>>,
    ambient_light: Option<ResMut<GlobalAmbientLight>>,
    shm_res: Option<Res<SharedMemResource>>,
    ui_entities: Query<Entity, With<UIEntity>>,
    spotlight_query: Query<&mut SpotLight, (Without<crate::utils::objects::HoleLight>, Without<GameEntity>)>,
    round_start: ResMut<RoundStartTimestamp>,
    mut door_win_entities: ResMut<DoorWinEntities>,
) {
    if !pending_reset.0 {
        return;
    }

    pending_reset.0 = false;

    // Reset commands received
    frame_counter.0 = 0;

    // Clear animation state to avoid stale entity references after despawn
    door_win_entities.animating_door = None;
    door_win_entities.animating_light = None;
    door_win_entities.animating_emissive = None;
    door_win_entities.animation_start_time = None;

    // Clear is_animating flag in SHM
    if let Some(ref shm_res) = shm_res {
        shm_res.0.get().game_structure_game.is_animating.store(false, Ordering::Relaxed);
    }

    despawn_all_game_and_ui(commands.reborrow(), game_entities, ui_entities);

    // Reset shared memory game structure to default values for new round
    setup_round(
        commands.reborrow(),
        meshes,
        materials,
        random_gen,
        camera_query,
        spotlight_query,
        ambient_light,
        shm_res,
        round_start,
        time,
    );

    spawn_score_bar(&mut commands);

}


/// System to handle animation door command
fn handle_animation_door_command(
    mut pending_anim: ResMut<PendingAnimation>,
    mut door_win_entities: ResMut<DoorWinEntities>,
    shm_res: Option<Res<SharedMemResource>>,
    time: Res<Time>,
    // Queries to find entities (similar to game_functions)
    frame_query: Query<&crate::utils::objects::BaseFrame>,
    light_query: Query<(Entity, &ChildOf), With<crate::utils::objects::HoleLight>>,
    emissive_query: Query<(Entity, &ChildOf), With<crate::utils::objects::HoleEmissive>>,
) {
    if !pending_anim.0 {
        return;
    }
    pending_anim.0 = false;

    let Some(shm_res) = shm_res else { return };
    let shm = shm_res.0.get();

    if shm.game_structure_game.is_animating.load(Ordering::Relaxed) {
        info!("Animation door command ignored: already animating");
        return;
    }

    // Find entities matching target
    let target = shm
        .game_structure_game
        .target_door
        .load(Ordering::Relaxed) as usize;

    let mut found_light = None;
    let mut found_emissive = None;

    for (light_entity, parent) in light_query.iter() {
        if let Ok(frame) = frame_query.get(parent.parent()) {
            if frame.door_index == target {
                found_light = Some(light_entity);
                break;
            }
        }
    }

    for (emissive_entity, parent) in emissive_query.iter() {
        if let Ok(frame) = frame_query.get(parent.parent()) {
            if frame.door_index == target {
                found_emissive = Some(emissive_entity);
                break;
            }
        }
    }

    if found_light.is_none() && found_emissive.is_none() {
        warn!("Animation door command: no light/emissive entities found for target_door={}", target);
        return;
    }

    // Only start animation if we found at least one entity
    info!("Starting door animation for target_door={}, light={:?}, emissive={:?}", target, found_light, found_emissive);
    door_win_entities.animating_light = found_light;
    door_win_entities.animating_emissive = found_emissive;
    door_win_entities.animation_start_time = Some(time.elapsed());
    shm.game_structure_game
        .is_animating
        .store(true, Ordering::Relaxed);
}

/// System to apply blank screen command - spawns/despawns a black fullscreen overlay
fn apply_blank_screen(
    mut commands: Commands,
    pending_blank: Res<PendingBlankScreen>,
    mut blank_state: ResMut<BlankScreenState>,
    overlay_query: Query<Entity, With<BlankScreenOverlay>>,
) {
    if pending_blank.0 {
        // Toggle blank screen state
        blank_state.is_active = !blank_state.is_active;

        if blank_state.is_active {
            // Spawn black fullscreen overlay
            spawn_blank_overlay(&mut commands);
            info!("Blank screen activated");
        } else {
            // Despawn the overlay
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn();
            }
            info!("Blank screen deactivated");
        }
    }
}

/// System to handle rendering pause - hides/shows the persistent camera
fn handle_rendering_pause(
    rendering_paused: Res<RenderingPaused>,
    mut visibility_query: Query<&mut Visibility, With<PersistentCamera>>,
) {
    // Only act when the resource has changed
    if !rendering_paused.is_changed() {
        return;
    }

    // When paused, we can hide the 3D camera to stop rendering
    for mut visibility in visibility_query.iter_mut() {
        if rendering_paused.0 {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}

/// Despawn all game and UI entities
fn despawn_all_game_and_ui(
    mut commands: Commands,
    game_query: Query<Entity, With<GameEntity>>,
    ui_query: Query<Entity, With<UIEntity>>,
) {
    for entity in &game_query {
        commands.entity(entity).try_despawn();
    }
    for entity in &ui_query {
        commands.entity(entity).try_despawn();
    }
}
