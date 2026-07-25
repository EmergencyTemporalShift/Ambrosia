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
use crate::util::units::Vector2Ext;

pub fn setup_beam_observers(app: &mut App) {
    app.add_observer(|
        // Immediate, event-driven reactive scheduling
        event: On<FireWeapon>,
        mut commands: Commands,
        wielders: Query<&Beam>,
        mut active_beams: Query<&mut BeamActive>,
        spatial_query: SpatialQuery,
        effect_handle: Res<ParticleEffectHandle>,
    | {
        let wielder = event.event().wielder;
        let weapon_pos = event.event().weapon_pos;
        let aim = event.event().aim;
        let intent = event.event().intent;

        info!("{:?}", weapon_pos);
        let dir = (aim - weapon_pos).normalize_or_zero();

        let Ok(beam) = wielders.get(wielder) else { return };
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
                    commands.entity(wielder).insert(BeamActive { end_point });
                }
                spawn_particle_entity(&mut commands, &effect_handle, end_point.to_bevy().extend(0.0));
            }
            WeaponIntent::ReleaseHold => {
                commands.entity(wielder).remove::<BeamActive>();
            }
        }
    });
}

pub fn manage_beam_visuals(
    mut commands: Commands,
    active_beams: Query<(Entity, &BeamActive), Changed<BeamActive>>,
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
    // 1. Get the mesh handle and source info from our visual entities
    visual_lines: Query<(&Mesh2d, &BeamLine)>,
    // 2. Look up the gameplay position of the entity casting the beam
    sources: Query<(&GlobalTransform, &BeamActive)>,
    // 3. Get access to the mutable mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (mesh_handle, beam_line) in &visual_lines {
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

                // Update the vertex positions asset data
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);

                // Re-bind the indices to connect the two new points
                mesh.insert_indices(Indices::U32(vec![0, 1]));
            }
        }
    }
}

pub fn cleanup_inactive_beams(
    mut commands: Commands,
    // Find all visual beam lines
    visual_lines: Query<(Entity, &BeamLine)>,
    // A query to check if the source entity still has an active beam component
    active_beams: Query<&BeamActive>,
) {
    for (visual_entity, beam_line) in &visual_lines {
        // If the source entity doesn't exist anymore, or no longer has BeamActive, destroy the line!
        if active_beams.get(beam_line.source_entity).is_err() {
            commands.entity(visual_entity).despawn();
        }
    }
}
