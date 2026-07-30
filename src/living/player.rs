use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian2d::prelude::*;
use bevy_tnua::{
    TnuaGhostOverwrites, TnuaObstacleRadar, TnuaToggle,
};
use bevy_tnua::control_helpers::{TnuaBlipReuseAvoidance, TnuaSimpleFallThroughPlatformsHelper};
use leafwing_input_manager::prelude::*;
use crate::character_control_systems::Dimensionality;
use crate::character_control_systems::platformer_control_scheme::{
    DemoControlScheme, DemoControlSchemeConfig,
};
use crate::character_control_systems::platformer_control_systems::CharacterMotionConfigForPlatformerDemo;
use crate::character_control_systems::player_input::PlayerAction;
use crate::levels_setup::for_2d_platformer::LayerNames;
use crate::ui::component_alteration::CommandAlteringSelectors;
use crate::living::{spawn_living, CharacterPhysicsConfig, CharacterVisualConfig, Team};
use crate::living::weapon_shooting::{SpawnWeaponExt, WeaponInventory};
use crate::living::weapon_shooting::WeaponKind::{Sword, Beam, Bow, HeavyBow};
#[cfg(feature = "egui")]
use crate::ui::info::InfoSource;
#[cfg(feature = "egui")]
use crate::ui::plotting::PlotSource;
use crate::ui::components::TrackedEntity;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct IsPlayer;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsPlayer>();
        app.add_systems(Startup, setup_player);
    }
}

pub fn setup_player(
    mut commands: Commands,
    mut control_scheme_config_assets: ResMut<Assets<DemoControlSchemeConfig>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {

    // Define what you want, map over them to spawn, and collect the Entity IDs
    let weapons: Vec<Entity> = [Sword, Bow, HeavyBow, Beam]
        .into_iter()
        .map(|kind| commands.spawn_weapon(kind))
        .collect();

    let player_entity = spawn_living(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        "Ambrosia",
        CharacterVisualConfig {
            texture_path: "Witchcraft_Sprites/Witchcraft_spr_1.png",
            tile_size: UVec2::new(24, 24),
            columns: 21,
            rows: 1,
            initial_frame_index: 0,
            sprite_scale: Vec3::new(0.25, 0.25, 1.0),
        },
        CharacterPhysicsConfig {
            collider: Collider::capsule(0.5, 1.0),
            lock_rotation: true,
            ..default()
        },
        |cmd| {
            cmd.insert((
                        IsPlayer,
                        Team::Player,
                        WeaponInventory::new(weapons.clone())
            ));

            cmd.insert((
                InputMap::default()
                    .with(PlayerAction::Fire, MouseButton::Left)
                    .with(PlayerAction::CyclePrev, KeyCode::BracketLeft)
                    .with(PlayerAction::CycleNext, KeyCode::BracketRight)
                    .with(PlayerAction::CycleNext, MouseScrollDirection::UP)
                    .with(PlayerAction::CyclePrev, MouseScrollDirection::DOWN),
                ActionState::<PlayerAction>::default(),
            ));

            cmd.insert((
                TnuaController::<DemoControlScheme>::default(),
                TnuaConfig::<DemoControlScheme>(control_scheme_config_assets.add(
                    DemoControlSchemeConfig {
                        ext: CharacterMotionConfigForPlatformerDemo {
                            dimensionality: Dimensionality::Dim2,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )),
            ));

            cmd.insert(TnuaObstacleRadar::new(1.0, 3.0));
            cmd.insert(TnuaBlipReuseAvoidance::<DemoControlScheme>::default());
            cmd.insert(TnuaToggle::default());

            cmd.insert({
                let command_altering_selectors = CommandAlteringSelectors::default()
                    .with_combo(
                        "Sensor Shape",
                        1,
                        &[
                            ("Point", |mut cmd| {
                                cmd.remove::<TnuaAvian2dSensorShape>();
                            }),
                            ("Flat (underfit)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::rectangle(
                                    0.99, 0.0,
                                )));
                            }),
                            ("Flat (exact)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::rectangle(1.0, 0.0)));
                            }),
                            ("flat (overfit)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::rectangle(
                                    1.01, 0.0,
                                )));
                            }),
                            ("Ball (underfit)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::circle(0.49)));
                            }),
                            ("Ball (exact)", |mut cmd| {
                                cmd.insert(TnuaAvian2dSensorShape(Collider::circle(0.5)));
                            }),
                        ],
                    )
                    .with_checkbox("Lock Tilt", false, |mut cmd, lock_tilt| {
                        if lock_tilt {
                            cmd.insert(LockedAxes::new().lock_rotation());
                        } else {
                            cmd.insert(LockedAxes::new());
                        }
                    })
                    .with_checkbox(
                        "Phase Through Collision Groups",
                        true,
                        |mut cmd, use_collision_groups| {
                            let player_layers: LayerMask = if use_collision_groups {
                                [LayerNames::Default, LayerNames::Player].into()
                            } else {
                                [
                                    LayerNames::Default,
                                    LayerNames::Player,
                                    LayerNames::PhaseThrough,
                                ]
                                    .into()
                            };
                            cmd.insert(CollisionLayers::new(player_layers, player_layers));
                        },
                    );
                command_altering_selectors
            });

            cmd.insert(TnuaGhostOverwrites::<DemoControlScheme>::default());
            cmd.insert(TnuaSimpleFallThroughPlatformsHelper::default());

            #[cfg(feature = "egui")]
            cmd.insert((
                TrackedEntity("Player".to_owned()),
                PlotSource::default(),
                InfoSource::default(),
            ));
        },
    );

    commands.entity(player_entity).add_children(&weapons);
}
