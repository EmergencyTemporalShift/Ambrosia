use std::cmp::PartialEq;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

use bevy::window::PrimaryWindow;
use leafwing_input_manager::prelude::*;
#[cfg(feature = "egui")]
use bevy_egui::EguiContexts;
use crate::living::player::IsPlayer;
use crate::living::{CharacterSprite, flip_sprite_for_direction};
use crate::living::weapon_shooting::{FireWeapon, Weapon, WeaponInventory, WeaponIntent, WeaponVisualConfig, BeamTerminator};
use crate::living::weapon_shooting::beam::trigger_beam_fade;
use crate::living::weapon_shooting::weapon_system::rotate_active_weapon;
use crate::util::game_states::GameState;
use crate::util::units::{BevyVec2Ext, CartesianSpace, Vector2Ext, WindowSpace};

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Fire,
    CycleNext,
    CyclePrev,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlayerInputSet;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

        // FIX 1: Move immediate inputs and visual alignments to Update
        app.add_systems(
            Update,
            (
                player_weapon_face_mouse,
                player_fire_input,
                flip_sprite_to_mouse,
                player_cycle_weapon,
            )
                .run_if(in_state(GameState::Running))
                .in_set(PlayerInputSet),
        );

        app.add_systems(
            Update,
            (
                update_weapon_selection_text,
                debug_player_components,
            )
                .in_set(PlayerInputSet),
        );
    }
}

pub fn player_fire_input(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    player: Query<(Entity, &Transform, &ActionState<PlayerAction>, &WeaponInventory), With<IsPlayer>>,
    active_beams: Query<&BeamTerminator>,
    #[cfg(feature = "egui")]
    mut egui_contexts: EguiContexts,
) {
    #[cfg(feature = "egui")]
    let egui_wants_pointer = egui_contexts
        .ctx_mut()
        .map_or(false, |ctx| ctx.egui_wants_pointer_input());
    #[cfg(not(feature = "egui"))]
    let egui_wants_pointer = false;

    let (cam, cam_tf) = *camera;

    for (entity, transform, action_state, wp_inv) in &player {
        let Some(active_weapon_entity) = wp_inv.current() else {
            continue;
        };

        let is_pressed = action_state.pressed(&PlayerAction::Fire);
        let just_pressed = action_state.just_pressed(&PlayerAction::Fire);
        let has_active_beam = active_beams.get(entity).is_ok();

        // 1. Release check: Runs even if mouse is off-screen or over UI
        if !is_pressed {
            if has_active_beam {
                commands.trigger(FireWeapon {
                    wielder: entity,
                    weapon: active_weapon_entity,
                    intent: WeaponIntent::ReleaseHold,
                });
            }
            continue;
        }

        // 2. If clicking on UI, ignore new/continued firing
        if egui_wants_pointer {
            continue;
        }

        // 3. Aim check: Only run when actively trying to fire
        let Some(cursor_pos) = window.cursor_position() else {
            continue;
        };
        let Ok(ray) = cam.viewport_to_world(cam_tf, cursor_pos) else {
            continue;
        };

        let weapon_pos = transform.translation.truncate().to_space();
        let aim = ray.origin.truncate().to_space::<CartesianSpace>();

        let intent = if just_pressed && !has_active_beam {
            WeaponIntent::BeginHold { weapon_pos, aim }
        } else {
            WeaponIntent::ContinueHold { weapon_pos, aim }
        };

        commands.trigger(FireWeapon {
            wielder: entity,
            weapon: active_weapon_entity,
            intent,
        });
    }
}

