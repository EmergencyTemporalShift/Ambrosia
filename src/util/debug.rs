#![allow(clippy::needless_pass_by_value)]
use avian2d::parry::glamx::Vec2;
use bevy::color::Color;
use bevy::prelude::{Bundle, Commands, Component, Entity, Gizmos, Query, Res, Time, Transform};

#[derive(Component)]
#[require(Transform, LifetimeTimer)]
pub struct DebugMarker {
    pub color: Color,
    pub radius: f32,
}

impl DebugMarker {
    #[must_use]
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

#[derive(Component, Default)]
pub struct LifetimeTimer {
    pub remaining: f32,
}

pub fn tick_lifetime_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LifetimeTimer)>,
) {
    for (entity, mut timer) in &mut query {
        timer.remaining -= time.delta_secs();
        if timer.remaining <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}