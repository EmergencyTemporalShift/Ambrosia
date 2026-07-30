use bevy::prelude::*;
use avian2d::prelude::*;

use crate::util::particles::{spawn_particle_entity, ParticleEffectHandle};
use super::components::*;
use super::events::{FireWeapon, WeaponIntent};

use bevy::render::mesh::PrimitiveTopology;
use bevy::render::mesh::Indices;
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::basic::*;
use bevy::ecs::observer::On;
use bevy::window::PrimaryWindow;
use glamour::Vector2;
use leafwing_input_manager::prelude::ActionState;
use crate::character_control_systems::player_input::PlayerAction;
use crate::living::player::IsPlayer;
use crate::util::units::{BevyVec2Ext, CartesianSpace, Vector2Ext};

pub fn setup_beam_observers(app: &mut App) {
    app.add_observer(|
        // Immediate, event-driven reactive scheduling
        event: On<FireWeapon>,
        mut commands: Commands,
        wielders: Query<&Beam>,
        mut active_beams: Query<&mut BeamTerminator>,
        spatial_query: SpatialQuery,
        effect_handle: Res<ParticleEffectHandle>,
    | {
        let wielder = event.event().wielder;
        let weapon = event.event().weapon;
        let weapon_pos = event.event().weapon_pos;
        let aim = event.event().aim;
        let intent = event.event().intent;

        dbg!(intent);

        let dir = (aim - weapon_pos).normalize_or_zero();

        let Ok(beam) = wielders.get(weapon) else { return };
        // Consider a tag for objects that attacks can pass though.
        let filter = SpatialQueryFilter::default().with_excluded_entities([wielder]);


        let end_point = if let Some(hit) = spatial_query.cast_ray(
            weapon_pos.to_bevy(),
            Dir2::new(dir.to_bevy()).unwrap_or(Dir2::Y),
            beam.range,
            true,
            &filter,
        ) {
            (weapon_pos + dir * hit.distance).extend(0.0)
        } else {
            (weapon_pos + dir * beam.range).extend(0.0)
        }.truncate();

        match intent {
            WeaponIntent::BeginHold | WeaponIntent::ContinueHold => {
                if let Ok(mut active_beam) = active_beams.get_mut(wielder) {
                    active_beam.end_point = end_point;
                } else {
                    commands.entity(wielder).insert(BeamTerminator { end_point });
                }
                spawn_particle_entity(&mut commands, &effect_handle, end_point.to_bevy().extend(0.0));
            }
            WeaponIntent::ReleaseHold => {
                trigger_beam_fade(commands, wielder);
            }
        }
    });
}

pub fn manage_beam_visuals(
    mut commands: Commands,
    active_beams: Query<(Entity, &BeamTerminator), Changed<BeamTerminator>>,
    visual_lines: Query<(Entity, &BeamLine)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (source_entity, _beam) in &active_beams {
        let already_exists = visual_lines.iter().any(|(_, line)| line.source_entity == source_entity);

        if !already_exists {
            // 1. Create the mesh
            let mut mesh = Mesh::new(
                PrimitiveTopology::LineStrip,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );

            // 2. Fix: Provide dummy placeholder data so the allocator doesn't choke on 0 vertices
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]]);
            mesh.insert_indices(Indices::U32(vec![0]));

            commands.spawn((
                Transform::default(),
                Visibility::default(),
                Mesh2d(meshes.add(mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(RED)))),
                BeamLine { source_entity },
            ));
        }
    }
}

