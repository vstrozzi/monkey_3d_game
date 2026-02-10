//! Core game and UI functions.
use bevy::prelude::*;

use crate::command_handler::PendingCheckAlignment;
use crate::command_handler::SharedMemResource;
use crate::utils::objects::{
    BaseDoor, BaseFrame, DoorWinEntities, GameEntity, HoleEmissive, HoleLight, ScoreBarFill,
    ScoreBarUI, UIEntity,
};
use core::sync::atomic::Ordering;
use shared::constants::game_constants::{
    SCORE_BAR_BORDER_THICKNESS, SCORE_BAR_HEIGHT, SCORE_BAR_TOP_OFFSET, SCORE_BAR_WIDTH_PERCENT,
    UI_REFERENCE_HEIGHT,
};

/// Helper to despawn ui entities given a mutable commands reference
pub fn despawn_ui_helper(commands: &mut Commands, query: &Query<Entity, With<UIEntity>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

/// Helper system to cleanup Game entities
pub fn cleanup_game_entities(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// System that applies pending check alignment command from the controller.
/// This is the command-driven version of the alignment check logic.
pub fn apply_pending_check_alignment(
    pending: Res<PendingCheckAlignment>,
    shm_res: Option<Res<SharedMemResource>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    door_query: Query<(Entity, &BaseDoor, &Transform)>,
    _light_query: Query<Entity, With<HoleLight>>,
    _emissive_query: Query<Entity, With<HoleEmissive>>,
    _frame_query: Query<(&BaseFrame, &Children)>,
    mut commands: Commands,
    ui_query: Query<Entity, With<UIEntity>>,
) {
    let Some(shm_res) = shm_res else { return };
    let shm = shm_res.0.get();
    let gs_control = &shm.game_structure_control;
    let gs_game = &shm.game_structure_game;

    // Only proceed check alignment was requested
    if !pending.0 {
        return;
    }

    // Increment attempt counter
    let attempts = gs_game.attempts.load(Ordering::Relaxed) + 1;
    gs_game.attempts.store(attempts, Ordering::Relaxed);

    // Clean old UI and spawn new
    despawn_ui_helper(&mut commands, &ui_query);
    spawn_score_bar(&mut commands);

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Get local camera direction
    let camera_forward = camera_transform.forward();

    // Project camera forward to XZ plane
    let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize();

    let mut best_alignment = -1.0;
    let mut _best_door_index = 0;
    let mut winning_door_alignment = -1.0;

    // Determine target door from SHM
    let target_door_idx = gs_control.target_door.load(Ordering::Relaxed);

    for (_, door, door_transform) in &door_query {
        // Get door normal in world space
        let door_normal_world = door_transform.rotation * door.normal;

        // Project to XZ plane
        let door_normal_xz = Vec3::new(door_normal_world.x, 0.0, door_normal_world.z).normalize();

        // Calculate alignment (dot product)
        let alignment = door_normal_xz.dot(camera_forward_xz);

        // Most positive = door facing toward camera (from outside)
        if alignment > best_alignment {
            best_alignment = alignment;
            _best_door_index = door.door_index;
        }

        // Save the alignment for the target door
        if door.door_index as u32 == target_door_idx {
            winning_door_alignment = alignment;
        }
    }

    // Store alignment for score bar animation AND SHM
    // game_state.cosine_alignment = Some(winning_door_alignment);
    gs_game
        .current_alignment
        .store(winning_door_alignment.to_bits(), Ordering::Relaxed);

    // Clean old UI and spawn new (Score Bar)
    despawn_ui_helper(&mut commands, &ui_query);
    spawn_score_bar(&mut commands);
}

/// Spawns the energy score bar at the top center of the screen
pub fn spawn_score_bar(commands: &mut Commands) {
    // Container for the score bar (centered at top)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Px(SCORE_BAR_TOP_OFFSET),
                justify_content: JustifyContent::Center,
                ..default()
            },
            UIEntity,
        ))
        .with_children(|parent| {
            // Outer border/background of the bar
            parent
                .spawn((
                    Node {
                        width: Val::Percent(SCORE_BAR_WIDTH_PERCENT),
                        height: Val::Px(SCORE_BAR_HEIGHT),
                        border: UiRect::all(Val::Px(SCORE_BAR_BORDER_THICKNESS)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)), // Dark subtle background
                    ScoreBarUI,
                ))
                .with_children(|bar_parent| {
                    // Inner fill bar (starts empty)
                    bar_parent.spawn((
                        Node {
                            width: Val::Percent(0.0), // Starts empty
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.6, 1.0, 0.3)), // Dim cyan glow when empty
                        ScoreBarFill,
                    ));
                });
        });
}

