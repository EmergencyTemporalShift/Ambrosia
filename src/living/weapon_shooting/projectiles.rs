use super::components::*;
use super::events::FireWeapon;
//use bevy::color::palettes::basic::{GREEN, PURPLE, RED, YELLOW};
use crate::living::{GameLayer::*, Team};
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::util::debug::LifetimeTimer;
use crate::util::units::{BevyVec2Ext, Vector2Ext};
use bevy::ecs::observer::On;
use crate::living::weapon_shooting::WeaponIntent;

pub fn setup_projectile_observers(app: &mut App) {
    app.add_observer(|
        event: On<FireWeapon>,
        wielder_q: Query<(&LinearVelocity, Option<&Team>)>,
        mut weapon_q: Query<(&Weapon, &ProjectileSpawner, Option<&mut FireRate>)>,
        projectiles: Query<&Projectile>,
        mut commands: Commands,
    | {
        let intent = event.event().intent;

        if intent == WeaponIntent::ReleaseHold {
            return;
        }

        let shooter = event.event().wielder;
        let weapon = event.event().weapon;

        let Ok((_weapon, spawner, mut fire_rate)) = weapon_q.get_mut(weapon) else { return; };

        if let Some(ref mut fr) = fire_rate {
            if !fr.0.is_finished() {
                return;
            }
            fr.0.reset();
        }

        let Some((weapon_pos, aim)) = intent.spatial_data() else {
            return;
        };

        // FIXED: Calculate direction vector instead of using raw aim position
        let direction = (aim - weapon_pos).normalize_or_zero();

        let Ok((vel, team_opt)) = wielder_q.get(shooter) else {
            return;
        };

        let team = team_opt.copied().unwrap_or(Team::Neutral);

        let active = projectiles.iter().filter(|p| p.weapon == weapon).count();
        if active >= spawner.max_simultaneous_projectiles {
            info!("Max projectiles hit for weapon {:?}.", weapon);
            return;
        }

        // FIXED: Using direction for velocity and angle math
        let shot_vel = direction * spawner.speed + vel.0.to_space();
        let angle = direction.y.atan2(direction.x);

        let collision_layers = build_team_interactions(team_opt);

        commands.spawn((
            Name(spawner.projectile_name.clone().into()),
            Projectile { team, shooter, weapon },
            LifetimeTimer { remaining: spawner.lifetime },
            Transform::from_translation(weapon_pos.to_bevy().extend(0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            LinearVelocity(shot_vel.to_bevy()),
            Collider::rectangle(spawner.collider_width, spawner.collider_height),
            RigidBody::Dynamic,
            collision_layers,
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
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn enable_projectile_wielder_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile, &Transform, &CollisionLayers)>,
    wielders: Query<(&Transform, Option<&Team>)>,
) {
    for (proj_entity, projectile, proj_transform, current_layers) in projectiles.iter() {
        if let Ok((wielder_transform, w_team)) = wielders.get(projectile.shooter) {
            let distance = proj_transform
                .translation
                .truncate()
                .distance(wielder_transform.translation.truncate());

            if distance > 2.0 {
                let collision_layers = build_team_interactions(w_team);
                if *current_layers != collision_layers {
                    commands.entity(proj_entity).insert(collision_layers);
                }
            }
        }
    }
}

fn build_team_interactions(team_opt: Option<&Team> ) -> CollisionLayers {
    match team_opt {
        Some(Team::Player) | Some(Team::Neutral) | None => CollisionLayers::new(
            FriendlyProjectile,
            [World, EnemyUnit, EnemyProjectile],
        ),
        Some(Team::Enemy) => CollisionLayers::new(
            EnemyProjectile,
            [World, FriendlyUnit, FriendlyProjectile],
        ),
        Some(Team::Hazard) => CollisionLayers::new(
            HazardProjectile,
            [World, EnemyUnit, EnemyProjectile, FriendlyUnit, FriendlyProjectile],
        ),
    }
}