//! Core game logic and UI functions.
use bevy::prelude::*;

use crate::utils::constants::game_constants::{
    COSINE_ALIGNMENT_CAMERA_FACE_THRESHOLD, DOOR_ANIMATION_FADE_IN_DURATION,
    DOOR_ANIMATION_FADE_OUT_DURATION, DOOR_ANIMATION_STAY_OPEN_DURATION,
};
use crate::utils::objects::{
    BaseDoor, BaseFrame, FaceMarker, GameEntity, GamePhase, GameState, HoleLight, Pyramid,
    UIEntity,
};

use crate::utils::objects::RandomGen;
use crate::utils::setup::setup;

/// A plugin for handling game functions, including checking for face alignment and managing the game UI.
pub struct GameFunctionsPlugin;

impl Plugin for GameFunctionsPlugin {
    /// Builds the plugin by adding the `check_face_alignment` and `game_ui` systems to the app.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                crate::utils::game_functions::check_face_alignment,
                crate::utils::game_functions::handle_door_animation,
                crate::utils::game_functions::game_ui,
            )
                .chain(),),
        );
    }
}

/// Spawns a black screen that covers the entire viewport.
pub fn spawn_black_screen(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        UIEntity, // Marker for despawning
    ));
}

/// Spawns centered text on a black screen.
pub fn spawn_centered_text_black_screen(commands: &mut Commands, text: &str) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center, // horizontally center children
                align_items: AlignItems::Center,         // vertically center children
                ..default()
            },
            UIEntity,                                    // Marker for despawning
            BackgroundColor(Color::srgb(0.0, 0.0, 0.0)), // transparent container
        ))
        .with_children(|parent| {
            // Spawn the text child
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    max_width: Val::Px(1200.0), // limit text width for wrapping
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));
        });
}

