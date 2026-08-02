use bevy::prelude::*;
use avian2d::prelude::*;
use crate::util::particles::ParticleEffectHandle;
use super::components::{Beam, BeamTerminator, BeamLine, BeamFadeTimer};
use super::events::FireWeapon;

use bevy::render::mesh::PrimitiveTopology;
use bevy::render::mesh::Indices;
use bevy::asset::RenderAssetUsages;
#[allow(clippy::wildcard_imports)]
use bevy::color::palettes::basic::*;
use bevy::ecs::observer::On;
use glamour::Vector2;
use crate::util::units::{CartesianSpace, Vector2Ext};

pub fn setup_beam_observers(app: &mut App) {
    app.add_observer(|
        event: On<FireWeapon>,
        commands: Commands,
        wielders: Query<&Beam>,
        _active_beams: Query<&mut BeamTerminator>,
        spatial_query: SpatialQuery,
        _effect_handle: Res<ParticleEffectHandle>,
    | {
        let wielder = event.event().wielder;
        let weapon = event.event().weapon;
        let intent = event.event().intent;

        // 1. Pull the data, or intercept the ReleaseHold!
        let Some((weapon_pos, aim)) = intent.spatial_data() else {
            trigger_beam_fade(commands, wielder);
            return;
        };

        // 2. Calculate direction and raycast
        let dir = (aim - weapon_pos).normalize_or_zero();

        let Ok(beam) = wielders.get(weapon) else { return };

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

        trigger_beam_activate(commands, wielder, end_point);
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
/// Does not query, runs when called
pub fn trigger_beam_activate(mut commands: Commands, wielder: Entity, end_point: Vector2<CartesianSpace>) {
    commands.entity(wielder)
        .insert(BeamTerminator{ end_point })
        .remove::<BeamFadeTimer>();
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
