use avian2d::prelude::*;
use bevy::color::palettes::basic::{GREEN, PURPLE, RED, YELLOW};
use bevy::prelude::*;
use crate::living::{GameLayer::*, Team};
use super::components::*;
use super::events::FireWeapon;

use bevy::ecs::observer::On;
use crate::living::weapon_shooting::SpawnDebugMarkerExt;
use crate::util::units::{BevyVec2Ext, Vector2Ext};

pub fn setup_projectile_observers(app: &mut App) {
    app.add_observer(|
        // 1. Listen for the FireWeapon event payload
        event: On<FireWeapon>,
        // 2. Fetch projectile attributes specifically for the wielder targeting this event
        mut wielders: Query<(&Weapon, &ProjectileSpawner, &LinearVelocity, Option<&mut FireRate>, Option<&Team>)>,
        // 3. Keep your projectiles query exactly as it was to verify active limits
        projectiles: Query<&Projectile>,
        mut commands: Commands,
        mut gizmos: Gizmos,
    | {
        // Extract values from the event payload wrapper
        let wielder = event.event().wielder;
        let origin = event.event().weapon_pos;
        let direction = (event.event().aim - origin).normalize_or_zero();

        // Pull components belonging to the combatant spawning the projectile
        let Ok((_, spawner, vel, mut fire_rate, wielder_team)) = wielders.get_mut(wielder) else {
            return;
        };

        // Reset the fire rate tracking clock if it is attached
        if let Some(ref mut fr) = fire_rate {
            fr.0.reset();
        }

        // Enforce the simultaneous active ammo limits per wielder
        let active = projectiles.iter().filter(|p| p.fired_by == wielder).count();
        if active >= spawner.max_simultaneous_projectiles {
            // Put time left, either the fire rate or despawn time left, whichever is less
            info!("Max projectiles hit.");
            return;
        }

        // Calculate velocity vectors and geometry angles
        let shot_vel = direction * spawner.speed + vel.0.to_space();

        commands.spawn_debug_circle(origin.to_bevy(), YELLOW.into(), 10.0, 0.5);
        commands.spawn_debug_circle(direction.to_bevy(), PURPLE.into(), 10.0, 0.5);
        commands.spawn_debug_circle(event.event().aim.to_bevy(), GREEN.into(), 10.0, 0.5);
        info!("---------------------------------------------");
        info!("Spawning projectile with aim: {:?}", event.event().aim);
        info!("Spawning projectile with origin: {:?}", origin);
        info!("Spawning projectile with vel: {:?}", vel.0);
        info!("Spawning projectile with direction: {:?}", direction);
        info!("Spawning projectile with shot_vel: {:?}", shot_vel);
        gizmos.line_2d(origin.to_bevy(), direction.to_bevy(), RED);

        let angle = direction.y.atan2(direction.x);
        info!("Spawning projectile with angle {:?}", angle);


        // Map team definitions over to defensive collision filtering matrices
        let (proj_layer, target_layers) = match wielder_team {
            Some(Team::Player) | Some(Team::Neutral) | None => (
                FriendlyProjectile,
                [World, EnemyUnit, EnemyProjectile],
            ),
            Some(Team::Enemy) => (
                EnemyProjectile,
                [World, FriendlyUnit, FriendlyProjectile],
            ),
        };

        // Instantiate the projectile entity in the game world
        commands.spawn((
            Name(spawner.projectile_name.clone().into()),
            Projectile { fired_by: wielder },
            LifetimeTimer { remaining: spawner.lifetime },
            Transform::from_translation(origin.to_bevy().extend(0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            LinearVelocity(shot_vel.to_bevy().normalize_or_zero()*40.0),
            Collider::rectangle(spawner.collider_width, spawner.collider_height),
            RigidBody::Dynamic,
            CollisionLayers::new(proj_layer, target_layers),
        ));
    });
}


pub fn process_projectile_ttl(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut LifetimeTimer)>,
) {
    let delta = time.delta_secs();

    for (entity, mut timer) in projectiles.iter_mut() {
        timer.remaining -= delta;

        if timer.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn enable_projectile_wielder_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile, &Transform, &CollisionLayers)>,
    wielders: Query<(&Transform, Option<&Team>)>,
) {
    for (proj_entity, projectile, proj_transform, current_layers) in projectiles.iter() {
        if let Ok((wielder_transform, w_team)) = wielders.get(projectile.fired_by) {
            let distance = proj_transform
                .translation
                .truncate()
                .distance(wielder_transform.translation.truncate());

            if distance > 2.0 {
                let (proj_layer, target_layers) = match w_team {
                    Some(Team::Player) | Some(Team::Neutral) | None => (
                        FriendlyProjectile,
                        [World, EnemyUnit, EnemyProjectile, FriendlyProjectile],
                    ),
                    Some(Team::Enemy) => (
                        EnemyProjectile,
                        [World, FriendlyUnit, FriendlyProjectile, EnemyProjectile],
                    ),
                };

                let new_layers = CollisionLayers::new(proj_layer, target_layers);

                if *current_layers != new_layers {
                    commands.entity(proj_entity).insert(new_layers);
                }
            }
        }
    }
}