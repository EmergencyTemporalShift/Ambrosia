use crate::living::Team;
use crate::util::units::CartesianSpace;
use bevy::prelude::*;
use glamour::Vector2;

#[derive(Component)]
pub struct Weapon {
    pub weapon_kind: WeaponKind,
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
    // Keep this private so other systems can't arbitrarily push non-weapons
    slots: Vec<Entity>,
    pub active: usize,
}

impl WeaponInventory {

    // Create a new inventory from a starting list
    pub fn new(starting_weapons: Vec<Entity>) -> Self {
        Self {
            slots: starting_weapons,
            active: 0,
        }
    }

    // A system can call this to safely add a weapon, proving it has the component
    pub fn add_weapon(&mut self, entity: Entity, weapon_query: &Query<&Weapon>) -> Result<(), &'static str> {
        if weapon_query.contains(entity) {
            self.slots.push(entity);
            Ok(())
        } else {
            Err("Attempted to add an entity to WeaponInventory that does not have a Weapon component!")
        }
    }
    
    pub fn current(&self) -> Option<Entity> {
        self.slots.get(self.active).copied()
    }

    pub fn slots(&self) -> &[Entity] {
        &self.slots
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

#[derive(Component, Clone, Debug, Reflect)]
pub struct WeaponVisualConfig {
    pub texture_path: &'static str,
    pub tile_size: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub initial_frame_index: usize,
    pub sprite_scale: Vec3,
    pub sprite_angle_offset: f32,
}

impl Default for WeaponVisualConfig {
    fn default() -> Self {
        Self {
            texture_path: "",
            tile_size: UVec2::new(24, 24),
            columns: 10,
            rows: 1,
            initial_frame_index: 0,
            sprite_scale: Vec3::new(0.25, 0.25, 1.0),
            sprite_angle_offset: 0.0,
        }
    }
}


#[derive(Component, Default)]
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
pub struct BeamTerminator {
    pub end_point: Vector2<CartesianSpace>,
}

#[derive(Component)]
pub struct BeamFadeTimer(pub Timer);

#[derive(Component)]
pub struct BeamLine {
    // Stores the entity ID of the source casting the beam
    pub source_entity: Entity,
}

#[derive(Component)]
pub struct Melee {
    pub arc: f32,
    pub reach: f32,
}

#[derive(Component)]
pub struct Cooldown {
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
    pub team: Team,
    pub shooter: Entity,
    pub weapon: Entity,
}