/// Handles the light animation
pub fn handle_door_animation(
    mut door_win_entities: ResMut<DoorWinEntities>,
    shm_res: Option<Res<SharedMemResource>>,
    time: Res<Time>,
    mut light_query: Query<(&mut Visibility, &mut SpotLight), With<HoleLight>>,

    mut emissive_query: Query<
        (&mut Visibility, &MeshMaterial3d<StandardMaterial>),
        (With<HoleEmissive>, Without<HoleLight>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(shm_res) = shm_res else { return };
    let shm = shm_res.0.get();
    let gs_game = &shm.game_structure_game;

    // Animation is started by handle_animation_door_command (sets is_animating + entities)
    let is_animating = gs_game.is_animating.load(Ordering::Relaxed);
    if !is_animating {
        return;
    }

    let Some(start_time) = door_win_entities.animation_start_time else {
        // No start time set — animation state is inconsistent, clear it
        warn!("handle_door_animation: is_animating=true but no start_time, clearing.");
        gs_game.is_animating.store(false, Ordering::Relaxed);
        return;
    };
    let elapsed = (time.elapsed() - start_time).as_secs_f32();

    // Config values from SHM
    let fade_out_end = f32::from_bits(gs_game.door_anim_fade_out.load(Ordering::Relaxed));
    let stay_open_end =
        fade_out_end + f32::from_bits(gs_game.door_anim_stay_open.load(Ordering::Relaxed));
    let fade_in_end =
        stay_open_end + f32::from_bits(gs_game.door_anim_fade_in.load(Ordering::Relaxed));

    // Get light entity from door_win_entities
    let Some(light_entity) = door_win_entities.animating_light else {
        // Entity was despawned (e.g. by reset) — clear animation state
        warn!("handle_door_animation: animating_light is None, clearing animation.");
        door_win_entities.animation_start_time = None;
        gs_game.is_animating.store(false, Ordering::Relaxed);
        return;
    };

    // Get light visibility and component
    let Ok((mut light_visibility, mut spotlight)) = light_query.get_mut(light_entity) else {
        // Entity no longer valid — clear animation state
        warn!("handle_door_animation: light entity not found in query, clearing animation.");
        door_win_entities.animating_light = None;
        door_win_entities.animating_emissive = None;
        door_win_entities.animation_start_time = None;
        gs_game.is_animating.store(false, Ordering::Relaxed);
        return;
    };

    // Calculate animation intensity (0.0 to 1.0)
    let intensity_factor = if elapsed < fade_out_end {
        // Phase 1: Fade Out (Opening) - 0.0 to 1.0
        elapsed / fade_out_end
    } else if elapsed < stay_open_end {
        // Phase 2: Stay Open - 1.0
        1.0
    } else if elapsed < fade_in_end {
        // Phase 3: Fade In (Closing) - 1.0 to 0.0
        1.0 - ((elapsed - stay_open_end) / f32::from_bits(gs_game.door_anim_fade_in.load(Ordering::Relaxed)))
    } else {
        // Animation finished
        0.0
    };

    // Max intensity values 
    let max_spotlight_intensity = f32::from_bits(gs_game.max_spotlight_intensity.load(Ordering::Relaxed));

    if intensity_factor > 0.0 {
        // Animation is in progress — update spotlight
        *light_visibility = Visibility::Visible;
        spotlight.intensity = max_spotlight_intensity * intensity_factor;

        // Also update emissive material
        if let Some(emissive_entity) = door_win_entities.animating_emissive {
            if let Ok((mut emissive_visibility, material_handle)) =
                emissive_query.get_mut(emissive_entity)
            {
                *emissive_visibility = Visibility::Visible;

                if let Some(material) = materials.get_mut(&material_handle.0) {
                    let light_color = spotlight.color.to_linear();
                    material.emissive = LinearRgba::new(
                        light_color.red * max_spotlight_intensity * intensity_factor,
                        light_color.green * max_spotlight_intensity * intensity_factor,
                        light_color.blue * max_spotlight_intensity * intensity_factor,
                        1.0,
                    );
                }
            }
        }
    } else {
        // Animation finished — hide spotlight
        *light_visibility = Visibility::Hidden;
        spotlight.intensity = 0.0;

        // Hide emissive and clear state
        if let Some(emissive_entity) = door_win_entities.animating_emissive {
            if let Ok((mut emissive_visibility, material_handle)) =
                emissive_query.get_mut(emissive_entity)
            {
                *emissive_visibility = Visibility::Hidden;

                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.emissive = LinearRgba::new(0.0, 0.0, 0.0, 0.0);
                }
            }
        }

        // Clear animation state
        door_win_entities.animating_light = None;
        door_win_entities.animating_emissive = None;
        door_win_entities.animation_start_time = None;
        gs_game.is_animating.store(false, Ordering::Relaxed);
    }
}


