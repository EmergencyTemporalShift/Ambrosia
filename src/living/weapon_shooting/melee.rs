use bevy::prelude::*;
use crate::living::{Health, Team};
use crate::util::units::{BevyVec2Ext, CartesianSpace, Vector2Ext};
use super::components::*;
use super::events::FireWeapon;

pub fn setup_melee_observers(app: &mut App) {
    app.add_observer(|
        event: On<FireWeapon>,
        //mut commands: Commands,
        wielder_q: Query<(&Transform, &Team)>,
        mut weapon_q: Query<(&Weapon, &Melee, &mut Cooldown,)>,
        mut targets: Query<(Entity, &Transform, &Team, &mut Health)>,
    | {
        // Extract values from the event payload wrapper
        let wielder = event.event().wielder;
        let origin = event.event().weapon_pos;
        let direction = event.event().aim - origin;

        // Pull the components of the specific combatant who swung the weapon
        let Ok((_transform, team)) = wielder_q.get(wielder) else {
            return;
        };

        let Ok((weapon, melee, mut cooldown)) = weapon_q.get_mut(event.event().weapon) else {
            return;
        };

        // Cooldown enforcement remains identical
        if !cooldown.cooldown.is_finished() {
            return;
        }
        cooldown.cooldown.reset();

        // Iterate over potential victims in the world
        for (target_ent, target_tf, target_team, mut target_health) in &mut targets {
            // Self-harm check
            if target_ent == wielder {
                continue;
            }
            if !team.is_hostile_to(target_team) {
                continue; // Skip teammates/neutrals
            }

            // Arc and range math rules remain identical
            let to_target = (target_tf.translation.truncate().to_space::<CartesianSpace>() - origin).normalize_or_zero();


            let dot = to_target.dot(direction);
            let dist = target_tf.translation.truncate().distance(origin.to_bevy());

            if dot > (melee.arc / 2.0).cos() && dist <= melee.reach {
                target_health.0 -= weapon.damage; // assuming Health wraps a float
            }
        }
    });
}