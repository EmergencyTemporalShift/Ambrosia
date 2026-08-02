use bevy::prelude::*;

pub struct TimeToDespawnPlugin;

impl Plugin for TimeToDespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_despawn);
    }
}

#[derive(Component)]
pub struct TimeToDespawn(Timer);

impl TimeToDespawn {
    #[must_use]
    pub fn from_seconds(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Once))
    }
}
#[allow(clippy::needless_pass_by_value)]
fn handle_despawn(
    time: Res<Time>,
    mut query: Query<(Entity, &mut TimeToDespawn)>,
    mut commands: Commands,
) {
    for (entity, mut ttd) in query.iter_mut() {
        if ttd.0.tick(time.delta()).just_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}
