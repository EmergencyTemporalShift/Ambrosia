use bevy::prelude::*;

pub mod beam;
pub mod components;
pub mod equip;
pub mod events;
pub mod melee;
pub mod projectiles;

pub use components::*;
pub use equip::*;
pub use events::*;

use beam::*;
use melee::*;
use projectiles::*;

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        setup_projectile_observers(app);
        setup_melee_observers(app);
        setup_beam_observers(app);
            app.add_systems(
                Update,
                (
                    manage_beam_visuals,
                    update_beam_visuals,
                    cleanup_inactive_beams,
                    process_projectile_ttl,
                    enable_projectile_wielder_collisions,
                    tick_fire_rate,
                    apply_active_weapon,
                    draw_debug_markers,
                    tick_lifetime_timers,
                ).chain(),
            );
    }
}

pub fn tick_fire_rate(
    mut fire_rates: Query<&mut FireRate>,
    mut melee_cooldowns: Query<&mut Melee>,
    time: Res<Time>,
) {
    let delta = time.delta();

    for mut fr in &mut fire_rates {
        fr.0.tick(delta);
    }

    for mut melee in &mut melee_cooldowns {
        melee.cooldown.tick(delta);
    }
}

impl DebugMarker {
    pub fn circle(pos: Vec2, color: Color, radius: f32, duration: f32) -> impl Bundle {
        (
            DebugMarker { color, radius },
            Transform::from_translation(pos.extend(100.0)),
            LifetimeTimer { remaining: duration },
        )
    }
}

pub trait SpawnDebugMarkerExt {
    fn spawn_debug_circle(&mut self, pos: Vec2, color: Color, radius: f32, duration: f32);
}

impl SpawnDebugMarkerExt for Commands<'_, '_> {
    fn spawn_debug_circle(&mut self, pos: Vec2, color: Color, radius: f32, duration: f32) {
        self.spawn((
            DebugMarker { color, radius },
            Transform::from_translation(pos.extend(100.0)),
            LifetimeTimer { remaining: duration },
        ));
    }
}

pub fn draw_debug_markers(
    query: Query<(&Transform, &DebugMarker)>,
    mut gizmos: Gizmos,
) {
    for (transform, marker) in &query {
        gizmos.circle_2d(
            transform.translation.truncate(), // Extracts the 2D (x, y) coordinates
            marker.radius,
            marker.color,
        );
    }
}

pub fn tick_lifetime_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LifetimeTimer)>,
) {
    for (entity, mut timer) in &mut query {
        timer.remaining -= time.delta_secs();
        if timer.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}