/// Checks if the player has won the game by aligning the camera with the correct face of the pyramid.
pub fn check_face_alignment(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut materials: ResMut<Assets<StandardMaterial>>, // Added access to materials
    camera_query: Query<&Transform, With<Camera3d>>,
    face_query: Query<(&Transform, &FaceMarker), With<Pyramid>>,
    mut door_query: Query<(Entity, &BaseDoor, &mut MeshMaterial3d<StandardMaterial>)>, // Made mutable for material replacement
    light_query: Query<Entity, With<HoleLight>>,
    frame_query: Query<(&BaseFrame, &Children)>,
) {
    // Only check if the game is in Playing state and NOT animating
    if game_state.phase != GamePhase::Playing || game_state.is_animating {
        return;
    }
    // Check for SPACE key press to check alignment
    if keyboard.just_pressed(KeyCode::Space) {
        game_state.nr_attempts += 1;

        let Ok(camera_transform) = camera_query.single() else {
            return;
        };
        // ... (rest of function)
        // Get camera direction
        let camera_forward = camera_transform.local_z();

        // Check which face is most aligned with camera by getting the one with
        // the smallest dot product between camera dir and face dir towards origin
        // (i.e. face is facing camera)
        let mut best_alignment = 1.0;
        let mut best_face_index = None;
        let mut best_face_alignment_val = None; // For winning condition check

        for (face_transform, face_marker) in &face_query {
            // Get face normal in world space
            // The local normal is stored in `face_marker.normal`
            let face_normal = (face_transform.rotation * (face_marker.normal)).normalize();

            // Project down to XZ plane
            let face_normal_xz = Vec3::new(face_normal.x, 0.0, face_normal.z).normalize();
            // Calculate alignment (dot product) of camera direction and face normal
            let alignment = face_normal_xz.dot(*camera_forward);

            if alignment < best_alignment {
                best_alignment = alignment;
                best_face_index = Some(face_marker.face_index);
                best_face_alignment_val = Some(alignment);
            }
        }

        if let Some(best_face_index) = best_face_index {
            // Determine if the player wins
            let has_won = if let Some(alignment) = best_face_alignment_val {
                 alignment < COSINE_ALIGNMENT_CAMERA_FACE_THRESHOLD && best_face_index == game_state.pyramid_target_face_index
            } else {
                false
            };
            
            // Set pending phase based on win condition
            if has_won {
                 game_state.pending_phase = Some(GamePhase::Won);
                 game_state.end_time = Some(time.elapsed());
                 game_state.cosine_alignment = best_face_alignment_val;
            } else {
                game_state.pending_phase = Some(GamePhase::Playing); // Continue playing if lost
            }

            // --- ANIMATION START LOGIC ---
            // Find the door to animate.
            // We want the door on the 'best_face_index', specifically the one most aligned with the camera.
            // A face has 3 doors (indices from `best_face_index * 3` to `best_face_index * 3 + 2`).
            // We can check which of these 3 doors has a normal most opposed to the camera forward vector.
            // However, simply picking the middle one often works well for "center of face".
            // The user says "open up the hole closer to the winning face/angle".
            
            let mut best_door_entity = None;

            // Find the door which matches the best_face_index and is the center door
             for (entity, door, _) in &door_query {
                if door.face_index == best_face_index  && door.is_center_door {
                     best_door_entity = Some(entity);
                     break; 
                }
            }


            if let Some(door_entity) = best_door_entity {
                // Determine which door entity we are modifying
                
                // IMPORTANT: Ensure the door has a unique material so we only fade THIS door.
                if let Ok((_, _, mut mat_handle)) = door_query.get_mut(door_entity) {
                    if let Some(material) = materials.get(&mat_handle.0) {
                        let mut new_material = material.clone();
                        // Reset alpha just in case
                        new_material.base_color.set_alpha(1.0); 
                        // Add as new asset
                        mat_handle.0 = materials.add(new_material);
                    }
                }

                 // Find the corresponding light.
                 // The light is a child of the `BaseFrame`. We need to find the `BaseFrame` with the same `side_index` as the door.
                 
                 // Reuse door info to find the frame
                 let door_side_index = door_query.get(door_entity).unwrap().1.side_index;

                 let mut found_light = None;

                 // Iterate frames to find the one with correct side index
                 for (frame, children) in &frame_query {
                     if frame.side_index == door_side_index {
                        // Check children for HoleLight
                        for child in children {
                            if light_query.get(*child).is_ok() {
                                found_light = Some(*child);
                                break;
                            }
                        }
                     }
                     if found_light.is_some() { break; }
                 }

                 if let Some(light_entity) = found_light {
                     // Start Animation
                     game_state.is_animating = true;
                     game_state.animating_door = Some(door_entity);
                     game_state.animating_light = Some(light_entity);
                     game_state.animation_start_time = Some(time.elapsed());
                 }
            }
        }
    }
}

/// Handles the door animation state machine
pub fn handle_door_animation(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut light_query: Query<&mut Visibility, With<HoleLight>>,
    door_query: Query<&MeshMaterial3d<StandardMaterial>, With<BaseDoor>>,
) {
    if !game_state.is_animating {
        return;
    }

    let Some(start_time) = game_state.animation_start_time else { return; };
    let elapsed = (time.elapsed() - start_time).as_secs_f32();

    let door_entity = game_state.animating_door.unwrap();
    let light_entity = game_state.animating_light.unwrap();

    let fade_out_end = DOOR_ANIMATION_FADE_OUT_DURATION;
    let stay_open_end = fade_out_end + DOOR_ANIMATION_STAY_OPEN_DURATION;
    let fade_in_end = stay_open_end + DOOR_ANIMATION_FADE_IN_DURATION;

    // Get material handle
    let Ok(material_handle) = door_query.get(door_entity) else { return; };
    let Some(material) = materials.get_mut(material_handle) else { return; };

    // Get light visibility
    let Ok(mut light_visibility) = light_query.get_mut(light_entity) else { return; };


    if elapsed < fade_out_end {
        // Phase 1: Fade Out (Opening)
        // Light should be visible immediately
        *light_visibility = Visibility::Visible;
        
        let t = elapsed / DOOR_ANIMATION_FADE_OUT_DURATION;
        // Alpha goes from 1.0 to 0.0
        let alpha = 1.0 - t.clamp(0.0, 1.0);
        material.base_color.set_alpha(alpha);

    } else if elapsed < stay_open_end {
        // Phase 2: Stay Open
        *light_visibility = Visibility::Visible;
        material.base_color.set_alpha(0.0);

    } else if elapsed < fade_in_end {
        // Phase 3: Fade In (Closing)
        // Light should still be visible? "When closing remember to not render the light anymore". 
        // Maybe turn it off at the start of closing? Or at end?
        // Prompt: "stay open for 0.5 sec and then close. (i.e. becpomming back fully visible) When closing remember to not render the light anymore."
        // Interpreting "When closing..." as "When the closing process is happening or finishes?"
        // Usually "When closing" implies the transition. If the door is becoming visible, the light inside might be occluded physically, 
        // but if we want to save performance or logic, maybe turn it off?
        // Let's assume we turn it off *after* it's closed, or linear fade?
        // But the prompt says "When closing...".
        // Let's keep it visible during the fade in (so we see the door closing over the light) and turn off at the very end.
        *light_visibility = Visibility::Visible;
        
        let t = (elapsed - stay_open_end) / DOOR_ANIMATION_FADE_IN_DURATION;
        // Alpha goes from 0.0 to 1.0
        let alpha = t.clamp(0.0, 1.0);
        material.base_color.set_alpha(alpha);

    } else {
        // Animation Finished
        material.base_color.set_alpha(1.0);
        *light_visibility = Visibility::Hidden; // Turn off light

        game_state.is_animating = false;
        game_state.animating_door = None;
        game_state.animating_light = None;
        game_state.animation_start_time = None;

        // Transition to pending phase
        if let Some(next_phase) = game_state.pending_phase {
            game_state.phase = next_phase;
            game_state.is_changed = true; // Trigger UI update
            
            // If we didn't win, we need to handle "attempts" counting or whatever logic was deferred?
            // Attempts were already incremented.
        }
        game_state.pending_phase = None;
    }
}

