use bevy::app::FixedMain;
use bevy::prelude::*;

use bevy_tnua::builtins::{
    TnuaBuiltinClimb, TnuaBuiltinClimbConfig, TnuaBuiltinCrouch, TnuaBuiltinCrouchConfig,
    TnuaBuiltinDash, TnuaBuiltinDashConfig, TnuaBuiltinJump, TnuaBuiltinJumpConfig,
    TnuaBuiltinKnockback, TnuaBuiltinWalk, TnuaBuiltinWalkConfig, TnuaBuiltinWalkHeadroom,
    TnuaBuiltinWallSlide, TnuaBuiltinWallSlideConfig,
};
use bevy_tnua::control_helpers::{TnuaActionSlots, TnuaAirActionDefinition, TnuaHasTargetEntity};
use bevy_tnua::math::*;
use bevy_tnua::{TnuaConfig, TnuaConfigModifier, TnuaScheme};
use serde::{Deserialize, Serialize};
use crate::ui::tuning::UiTunable;
use super::Dimensionality;
// use crate::character_control_systems::platformer_control_scheme::FallingThroughControlScheme;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk, config_ext = CharacterMotionConfigForPlatformerDemo)]
pub enum DemoControlScheme {
    Jump(TnuaBuiltinJump),
    Crouch(
        TnuaBuiltinCrouch,
        #[scheme(modify_basis_config)] SlowDownWhileCrouching,
    ),
    Dash(TnuaBuiltinDash),
    Knockback(TnuaBuiltinKnockback),
    WallSlide(TnuaBuiltinWallSlide, Entity),
    #[scheme(same_trigger(Jump))]
    WallJump(TnuaBuiltinJump),
    Climb(
        TnuaBuiltinClimb,
        Entity,
        // Initiation direction:
        Vector3,
    ),
}

#[derive(Serialize, Deserialize)]
pub struct CharacterMotionConfigForPlatformerDemo {
    pub dimensionality: Dimensionality,
    pub jumps_in_air: usize,
    pub dashes_in_air: usize,
    pub one_way_platforms_min_proximity: Float,
    pub falling_through: FallingThroughControlScheme,
}

impl Default for CharacterMotionConfigForPlatformerDemo {
    fn default() -> Self {
        Self {
            dimensionality: Dimensionality::Dim3,
            jumps_in_air: 1,
            dashes_in_air: 1,
            one_way_platforms_min_proximity: 1.0,
            falling_through: FallingThroughControlScheme::SingleFall,
        }
    }
}

pub struct SlowDownWhileCrouching(pub bool);

impl TnuaConfigModifier<TnuaBuiltinWalkConfig> for SlowDownWhileCrouching {
    fn modify_config(&self, config: &mut TnuaBuiltinWalkConfig) {
        if self.0 {
            config.speed *= 0.2;
        }
    }
}

impl TnuaAirActionDefinition for DemoControlScheme {
    fn is_air_action(action: Self::ActionDiscriminant) -> bool {
        match action {
            DemoControlSchemeActionDiscriminant::Jump => true,
            DemoControlSchemeActionDiscriminant::Crouch => false,
            DemoControlSchemeActionDiscriminant::Dash => true,
            DemoControlSchemeActionDiscriminant::Knockback => true,
            DemoControlSchemeActionDiscriminant::WallSlide => true,
            DemoControlSchemeActionDiscriminant::WallJump => true,
            DemoControlSchemeActionDiscriminant::Climb => true,
        }
    }
}

#[derive(Debug, TnuaActionSlots)]
#[slots(scheme = DemoControlScheme, ending(WallSlide, WallJump, Climb))]
pub struct DemoControlSchemeAirActions {
    #[slots(Jump)]
    jump: usize,
    #[slots(Dash)]
    dash: usize,
}

impl TnuaHasTargetEntity for DemoControlScheme {
    fn target_entity(action_state: &Self::ActionState) -> Option<Entity> {
        match action_state {
            DemoControlSchemeActionState::Jump(_) => None,
            DemoControlSchemeActionState::Crouch(_, _) => None,
            DemoControlSchemeActionState::Dash(_) => None,
            DemoControlSchemeActionState::Knockback(_) => None,
            DemoControlSchemeActionState::WallSlide(_, entity) => Some(*entity),
            DemoControlSchemeActionState::WallJump(_) => None,
            DemoControlSchemeActionState::Climb(_, entity, _) => Some(*entity),
        }
    }
}

impl Default for DemoControlSchemeConfig {
    fn default() -> Self {
        Self {
            basis: TnuaBuiltinWalkConfig {
                float_height: 2.0,
                headroom: Some(TnuaBuiltinWalkHeadroom {
                    distance_to_collider_top: 1.0,
                    ..Default::default()
                }),
                max_slope: float_consts::FRAC_PI_4,
                ..Default::default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 4.0,
                ..Default::default()
            },
            crouch: TnuaBuiltinCrouchConfig {
                float_offset: -0.9,
                ..Default::default()
            },
            dash: TnuaBuiltinDashConfig {
                horizontal_distance: 10.0,
                vertical_distance: 0.0,
                ..Default::default()
            },
            knockback: Default::default(),
            wall_slide: TnuaBuiltinWallSlideConfig {
                maintain_distance: Some(0.7),
                ..Default::default()
            },
            wall_jump: TnuaBuiltinJumpConfig {
                height: 4.0,
                takeoff_extra_gravity: 90.0, // 3 times the default
                takeoff_above_velocity: 0.0,
                horizontal_distance: 2.0,
                ..Default::default()
            },
            climb: TnuaBuiltinClimbConfig {
                climb_speed: 10.0,
                ..Default::default()
            },
            ext: CharacterMotionConfigForPlatformerDemo {
                dimensionality: Dimensionality::Dim3,
                jumps_in_air: 1,
                dashes_in_air: 1,
                one_way_platforms_min_proximity: 1.0,
                falling_through: FallingThroughControlScheme::SingleFall,
            },
        }
    }
}