pub fn player_weapon_face_mouse(
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_q: Query<(&GlobalTransform, &WeaponInventory), With<IsPlayer>>,
    mut weapon_q: Query<(&mut Transform, Option<&WeaponVisualConfig>), With<Weapon>>,
) {
    // 1. Extract required world entities
    let Ok(window) = window_q.single() else { return; };
    let Ok((camera, camera_gtf)) = camera_q.single() else { return; };
    let Ok((player_gtf, inventory)) = player_q.single() else { return; };

    // 2. Look up the active weapon
    let Some(active_weapon_entity) = inventory.current() else { return; };
    let Ok((mut weapon_tf, visual_config)) = weapon_q.get_mut(active_weapon_entity) else {
        return;
    };

    // 3. Resolve cursor to world coordinates
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let window_cursor = cursor_pos.to_space::<WindowSpace>();

    let Ok(world_cursor) = camera.viewport_to_world_2d(camera_gtf, window_cursor.to_bevy()) else {
        return;
    };

    // 4. Calculate aim vector & target angle
    let player_pos = player_gtf.translation().truncate().to_space();
    let aim_vector = world_cursor.to_space::<CartesianSpace>() - player_pos;

    let sprite_offset = visual_config
        .map(|config| config.sprite_angle_offset)
        .unwrap_or(0.0);

    let aim_angle = aim_vector.to_angle() + sprite_offset;

    rotate_active_weapon(&mut weapon_tf, aim_angle);
}

pub fn player_cycle_weapon(
    mut commands: Commands,
    mut inventory_q: Query<(Entity, &mut WeaponInventory, &ActionState<PlayerAction>), With<IsPlayer>>,
    wielders_q: Query<&BeamTerminator>,
    #[cfg(feature = "egui")]
    mut egui_contexts: EguiContexts,
) {
    #[cfg(feature = "egui")]
    if egui_contexts
        .ctx_mut()
        .map_or(false, |ctx| ctx.egui_wants_pointer_input())
    {
        return;
    }

    let Ok((player, mut inv, action_state)) = inventory_q.single_mut() else {
        return;
    };

    // Determine the number of steps to cycle forward or backward.
    // If CycleNext/CyclePrev are digital, value() returns 1.0 when pressed.
    // If mapped to a continuous scroll axis, this can accumulate multiple ticks in a single frame.
    let next_pressed = action_state.just_pressed(&PlayerAction::CycleNext);
    let prev_pressed = action_state.just_pressed(&PlayerAction::CyclePrev);

    if !next_pressed && !prev_pressed {
        return;
    }

    // Fade out the beam if active before switching weapons
    if wielders_q.get(player).is_ok() {
        trigger_beam_fade(commands.reborrow(), player);
    }

    // Handle mutually exclusive or simultaneous cancellations safely
    if next_pressed && !prev_pressed {
        inv.cycle(true);
    } else if prev_pressed && !next_pressed {
        inv.cycle(false);
    }
}


pub fn flip_sprite_to_mouse(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut player_sprite_query: Query<(&ChildOf, &mut Sprite), With<CharacterSprite>>,
    player_query: Query<&GlobalTransform, With<IsPlayer>>,
) {
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (parent, mut sprite) in player_sprite_query.iter_mut() {
        if let Ok(player_transform) = player_query.get(parent.0) {
            let player_x = player_transform.translation().x;
            flip_sprite_for_direction(&mut sprite, world_pos.x - player_x);
        }
    }
}

// ─── Debug Text ─────────────────────────────────────────────

#[derive(Component)]
pub struct DebugTextTag;

pub fn setup_weapon_text(mut commands: Commands) {
    commands.spawn((
        Text::new("Weapon Status: Unknown"),
        TextFont {
            font_size: FontSize::Px(30.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        DebugTextTag,
    ));
}

pub fn update_weapon_selection_text(
    player_q: Query<&WeaponInventory, With<IsPlayer>>,
    weapon_q: Query<&Weapon>,
    mut text_query: Query<&mut Text, With<DebugTextTag>>,
) {
    let Ok(mut text): std::result::Result<Mut<Text>, QuerySingleError> = text_query.single_mut() else { return };
    let Ok(inventory) = player_q.single() else { return };

    if let Some(active_entity) = inventory.current() {
        let entity_id = active_entity;

        if let Ok(weapon) = weapon_q.get(entity_id) {
            text.0 = format!("Weapon Equipped: {:?}", weapon.weapon_kind);
            return;
        }
    }

    text.0 = "Weapon Equipped: None".to_string();
}


pub fn debug_player_components(
    player_query: Query<Entity, With<IsPlayer>>,
) {
    let count = player_query.iter().count();
    if count > 1 {
        warn!("CRITICAL: Found {} entities with IsPlayer! The shooting system might be grabbing the wrong one.", count);
    }
}