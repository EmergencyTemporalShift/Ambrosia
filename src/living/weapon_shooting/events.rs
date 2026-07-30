use bevy::prelude::*;
use glamour::Vector2;
use crate::util::units::CartesianSpace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponIntent {
    BeginHold,
    ContinueHold,
    ReleaseHold,
}

#[derive(Event)]
pub struct FireWeapon {
    pub wielder: Entity, // The player/character entity (has LinearVelocity, Team, etc.)
    pub weapon: Entity,  // The weapon entity (has ProjectileSpawner, Melee, Beam, FireRate, etc.)
    pub weapon_pos: Vector2<CartesianSpace>,
    pub aim: Vector2<CartesianSpace>,
    pub intent: WeaponIntent,
}
