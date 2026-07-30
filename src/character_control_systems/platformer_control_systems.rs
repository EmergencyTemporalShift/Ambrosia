use crate::character_control_systems::platformer_control_scheme::{CROUCH_BUTTONS_2D, CROUCH_BUTTONS_3D};
use crate::character_control_systems::platformer_control_scheme::{CameraController, CameraControllerFloating};
use crate::character_control_systems::platformer_control_scheme::FallingThroughControlScheme;
pub(crate) use crate::character_control_systems::platformer_control_scheme::JustPressedCache;
use crate::character_control_systems::platformer_control_scheme::CameraControllerMounted;
use std::cmp::Ordering;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_tnua::{TnuaConfig, TnuaController, TnuaGhostOverwrites, TnuaSensorsEntities};
use bevy_tnua::builtins::{TnuaBuiltinClimb, TnuaBuiltinCrouchMemory, TnuaBuiltinDash, TnuaBuiltinWallSlide};
use bevy_tnua::control_helpers::{TnuaActionsCounter, TnuaBlipReuseAvoidance, TnuaSimpleFallThroughPlatformsHelper};
use bevy_tnua::prelude::*;
use bevy_tnua::radar_lens::{TnuaBlipSpatialRelation, TnuaRadarLens};
use bevy_tnua_physics_integration_layer::data_for_backends::TnuaGhostSensor;
use bevy_tnua_physics_integration_layer::math::{AdjustPrecision, AsF32, Float};
use bevy_tnua_physics_integration_layer::obstacle_radar::TnuaObstacleRadar;
use glamour::Vector3;
use crate::character_control_systems::Dimensionality;
use crate::character_control_systems::platformer_control_scheme::{DemoControlScheme, DemoControlSchemeActionDiscriminant, DemoControlSchemeActionState, DemoControlSchemeAirActions, DemoControlSchemeConfig, SlowDownWhileCrouching};
use crate::character_control_systems::querying_helpers::ObstacleQueryHelper;
use crate::character_control_systems::spatial_ext_facade::SpatialExtFacade;

// 1. Define the QueryData struct to consolidate the massive tuple
#[derive(QueryData)]
#[query_data(mutable)]
pub struct PlatformerControlQuery {
    pub controller: &'static mut TnuaController<DemoControlScheme>,
    pub config: &'static TnuaConfig<DemoControlScheme>,
    pub sensors_entities: &'static TnuaSensorsEntities<DemoControlScheme>,
    pub ghost_overwrites: &'static mut TnuaGhostOverwrites<DemoControlScheme>,
    pub fall_through_helper: &'static mut TnuaSimpleFallThroughPlatformsHelper,
    pub air_actions: &'static TnuaActionsCounter<DemoControlSchemeAirActions>,
    pub camera_controller: (
        Option<&'static CameraControllerFloating>,
        Option<&'static CameraControllerMounted>,
    ),
    pub obstacle_radar: &'static TnuaObstacleRadar,
    pub blip_reuse_avoidance: &'static mut TnuaBlipReuseAvoidance<DemoControlScheme>,
}

