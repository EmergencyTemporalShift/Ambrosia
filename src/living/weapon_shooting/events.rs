use bevy::prelude::*;
use glamour::Vector2;
use crate::util::units::CartesianSpace;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponIntent {
    BeginHold { weapon_pos: Vector2<CartesianSpace>, aim: Vector2<CartesianSpace> },

    ContinueHold { weapon_pos: Vector2<CartesianSpace>, aim: Vector2<CartesianSpace> },

    ReleaseHold,

}

impl WeaponIntent {
    /// Extracts the weapon position and aim if the intent is an active hold.
    #[must_use]
    pub const fn spatial_data(&self) -> Option<(Vector2<CartesianSpace>, Vector2<CartesianSpace>)> {
        match *self {
            Self::BeginHold { weapon_pos, aim }
            | Self::ContinueHold { weapon_pos, aim } => Some((weapon_pos, aim)),
            Self::ReleaseHold => None,
        }
    }
}

#[derive(Event)]
pub struct FireWeapon {
    pub wielder: Entity, // The player/character entity (has LinearVelocity, Team, etc.)
    pub weapon: Entity,  // The weapon entity (has ProjectileSpawner, Melee, Beam, FireRate, etc.)
    pub intent: WeaponIntent,
}