/// Updates the score bar fill and color during the door animation
pub fn update_score_bar_animation(
    door_win_entities: Res<DoorWinEntities>,
    shm_res: Option<Res<SharedMemResource>>,
    time: Res<Time>,
    mut fill_query: Query<(&mut Node, &mut BackgroundColor), With<ScoreBarFill>>,
) {
    let Ok((mut node, mut bg_color)) = fill_query.single_mut() else {
        return;
    };
    let Some(shm_res) = shm_res else { return };
    let shm = shm_res.0.get();

    // Get alignment score (normalized to 0.0 - 1.0 range from -1.0 - 1.0)
    let alignment_bits = shm
        .game_structure_game
        .current_alignment
        .load(Ordering::Relaxed);
    let alignment = f32::from_bits(alignment_bits);
    let alignment_normalized = ((alignment + 1.0) / 2.0).clamp(0.0, 1.0);

    let is_animating = shm.game_structure_game.is_animating.load(Ordering::Relaxed);

    // Calculate the bar width
    let current_width = if is_animating {
        // During animation: fill progressively based on animation progress
        let Some(start_time) = door_win_entities.animation_start_time else {
            return;
        };
        let elapsed = (time.elapsed() - start_time).as_secs_f32();

        let fade_out_end = f32::from_bits(
            shm.game_structure_game
                .door_anim_fade_out
                .load(Ordering::Relaxed),
        );
        let stay_open_dur = f32::from_bits(
            shm.game_structure_game
                .door_anim_stay_open
                .load(Ordering::Relaxed),
        );
        let fade_in_dur = f32::from_bits(
            shm.game_structure_game
                .door_anim_fade_in
                .load(Ordering::Relaxed),
        );

        let total_duration = fade_out_end + stay_open_dur + fade_in_dur;
        let fill_progress = (elapsed / total_duration).clamp(0.0, 1.0);
        let target_width = alignment_normalized * 100.0;
        fill_progress * target_width
    } else {
        // Not animating: show the current alignment directly
        alignment_normalized * 100.0
    };

    node.width = Val::Percent(current_width);

    // Color gradient based on alignment quality (cyan -> yellow -> white)
    let color = if alignment_normalized < 0.5 {
        let t = alignment_normalized * 2.0; // 0.0 to 1.0 for first half
        Color::srgba(
            0.2 + t * 0.8, // R: 0.2 -> 1.0
            0.6 + t * 0.4, // G: 0.6 -> 1.0
            1.0 - t * 0.2, // B: 1.0 -> 0.8
            0.7 + t * 0.2, // A: 0.7 -> 0.9
        )
    } else {
        let t = (alignment_normalized - 0.5) * 2.0; // 0.0 to 1.0 for second half
        Color::srgba(
            1.0,           // R: stays at 1.0
            1.0,           // G: stays at 1.0
            0.8 + t * 0.2, // B: 0.8 -> 1.0 (yellow to white)
            0.9 + t * 0.1, // A: 0.9 -> 1.0
        )
    };

    *bg_color = BackgroundColor(color);
}

/// Updates UI scale based on window size for responsive design
/// Targets 1080p (1920x1080) as the reference resolution
pub fn update_ui_scale(mut ui_scale: ResMut<UiScale>, window_query: Query<&Window>) {
    let Ok(window) = window_query.single() else {
        return;
    };

    // Calculate scale based on window height (reference: 1080p)
    let scale = window.height() / UI_REFERENCE_HEIGHT;

    // Clamp scale to reasonable bounds (0.5x to 2.0x)
    let clamped_scale = scale.clamp(0.5, 2.0);

    ui_scale.0 = clamped_scale;
}
