use bevy::prelude::*;
use glamour::Vector2;
use crate::util::units::CartesianSpace;

#[derive(Component)]
pub struct Weapon {
    pub damage: f32,
}

#[derive(Component)]
pub struct Ammo {
    pub max_ammo: u32,
    pub max_projectiles: u32,
    pub current_ammo: u32,
}

#[derive(Component)]
pub struct AmmoFluid {
    pub max_ammo: f32,
    pub max_projectiles: u32,
    pub current_ammo: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Debug, Reflect)]
pub enum WeaponKind {
    Bow,
    Sword,
    HeavyBow,
    Beam,
}

#[derive(Component, Reflect)]
pub struct WeaponInventory {
    pub slots: Vec<WeaponKind>,
    pub active: usize,
}

impl WeaponInventory {
    pub fn current(&self) -> Option<WeaponKind> {
        self.slots.get(self.active).copied()
    }

    pub fn cycle(&mut self, forward: bool) {
        if self.slots.is_empty() {
            return;
        }
        if forward {
            self.active = (self.active + 1) % self.slots.len();
        } else {
            self.active = (self.active + self.slots.len() - 1) % self.slots.len();
        }
    }
}

#[derive(Component, Debug)]
pub struct WeaponOffset {
    pub(crate) offset: f32,
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct WeaponSprite;

#[derive(Component)]
pub struct ProjectileSpawner {
    pub projectile_name: String,
    pub speed: f32,
    pub max_simultaneous_projectiles: usize,
    pub lifetime: f32,
    pub collider_width: f32,
    pub collider_height: f32,
}

#[derive(Component)]
pub struct Hitscan {
    pub range: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct Beam {
    pub range: f32,
    pub damage_per_second: f32,
}

#[derive(Component)]
pub struct BeamActive {
    pub end_point: Vector2<CartesianSpace>,
}

#[derive(Component)]
pub struct BeamLine {
    // Stores the entity ID of the source casting the beam
    pub source_entity: Entity,
}

#[derive(Component)]
pub struct Melee {
    pub arc: f32,
    pub reach: f32,
    pub damage: f32,
    pub cooldown: Timer,
}

#[derive(Component)]
pub struct Tackle {
    pub damage: f32,
    pub cooldown: Timer,
}

#[derive(Component, Default)]
pub struct FireRate(pub Timer);

#[derive(Component)]
pub struct Projectile {
    pub fired_by: Entity,
}

#[derive(Component, Default)]
pub struct LifetimeTimer {
    pub remaining: f32,
}

#[derive(Component)]
#[require(Transform, LifetimeTimer)]
pub struct DebugMarker {
    pub color: Color,
    pub radius: f32,
}