use bevy::prelude::*;
use crate::living::{Health, Team};
use crate::util::units::{BevyVec2Ext, CartesianSpace, Vector2Ext};
use super::components::*;
use super::events::FireWeapon;

pub fn setup_melee_observers(app: &mut App) {
    app.add_observer(|
        // 1. Listen for the FireWeapon event payload
        event: On<FireWeapon>,
        // 2. Fetch combat data specifically for the wielder targeting this event
        mut wielders: Query<(&Weapon, &mut Melee, Option<&mut FireRate>, Option<&Team>)>,
        // 3. Keep your targets query exactly as it was
        targets: Query<(Entity, &Transform, Option<&Team>)>,
        mut commands: Commands,
    | {
        // Extract values from the event payload wrapper
        let wielder = event.event().wielder;
        let origin = event.event().weapon_pos;
        let direction = event.event().aim - origin;

        // Pull the components of the specific combatant who swung the weapon
        let Ok((_, mut melee, mut fire_rate, wielder_team)) = wielders.get_mut(wielder) else {
            return;
        };

        // Cooldown enforcement remains identical
        if !melee.cooldown.is_finished() {
            return;
        }

        melee.cooldown.reset();
        if let Some(ref mut fr) = fire_rate {
            fr.0.reset();
        }

        // Iterate over potential victims in the world
        for (target, target_tf, target_team) in &targets {
            // Self-harm check
            if target == wielder {
                continue;
            }

            // Friendly fire checking rules
            if let (Some(w_team), Some(t_team)) = (wielder_team, target_team) {
                if w_team == t_team {
                    continue;
                }
            }

            // Arc and range math rules remain identical
            let to_target = (target_tf.translation.truncate().to_space::<CartesianSpace>() - origin).normalize_or_zero();


            let dot = to_target.dot(direction);
            let dist = target_tf.translation.truncate().distance(origin.to_bevy());

            if dot > (melee.arc / 2.0).cos() && dist <= melee.reach {
                commands.entity(target).insert(Health(-melee.damage));
            }
        }
    });
}