impl UiTunable for CharacterMotionConfigForPlatformerDemo {
    #[cfg(feature = "egui")]
    fn tune(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.jumps_in_air, 0..=8).text("Max Jumps in Air"));
        ui.add(egui::Slider::new(&mut self.dashes_in_air, 0..=8).text("Max Dashes in Air"));
        ui.collapsing("One-way Platforms", |ui| {
            ui.add(
                egui::Slider::new(&mut self.one_way_platforms_min_proximity, 0.0..=2.0)
                    .text("Min Proximity"),
            );
            self.falling_through.tune(ui);
        });
    }
}

impl UiTunable for DemoControlSchemeConfig {
    #[cfg(feature = "egui")]
    fn tune(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Walking:", |ui| self.basis.tune(ui));
        ui.collapsing("Jumping:", |ui| self.jump.tune(ui));
        ui.collapsing("Dashing:", |ui| self.dash.tune(ui));
        ui.collapsing("Crouching:", |ui| self.crouch.tune(ui));
        ui.collapsing("Knockback:", |ui| self.knockback.tune(ui));
        ui.collapsing("Wall Slide:", |ui| self.wall_slide.tune(ui));
        ui.collapsing("Wall Jump:", |ui| self.wall_jump.tune(ui));
        ui.collapsing("Climb", |ui| self.climb.tune(ui));
        self.ext.tune(ui);
    }
}

#[derive(Component, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum FallingThroughControlScheme {
    JumpThroughOnly,
    WithoutHelper,
    #[default]
    SingleFall,
    KeepFalling,
}

impl UiTunable for FallingThroughControlScheme {
    #[cfg(feature = "egui")]
    fn tune(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Falling Through Control Scheme")
            .selected_text(format!("{:?}", self))
            .show_ui(ui, |ui| {
                for variant in [
                    FallingThroughControlScheme::JumpThroughOnly,
                    FallingThroughControlScheme::WithoutHelper,
                    FallingThroughControlScheme::SingleFall,
                    FallingThroughControlScheme::KeepFalling,
                ] {
                    if ui.selectable_label(*self == variant, format!("{:?}", variant)).clicked() {
                        *self = variant;
                    }
                }
            });
    }
}

pub trait CameraController {
    fn camera_forward(&self) -> Vector3;
    fn calculate_transform_for_controls(&self, screen_space_forward: Dir3, player_up: Dir3) -> Transform {
        let forward = self
            .camera_forward()
            .reject_from(player_up.adjust_precision())
            .normalize();
        Transform::default().with_rotation(
            Quaternion::from_rotation_arc(screen_space_forward.adjust_precision(), forward).f32(),
        )
    }
}

#[derive(Component)]
pub struct CameraControllerMounted {
    pub forward: Vector3,
    pub pitch_angle: Float,
}

impl Default for CameraControllerMounted {
    fn default() -> Self {
        Self { forward: Vector3::NEG_Z, pitch_angle: 0.0 }
    }
}

impl CameraController for CameraControllerMounted {
    fn camera_forward(&self) -> Vector3 { self.forward.adjust_precision() }
}

#[derive(Component)]
pub struct CameraControllerFloating {
    pub looking_from: Vector3,
    pub looking_to: Vector3,
}

impl CameraController for CameraControllerFloating {
    fn camera_forward(&self) -> Vector3 { self.looking_to - self.looking_from }
}

pub struct JustPressedCachePlugin;

impl Plugin for JustPressedCachePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JustPressedCache>();
        app.add_systems(
            RunFixedMainLoop,
            (
                collect_just_pressed_cache.before(FixedMain::run_fixed_main),
                clear_just_pressed_cache.after(FixedMain::run_fixed_main),
            ),
        );
    }
}

fn collect_just_pressed_cache(
    query: Query<&TnuaConfig<DemoControlScheme>>,
    config_assets: Res<Assets<DemoControlSchemeConfig>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut just_pressed: ResMut<JustPressedCache>,
) {
    for config in &query {
        let Some(config) = config_assets.get(&config.0) else { continue; };
        let crouch_buttons = match config.ext.dimensionality {
            Dimensionality::Dim2 => CROUCH_BUTTONS_2D.iter().copied(),
            Dimensionality::Dim3 => CROUCH_BUTTONS_3D.iter().copied(),
        };
        just_pressed.crouch = keyboard.any_just_pressed(crouch_buttons);
    }
}

fn clear_just_pressed_cache(mut just_pressed: ResMut<JustPressedCache>) {
    if just_pressed.was_read { *just_pressed = default() }
}

// Make fields and constants public so platformer_control_systems.rs can access them
#[derive(Resource, Default)]
pub struct JustPressedCache {
    pub crouch: bool,
    pub was_read: bool,
}

pub const CROUCH_BUTTONS_2D: &[KeyCode] = &[KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::ArrowDown, KeyCode::KeyS];
pub const CROUCH_BUTTONS_3D: &[KeyCode] = &[KeyCode::ControlLeft, KeyCode::ControlRight];