pub fn update_beam_visuals(
    // mut commands: Commands,
    //Get the mesh handle and source info from our visual entities
    visual_lines: Query<(&Mesh2d, &BeamLine)>,
    //Look up the gameplay position of the entity casting the beam
    sources: Query<(&GlobalTransform, &BeamTerminator)>,
    //Get access to the mutable mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // let active_count = active_beams.iter().count();
    // info!("Active beams this frame: {}", active_count);
    //info!("update_beam_visuals");
    for (mesh_handle, beam_line) in &visual_lines {
        // info!("Updating beam visuals for line {:?}", &beam_line.source_entity);
        // Find the weapon that owns this specific beam line
        if let Ok((transform, active_beam)) = sources.get(beam_line.source_entity) {
            // Get the live start and end coordinates
            let start = transform.translation();
            let end = active_beam.end_point;

            // Grab a mutable reference to this line's mesh data
            if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
                // Construct the positions (Bevy meshes require Vec3 format [x, y, z])
                let new_positions = vec![
                    [start.x, start.y, 0.0],
                    [end.x, end.y, 0.0],
                ];
                // commands.spawn_debug_circle(Vec2::new(start.x, start.y), PURPLE.into(), 30.0, 0.1);
                // commands.spawn_debug_circle(Vec2::new(end.x, end.y), GREEN.into(), 30.0, 0.1);
                // Update the vertex positions asset data
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);

                // Re-bind the indices to connect the two new points
                mesh.insert_indices(Indices::U32(vec![0, 1]));
            }
        }
    }
}
/// Does not query, runs when called
pub fn trigger_beam_fade(mut commands: Commands, wielder: Entity) {
    commands.entity(wielder)
        .remove::<BeamTerminator>()
        .insert(BeamFadeTimer(Timer::from_seconds(0.3, TimerMode::Once)));
}

pub fn cleanup_inactive_beams(
    mut commands: Commands,
    visual_lines: Query<(Entity, &BeamLine)>,
    active_beams: Query<&BeamTerminator>,
) {
    for (visual_entity, beam_line) in &visual_lines {
        // If the source entity doesn't exist anymore, or no longer has BeamActive, destroy the line!
        if active_beams.get(beam_line.source_entity).is_err() {
            commands.entity(visual_entity).despawn();
        }
    }
}

pub fn player_fire_input(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    player: Query<(Entity, &Transform, &ActionState<PlayerAction>, &WeaponInventory), With<IsPlayer>>,
    weapons_q: Query<Option<&FireRate>, With<Weapon>>,
    // #[cfg(feature = "egui")]
    // mut egui_contexts: EguiContexts,
) {
    // #[cfg(feature = "egui")]
    // if egui_contexts
    //     .ctx_mut()
    //     .map_or(false, |ctx| ctx.egui_wants_pointer_input())
    // {
    //     return;
    // }

    let (cam, cam_tf) = *camera;
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = cam.viewport_to_world(cam_tf, cursor) else {
        return;
    };
    let world_pos = ray.origin.truncate();

    for (entity, transform, action_state, wp_inv) in &player {
        let active_weapon_entity = match wp_inv.current() {
            Some(entity) => entity,
            None => continue,
        };

        let mut intent = WeaponIntent::ContinueHold; // Default to ContinueHold if pressed but not begin/release

        let is_fire_pressed = action_state.pressed(&PlayerAction::Fire);
        let just_fired_begin = action_state.just_pressed(&PlayerAction::Fire);
        let just_fired_release = action_state.just_released(&PlayerAction::Fire);

        if just_fired_release {
            intent = WeaponIntent::ReleaseHold;
        } else if just_fired_begin {
            intent = WeaponIntent::BeginHold;
        } else if !is_fire_pressed {
            // If the action is not pressed at all, we should not be here if just_released was also false.
            // This case handles when the button is not being held and no new press/release happened.
            continue;
        }
        // If we are here, either just_released was true, just_pressed was true, or is_fire_pressed is true.
        // The 'intent' variable will correctly reflect the current state.

        // --- Event Triggering and Cooldown Logic ---

        // Handle ReleaseHold immediately, as it does not depend on cooldowns.
        if intent == WeaponIntent::ReleaseHold {
            commands.trigger(FireWeapon {
                wielder: entity,
                weapon: active_weapon_entity,
                weapon_pos: transform.translation.truncate().to_space(),
                aim: world_pos.to_space::<CartesianSpace>(),
                intent,
            });
            continue; // ReleaseHold is processed; skip cooldown checks.
        }

        // For BeginHold and ContinueHold intents, check the weapon's cooldown.
        if let Ok(Some(current_wep)) = weapons_q.get(active_weapon_entity) {
            if !current_wep.0.is_finished() {
                continue; // Weapon is on cooldown
            }
        }
        // Note: If FireRate is missing, we assume it's always ready.

        // Trigger FireWeapon event for BeginHold or ContinueHold if Aim is valid.
        let aim = world_pos.to_space::<CartesianSpace>();
        if aim != Vector2::ZERO {
            commands.trigger(FireWeapon {
                wielder: entity,
                weapon: active_weapon_entity,
                weapon_pos: transform.translation.truncate().to_space(),
                aim,
                intent,
            });
        }
    }
}
