use std::f32::consts::{FRAC_PI_3, FRAC_PI_4};
use bevy::prelude::*;
use super::components::{WeaponKind, Weapon, WeaponVisualConfig, ProjectileSpawner, FireRate, Melee, Cooldown, Beam};

// pub(crate) fn weapon_bundle(kind: WeaponKind) -> (
//     Weapon,
//     WeaponKind,
//     WeaponVisualConfig,
//     Option<ProjectileSpawner>,
//     Option<Melee>,
//     Option<FireRate>,
//     Option<Beam>,
// ) {
//     match kind {
//         WeaponKind::Bow => (
//             Weapon { damage: 1.5 },
//             kind,
//             WeaponVisualConfig {
//                 texture_path: "weapons/bow/Bow Pack.png",
//                 tile_size: UVec2::new(24, 24),
//                 columns: 6,
//                 rows: 6,
//                 initial_frame_index: 25,
//                 sprite_scale: Vec3::splat(0.25),
//                 sprite_angle_offset: FRAC_PI_4,
//             },
//             Some(ProjectileSpawner {
//                 projectile_name: "Arrow".to_string(),
//                 speed: 40.0,
//                 max_simultaneous_projectiles: 12,
//                 lifetime: 5.0,
//                 collider_width: 1.0,
//                 collider_height: 0.2,
//             }),
//             None,
//             Some(FireRate(Timer::from_seconds(0.3, TimerMode::Once))),
//             None,
//         ),
//         WeaponKind::Sword => (
//             Weapon { damage: 10.0 },
//             kind,
//             WeaponVisualConfig {
//                 texture_path: "weapons/Swords/sword_icons.png",
//                 tile_size: UVec2::new(16, 16),
//                 columns: 6,
//                 rows: 4,
//                 initial_frame_index: 12,
//                 sprite_scale: Vec3::splat(0.25),
//                 sprite_angle_offset: -FRAC_PI_4,
//             },
//             None,
//             Some(Melee {
//                 arc: FRAC_PI_3,
//                 reach: 40.0,
//                 cooldown: Timer::from_seconds(0.4, TimerMode::Once),
//             }),
//             None,
//             None,
//         ),
//         WeaponKind::HeavyBow => (
//             Weapon { damage: 3.0 },
//             kind,
//             WeaponVisualConfig {
//                 texture_path: "weapons/bow/Bow Pack.png",
//                 tile_size: UVec2::new(24, 24),
//                 columns: 6,
//                 rows: 6,
//                 initial_frame_index: 31,
//                 sprite_scale: Vec3::splat(0.25),
//                 sprite_angle_offset: FRAC_PI_4,
//             },
//             Some(ProjectileSpawner {
//                 projectile_name: "Heavy Arrow".to_string(),
//                 speed: 40.0,
//                 max_simultaneous_projectiles: 3,
//                 lifetime: 5.0,
//                 collider_width: 1.2,
//                 collider_height: 0.35,
//             }),
//             None,
//             Some(FireRate(Timer::from_seconds(0.6, TimerMode::Once))),
//             None,
//         ),
//         WeaponKind::Beam => (
//             Weapon { damage: 0.01 },
//             kind,
//             WeaponVisualConfig {
//                 texture_path: "weapons/beam_circle_2.png",
//                 tile_size: UVec2::new(24, 24),
//                 columns: 1,
//                 rows: 1,
//                 initial_frame_index: 0,
//                 sprite_scale: Vec3::splat(0.25),
//                 sprite_angle_offset: 0.0,
//             },
//             None,
//             None,
//             None,
//             Some(Beam { range: 300.0, damage_per_second: 15.0 }),
//         ),
//     }
// }

pub trait SpawnWeaponExt {
    fn spawn_weapon(&mut self, kind: WeaponKind) -> Entity;
}

impl<'w, 's> SpawnWeaponExt for Commands<'w, 's> {
    fn spawn_weapon(&mut self, kind: WeaponKind) -> Entity {
        // Spawn an empty entity and grab its ID
        let mut ent_cmd = self.spawn_empty();
        let entity = ent_cmd.id();


        ent_cmd.insert((
            Transform::from_translation(Vec3::new(0.,0.,0.1)),
            Visibility::default(),
        ));

        // Insert components directly based on the weapon kind.
        // Because we have direct access to the EntityCommands wrapper,
        // we only insert the components the specific weapon actually uses.
        match kind {
            WeaponKind::Bow => {
                ent_cmd.insert((
                    Weapon { weapon_kind: kind, damage: 1.5 },
                    kind,
                    WeaponVisualConfig {
                        texture_path: "weapons/bow/Bow Pack.png",
                        tile_size: UVec2::new(24, 24),
                        columns: 6,
                        rows: 6,
                        initial_frame_index: 25,
                        sprite_scale: Vec3::splat(0.25),
                        sprite_angle_offset: FRAC_PI_4,
                    },
                    ProjectileSpawner {
                        projectile_name: "Arrow".to_string(),
                        speed: 40.0,
                        max_simultaneous_projectiles: 12,
                        lifetime: 5.0,
                        collider_width: 1.0,
                        collider_height: 0.2,
                    },
                    FireRate(Timer::from_seconds(0.3, TimerMode::Once)),
                ));
            }
            WeaponKind::HeavyBow => {
                ent_cmd.insert((
                    Weapon { weapon_kind: kind, damage: 3.0 },
                    kind,
                    WeaponVisualConfig {
                        texture_path: "weapons/bow/Bow Pack.png",
                        tile_size: UVec2::new(24, 24),
                        columns: 6,
                        rows: 6,
                        initial_frame_index: 31,
                        sprite_scale: Vec3::splat(0.25),
                        sprite_angle_offset: FRAC_PI_4,
                    },
                    ProjectileSpawner {
                        projectile_name: "Heavy Arrow".to_string(),
                        speed: 40.0,
                        max_simultaneous_projectiles: 3,
                        lifetime: 5.0,
                        collider_width: 1.2,
                        collider_height: 0.35,
                    },
                    FireRate(Timer::from_seconds(0.6, TimerMode::Once)),
                ));
            }
            WeaponKind::Sword => {
                ent_cmd.insert((
                    Weapon { weapon_kind: kind, damage: 25.0 },
                    kind,
                    WeaponVisualConfig {
                        texture_path: "weapons/Swords/sword_icons.png",
                        tile_size: UVec2::new(16, 16),
                        columns: 6,
                        rows: 4,
                        initial_frame_index: 12,
                        sprite_scale: Vec3::splat(0.25),
                        sprite_angle_offset: -FRAC_PI_4,
                    },
                    Melee {
                        arc: FRAC_PI_3,
                        reach: 40.0,
                    },
                    Cooldown {
                        cooldown: Timer::from_seconds(0.4, TimerMode::Once),
                    }
                ));
            }
            WeaponKind::Beam => {
                ent_cmd.insert((
                    Weapon { weapon_kind: kind, damage: 0.01 },
                    kind,
                    WeaponVisualConfig {
                        texture_path: "weapons/beam_circle_2.png",
                        tile_size: UVec2::new(24, 24),
                        columns: 1,
                        rows: 1,
                        initial_frame_index: 0,
                        sprite_scale: Vec3::splat(0.25),
                        sprite_angle_offset: 0.0,
                    },
                    Beam { range: 300.0, damage_per_second: 15.0 },
                ));
            }
        }
        entity
    }
}
