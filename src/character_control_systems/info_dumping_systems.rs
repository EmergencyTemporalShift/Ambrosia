#![allow(clippy::needless_pass_by_value)]
use bevy::{color::palettes::css, prelude::*};
use bevy_tnua::TnuaSensorsSet;
use bevy_tnua::prelude::TnuaController;
use bevy_tnua::{
    TnuaGhostSensor, TnuaObstacleRadar, TnuaProximitySensor, math::AsF32, radar_lens::TnuaRadarLens,
};

use crate::ui::info::InfoSource;

use super::platformer_control_scheme::DemoControlScheme;
use super::spatial_ext_facade::SpatialExtFacade;

#[allow(clippy::type_complexity)]
pub fn character_control_info_dumping_system(
    mut query: Query<(
        &mut InfoSource,
        &TnuaController<DemoControlScheme>,
        &TnuaSensorsSet,
        Option<&TnuaObstacleRadar>,
    )>,
    sensors_query: Query<(&TnuaProximitySensor, Option<&TnuaGhostSensor>)>,
    names_query: Query<&Name>,
) {
    for (mut info_source, controller, sensors, obstacle_radar) in query.iter_mut() {
        if !info_source.is_active() {
            continue;
        }
        info_source.label(
            "Action",
            controller
                .action_discriminant()
                .map(|action| format!("{action:?}"))
                .unwrap_or_default(),
        );
        for sensor_entity in sensors.iter() {
            let Ok((sensor, ghost_sensor)) = sensors_query.get(sensor_entity) else {
                continue;
            };
            let label = format!("{sensor_entity} hit");

            if let Some(sensor_output) = sensor.output.as_ref() {
                if let Ok(name) = names_query.get(sensor_output.entity) {
                    info_source.label(&label, name.as_str());
                } else {
                    info_source.label(&label, format!("{:?}", sensor_output.entity));
                }
            } else {
                info_source.label(&label, "<Nothing>");
            }
            if let Some(ghost_sensor) = ghost_sensor.as_ref() {
                let text = ghost_sensor
                    .iter()
                    .map(|hit| match names_query.get(hit.entity) {
                        Ok(name) => name.to_string(),
                        Err(_) => format!("{:?}", hit.entity),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                info_source.label(&format!("{sensor_entity} ghost"), text);
            }
        }
        if let Some(obstacle_radar) = obstacle_radar.as_ref() {
            let mut obstacles = obstacle_radar
                .iter_blips()
                .map(|entity| {
                    names_query
                        .get(entity)
                        .ok()
                        .map_or_else(|| format!("{entity}"), ToString::to_string)
                })
                .collect::<Vec<_>>();
            obstacles.sort();
            info_source.label("Obstacle radar", obstacles.join("\n"));
        }
    }
}

//noinspection RsConstantConditionIf
pub fn character_control_radar_visualization_system(
    query: Query<&TnuaObstacleRadar>,
    #[allow(clippy::needless_pass_by_value)]
    spatial_ext: SpatialExtFacade,
    mut gizmos: Gizmos,
) {
    if false {
        // Don't show the gizmos
        return;
    }
    for obstacle_radar in query.iter() {
        let radar_lens = TnuaRadarLens::new(obstacle_radar, &spatial_ext);
        for blip in radar_lens.iter_blips() {
            let closest_point = blip.closest_point().get();
            gizmos.arrow(
                obstacle_radar.tracked_position().f32(),
                closest_point.f32(),
                css::PALE_VIOLETRED,
            );
        }
    }
}
