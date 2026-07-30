use std::mem::discriminant;
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
    /// Checks if two share the same enum variant head, ignoring inner data.
    pub fn is_same_variant(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
    
    /// Extracts the weapon position and aim if the intent is an active hold.
    pub fn spatial_data(&self) -> Option<(Vector2<CartesianSpace>, Vector2<CartesianSpace>)> {
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
