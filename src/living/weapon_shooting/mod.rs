use bevy::prelude::*;

pub mod beam;
pub mod components;
pub mod equip;
pub mod events;
pub mod melee;
pub mod projectiles;
pub mod weapon_system;

pub use components::*;
pub use equip::*;
pub use events::*;

use crate::living::weapon_shooting::weapon_system::*;
use crate::util::debug::{draw_debug_markers, tick_lifetime_timers};
use beam::*;
use melee::*;
use projectiles::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct WeaponSet;
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        // Setup observers
        setup_projectile_observers(app);
        setup_melee_observers(app);
        setup_beam_observers(app);

        // 2. Register all Update systems under WeaponSet in a single call
        // 1. Beam Domain Systems
        app.add_systems(
            Update,
            (
                manage_beam_visuals,
                update_beam_visuals,
                cleanup_inactive_beams,
            )
                .in_set(WeaponSet),
        );

        // 2. Projectile Domain Systems
        app.add_systems(
            Update,
            (
                process_projectile_ttl,
                enable_projectile_wielder_collisions,
            )
                .in_set(WeaponSet),
        );

        // 3. Weapon Logic & Visuals
        app.add_systems(
            Update,
            (
                tick_cooldowns,
                build_weapon_visuals,
                apply_active_weapon,
            )
                .in_set(WeaponSet),
        );

        // 4. Utilities / Timers
        app.add_systems(
            Update,
            (
                draw_debug_markers,
                tick_lifetime_timers,
            )
                .in_set(WeaponSet),
        );

    }
}