#[allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::useless_conversion
)]
pub fn apply_platformer_controls(
    #[cfg(feature = "egui")] mut egui_context: EguiContexts,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut just_pressed: ResMut<JustPressedCache>,
    // 2. Utilize the new QueryData struct in the signature
    mut query: Query<PlatformerControlQuery>,
    config_assets: Res<Assets<DemoControlSchemeConfig>>,
    spatial_ext: SpatialExtFacade,
    obstacle_query: Query<ObstacleQueryHelper>,
    ghost_sensors_query: Query<&TnuaGhostSensor>,
    #[cfg(feature = "egui")] mut is_egui_initialized: Local<bool>,
) {
    // Egui conditional block meticulously preserved
    #[cfg(feature = "egui")]
    {
        if !*is_egui_initialized {
            *is_egui_initialized = true;
        } else if let Ok(ctx) = egui_context.ctx_mut() {
            if ctx.egui_wants_keyboard_input() {
                for mut q in query.iter_mut() {
                    // The basis remembers its last frame status, so if we cannot feed it proper input this
                    // frame (for example, because the GUI takes the input focus), we need to neutralize it.
                    q.controller.basis = Default::default();
                }
                return;
            }
        }
    }

    for mut q in query.iter_mut() {
        // 3. Rebind the struct fields to local variables.
        // This prevents us from having to rewrite `q.` in front of hundreds of references below,
        // drastically reducing the chance of introducing a typo into the core physics logic.
        let mut controller = q.controller;
        let config = q.config;
        let sensors_entities = q.sensors_entities;
        let mut ghost_overwrites = q.ghost_overwrites;
        let mut fall_through_helper = q.fall_through_helper;
        let air_actions = q.air_actions;
        let camera_controller = q.camera_controller;
        let obstacle_radar = q.obstacle_radar;
        let mut blip_reuse_avoidance = q.blip_reuse_avoidance;

        let Some(config) = config_assets.get(&config.0) else {
            continue;
        };
        controller.initiate_action_feeding();
        let up_direction = controller.up_direction().unwrap_or(Dir3::Y);

        // This part is just keyboard input processing. In a real game this would probably be done
        // with a third party plugin.
        let mut direction = Vector3::ZERO;

        let is_climbing =
            controller.action_discriminant() == Some(DemoControlSchemeActionDiscriminant::Climb);

        if config.ext.dimensionality == Dimensionality::Dim3 || is_climbing {
            if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
                direction -= Vector3::Z;
            }
            if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
                direction += Vector3::Z;
            }
        }
        if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            direction -= Vector3::X;
        }
        if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            direction += Vector3::X;
        }

        let screen_space_direction = direction.clamp_length_max(1.0);

        let transform_for_controls = match camera_controller {
            (None, None) => None,
            (None, Some(camera)) => Some(camera as &dyn CameraController),
            (Some(camera), None) => Some(camera as &dyn CameraController),
            (Some(_), Some(_)) => panic!("both floating and mounted cameras at the same time"),
        }
            .map(|c| c.calculate_transform_for_controls(Dir3::NEG_Z, up_direction))
            .unwrap_or_default();
        let direction = transform_for_controls
            .transform_point(screen_space_direction.f32())
            .adjust_precision();

        let jump = match (config.ext.dimensionality, is_climbing) {
            (Dimensionality::Dim2, true) => keyboard.any_pressed([KeyCode::Space]),
            (Dimensionality::Dim2, false) => {
                keyboard.any_pressed([KeyCode::Space, KeyCode::ArrowUp, KeyCode::KeyW])
            }
            (Dimensionality::Dim3, _) => keyboard.any_pressed([KeyCode::Space]),
        };
        let dash = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        let has_mounted_camera = {
            let mounted_camera_controller: &Option<&CameraControllerMounted> = &camera_controller.1;
            mounted_camera_controller.is_some()
        };
        let turn_in_place =
            !has_mounted_camera && keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

        let crouch_buttons = match (config.ext.dimensionality, is_climbing) {
            (Dimensionality::Dim2, true) => CROUCH_BUTTONS_3D.iter().copied(),
            (Dimensionality::Dim2, false) => CROUCH_BUTTONS_2D.iter().copied(),
            (Dimensionality::Dim3, _) => CROUCH_BUTTONS_3D.iter().copied(),
        };
        let crouch_pressed = keyboard.any_pressed(crouch_buttons);
        let crouch_just_pressed = just_pressed.crouch;
        just_pressed.was_read = true;

        // This also needs to be called once per frame. It checks which obstacles needs to be
        // blocked - e.g. because we've just finished an action on them, and we don't want to
        // reinitiate that action.
        blip_reuse_avoidance.update(controller.as_ref(), obstacle_radar);

        // Here we will handle one-way platforms. It looks long and complex, but it's actual
        // several schemes with observable changes in behavior, and each implementation is rather
        // short and simple.
        let crouch;
        if let Some(ghost_sensor) = sensors_entities
            .ground
            .and_then(|entity| ghost_sensors_query.get(entity).ok())
        {
            match config.ext.falling_through {
                FallingThroughControlScheme::JumpThroughOnly => {
                    crouch = crouch_pressed;
                    ghost_overwrites
                        .ground
                        .set(ghost_sensor.iter().find(|ghost_platform| {
                            config.ext.one_way_platforms_min_proximity <= ghost_platform.proximity
                        }));
                }
                FallingThroughControlScheme::WithoutHelper => {
                    let relevant_platform = ghost_sensor.iter().find(|ghost_platform| {
                        config.ext.one_way_platforms_min_proximity <= ghost_platform.proximity
                    });
                    if crouch_pressed {
                        crouch = relevant_platform.is_none();
                        ghost_overwrites.ground.set(None);
                    } else {
                        crouch = false;
                        ghost_overwrites.ground.set(relevant_platform);
                    }
                }
                FallingThroughControlScheme::SingleFall => {
                    let mut handler = fall_through_helper.with(
                        &mut ghost_overwrites.ground,
                        ghost_sensor,
                        config.ext.one_way_platforms_min_proximity,
                    );
                    if crouch_pressed {
                        crouch = !handler.try_falling(crouch_just_pressed);
                    } else {
                        crouch = false;
                        handler.dont_fall();
                    }
                }
                FallingThroughControlScheme::KeepFalling => {
                    let mut handler = fall_through_helper.with(
                        &mut ghost_overwrites.ground,
                        ghost_sensor,
                        config.ext.one_way_platforms_min_proximity,
                    );
                    if crouch_pressed {
                        crouch = !handler.try_falling(true);
                    } else {
                        crouch = false;
                        handler.dont_fall();
                    }
                }
            };
        } else {
            crouch = crouch_pressed;
        }

        let slow_down_while_crouching = SlowDownWhileCrouching(
            if let Some(DemoControlSchemeActionState::Crouch(state, _)) =
                controller.current_action.as_ref()
            {
                !matches!(state.memory, TnuaBuiltinCrouchMemory::Rising)
            } else {
                false
            },
        );

        controller.basis = TnuaBuiltinWalk {
            desired_motion: if turn_in_place {
                Vector3::ZERO
            } else {
                direction
            },
            desired_forward: if let Some(CameraControllerMounted { forward, .. }) =
                camera_controller.1
            {
                Dir3::new(forward.f32()).ok()
            } else {
                Dir3::new(direction.f32()).ok()
            },
        };

        let radar_lens = TnuaRadarLens::new(obstacle_radar, &spatial_ext);

        let already_sliding_on = if let Some(DemoControlSchemeActionState::WallSlide(_, entity)) =
            controller.current_action.as_ref()
            && obstacle_radar.has_blip(*entity)
        {
            Some(*entity)
        } else {
            None
        };

        let already_climbing_on =
            if let Some(DemoControlSchemeActionState::Climb(state, entity, initiation_direction)) =
                controller.current_action.as_ref()
                && obstacle_radar.has_blip(*entity)
            {
                Some((*entity, state.input.clone(), *initiation_direction))
            } else {
                None
            };

        let mut walljump_candidate = None;

        'blips_loop: for blip in radar_lens.iter_blips() {
            if !blip_reuse_avoidance.should_avoid(blip.entity())
                && obstacle_query
                .get(blip.entity())
                .expect("ObstacleQueryHelper has nothing that could fail when missing")
                .climbable
            {
                if let Some((climbable_entity, action, initiation_direction)) =
                    already_climbing_on.as_ref()
                {
                    if *climbable_entity != blip.entity() {
                        continue 'blips_loop;
                    }
                    let dot_initiation = direction.dot(*initiation_direction);
                    let initiation_direction = if 0.5 < dot_initiation {
                        *initiation_direction
                    } else {
                        Vector3::ZERO
                    };
                    if initiation_direction == Vector3::ZERO {
                        let right_left = screen_space_direction.dot(Vector3::X);
                        if 0.5 <= right_left.abs() {
                            continue 'blips_loop;
                        }
                    }

                    let mut action = TnuaBuiltinClimb {
                        anchor: blip.closest_point().get(),
                        desired_climb_motion: screen_space_direction.dot(Vector3::NEG_Z)
                            * Vector3::Y,
                        desired_vec_to_anchor: action.desired_vec_to_anchor,
                        desired_forward: action.desired_forward,
                        ..Default::default()
                    };

                    const LOOK_ABOVE_OR_BELOW: Float = 5.0;
                    match action
                        .desired_climb_motion
                        .dot(Vector3::Y)
                        .partial_cmp(&0.0)
                        .unwrap()
                    {
                        Ordering::Less => {
                            if controller.is_airborne().unwrap() {
                                let extent = blip
                                    .probe_extent_from_closest_point(-Dir3::Y, LOOK_ABOVE_OR_BELOW);
                                if extent < 0.9 * LOOK_ABOVE_OR_BELOW {
                                    action.hard_stop_down =
                                        Some(blip.closest_point().get() - extent * Vector3::Y);
                                }
                            } else if initiation_direction == Vector3::ZERO {
                                continue 'blips_loop;
                            } else {
                                action.desired_climb_motion = Vector3::ZERO;
                            }
                        }
                        Ordering::Equal => {}
                        Ordering::Greater => {
                            let extent =
                                blip.probe_extent_from_closest_point(Dir3::Y, LOOK_ABOVE_OR_BELOW);
                            if extent < 0.9 * LOOK_ABOVE_OR_BELOW {
                                action.hard_stop_up =
                                    Some(blip.closest_point().get() + extent * Vector3::Y);
                            }
                        }
                    }

                    controller.action(DemoControlScheme::Climb(
                        action,
                        blip.entity(),
                        initiation_direction,
                    ));
                } else if let TnuaBlipSpatialRelation::Aeside(blip_direction) =
                    blip.spatial_relation(0.5)
                    && 0.5 < direction.dot(blip_direction.adjust_precision())
                {
                    let direction_to_anchor = match config.ext.dimensionality {
                        Dimensionality::Dim2 => Vector3::ZERO,
                        Dimensionality::Dim3 => -blip
                            .normal_from_closest_point()
                            .reject_from_normalized(Vector3::Y),
                    };
                    controller.action(DemoControlScheme::Climb(
                        TnuaBuiltinClimb {
                            anchor: blip.closest_point().get(),
                            desired_vec_to_anchor: 0.5 * direction_to_anchor,
                            desired_forward: Dir3::new(direction_to_anchor.f32()).ok(),
                            ..Default::default()
                        },
                        blip.entity(),
                        direction.normalize_or_zero(),
                    ));
                }
            }
            if !blip.is_interactable() {
                continue;
            }
            match blip.spatial_relation(0.5) {
                TnuaBlipSpatialRelation::Invalid => {}
                TnuaBlipSpatialRelation::Above => {}
                TnuaBlipSpatialRelation::Below => {}
                TnuaBlipSpatialRelation::Aeside(blip_direction) => {
                    let dot_threshold = if Some(blip.entity()) == already_sliding_on {
                        -0.1
                    } else {
                        0.0
                    };
                    if controller.is_airborne().unwrap() {
                        let dot_direction = direction.dot(blip_direction.adjust_precision());

                        if Some(blip.entity()) == already_sliding_on {
                            walljump_candidate = Some((blip.entity(), -blip_direction));
                        }

                        if dot_threshold < dot_direction
                            && 0.8 < blip.flat_wall_score(Dir3::Y, &[-1.0, 1.0])
                        {
                            let Ok(normal) = Dir3::new(blip.normal_from_closest_point().f32())
                            else {
                                continue;
                            };
                            controller.action(DemoControlScheme::WallSlide(
                                TnuaBuiltinWallSlide {
                                    contact_point_with_wall: blip.closest_point().get(),
                                    normal,
                                    force_forward: Some(blip_direction),
                                },
                                blip.entity(),
                            ));
                        }
                    }
                }
            }
        }

        if crouch {
            controller.action(DemoControlScheme::Crouch(
                Default::default(),
                slow_down_while_crouching,
            ));
        }

        if jump {
            let action_flow_status = controller.action_flow_status().clone();
            if matches!(
                action_flow_status.ongoing(),
                Some(
                    DemoControlSchemeActionDiscriminant::Jump
                        | DemoControlSchemeActionDiscriminant::WallJump
                )
            ) {
                controller.prolong_action();
            } else if let Some((_, walljump_direction)) = walljump_candidate {
                controller.action(DemoControlScheme::WallJump(TnuaBuiltinJump {
                    horizontal_displacement: Some(walljump_direction.adjust_precision()),
                    allow_in_air: true,
                    force_forward: Some(-walljump_direction),
                }));
            } else {
                let current_action_discriminant = controller.action_discriminant();
                controller.action(DemoControlScheme::Jump(TnuaBuiltinJump {
                    allow_in_air: air_actions.count_for(DemoControlSchemeActionDiscriminant::Jump)
                        <= config.ext.jumps_in_air
                        || current_action_discriminant == Some(DemoControlSchemeActionDiscriminant::Climb),
                    ..Default::default()
                }));
            }
        }

        if dash {
            controller.action(DemoControlScheme::Dash(TnuaBuiltinDash {
                displacement: direction.normalize()
                    + up_direction.adjust_precision(),
                desired_forward: if has_mounted_camera {
                    None
                } else {
                    Dir3::new(direction.f32()).ok()
                },
                allow_in_air: air_actions.count_for(DemoControlSchemeActionDiscriminant::Dash)
                    <= config.ext.dashes_in_air,
            }));
        }
    }
}