use avian2d::prelude::*;
use bevy::prelude::*;

pub mod components;
pub mod config;
pub mod spawner;
pub mod enemy;
pub mod player;
pub mod weapon_shooting;

// Re-export common items so external code isn't impacted by this refactor
pub use components::*;
pub use config::*;
pub use spawner::*;

#[allow(dead_code)]
#[derive(PhysicsLayer, Default)]
pub(crate) enum GameLayer {
    #[default]
    World,
    FriendlyUnit,
    FriendlyProjectile,
    EnemyUnit,
    EnemyProjectile,
    NeutralUnit,
    NeutralProjectile,
    HazardUnit,
    HazardProjectile,
}

pub struct LivingPlugin;

impl Plugin for LivingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Team>()
            .register_type::<CharacterSprite>()
            .register_type::<CharacterVisualConfig>()
            .register_type::<CharacterPhysicsConfig>();
    }
}

impl Team {
    pub fn is_hostile_to(&self, other: &Team) -> bool {
        match (self, other) {
            // Environment/Hazards harm everyone except other hazards
            (Team::Hazard, Team::Hazard) => false,
            (Team::Hazard, _) | (_, Team::Hazard) => true,

            // Neutral targets nobody
            (Team::Neutral, _) | (_, Team::Neutral) => false,

            // Same team never hurts each other
            (a, b) if a == b => false,

            // Player and Enemy oppose each other
            (Team::Player, Team::Enemy) | (Team::Enemy, Team::Player) => true,
            // Fallback: Same team (Player vs Player, Enemy vs Enemy, etc.)
            _ => false,
        }
    }
}