/// Manages the game's UI based on the current game state.
pub fn game_ui(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    entities: Query<Entity, With<GameEntity>>,
    query: Query<Entity, With<UIEntity>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    random_gen: ResMut<RandomGen>,
    time: Res<Time>,
) {
    // Check if the game state has changed from last frame before doing anything
    if !game_state.is_changed {
        return;
    }
    game_state.is_changed = false;

    // Clear all texts entities
    for entity in &query {
        commands.entity(entity).despawn();
    }

    // State Machine
    match game_state.phase {
        GamePhase::NotStarted => {
            if keyboard.just_pressed(KeyCode::Space) {
                // Transition: NotStarted -> Playing
                game_state.phase = GamePhase::Playing;
                game_state.start_time = Some(time.elapsed());
                game_state.nr_attempts = 0;
                game_state.is_changed = true;
                
            } else {
                // Display start screen
                let text = "Press SPACE to start the game! \nGame Commands: Arrow Keys/WASD: Rotate | SPACE: Check";
                spawn_centered_text_black_screen(&mut commands, text);
                game_state.is_changed = true;
            }
        }

        GamePhase::Playing => {
            // Display game UI
            let text = format!(
                "Arrow Keys/WASD: Rotate | SPACE: Check \nFind the RED face! | Attempts: {}",
                game_state.nr_attempts
            );
            commands.spawn((
                Text::new(text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                UIEntity,
            ));
            game_state.is_changed = true;
        }

        GamePhase::Won => {
            if keyboard.just_pressed(KeyCode::KeyR) {
                // Transition: Won -> NotStarted (restart)
                for entity in entities.iter() {
                    commands.entity(entity).despawn();
                }
                spawn_black_screen(&mut commands);
                setup(commands, meshes, materials, random_gen, time);
            } else {
                // Display win screen
                let elapsed = game_state.end_time.unwrap().as_secs_f32()
                    - game_state.start_time.unwrap().as_secs_f32();
                let accuracy = game_state.cosine_alignment.unwrap() * 100.0;

                let mut text = format!(
                    "Refresh (R) to play again\n\n\
                    CONGRATULATIONS! YOU WIN!\n\
                    - Time taken: {:.5} seconds\n\
                    - Attempts: {}\n\
                    - Alignment accuracy: {:.1}%",
                    elapsed, game_state.nr_attempts, accuracy
                );

                if game_state.nr_attempts == 1 {
                    text.push_str("\nPERFECT! First try!");
                }

                spawn_centered_text_black_screen(&mut commands, &text);
                game_state.is_changed = true;
            }
        }
    }
}
