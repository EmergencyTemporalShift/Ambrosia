use bevy::prelude::*;
use crate::living::{Health, Team};
use crate::util::units::{BevyVec2Ext, CartesianSpace, Vector2Ext};
use super::components::{Weapon, Melee, Cooldown};
use super::events::FireWeapon;

pub fn setup_melee_observers(app: &mut App) {
    app.add_observer(|
        event: On<FireWeapon>,
        //mut commands: Commands,
        wielder_q: Query<(&Transform, &Team)>,
        mut weapon_q: Query<(&Weapon, &Melee, &mut Cooldown)>,
        mut targets_q: Query<(Entity, &Transform, &Team, &mut Health)>,
    | {
        // Extract values from the event payload wrapper
        let wielder = event.event().wielder;
        let intent = event.event().intent;

        // 1. Extract the data, or quietly do nothing if the player released the button
        let Some((weapon_pos, aim)) = intent.spatial_data() else {
            return;
        };

        // 2. We must calculate the normalized direction vector to do the arc/dot-product math
        let direction = (aim - weapon_pos).normalize_or_zero();

        // 3. Pull the components of the specific combatant who swung the weapon
        let Ok((_transform, team)) = wielder_q.get(wielder) else {
            return;
        };

        // 4. Pull the weapon components (THIS WAS MISSING!)
        let Ok((weapon, melee, mut cooldown)) = weapon_q.get_mut(event.event().weapon) else {
            return;
        };

        // 5. Cooldown enforcement
        if !cooldown.cooldown.is_finished() {
            return;
        }
        cooldown.cooldown.reset();

        // 6. Iterate over potential victims in the world
        for (target_ent, target_tf, target_team, mut target_health) in &mut targets_q {
            // Self-harm & friendly fire checks
            if target_ent == wielder || !team.is_hostile_to(target_team) {
                continue;
            }

            // Math rules using our extracted 'weapon_pos' and 'direction'
            let to_target = (target_tf.translation.truncate().to_space::<CartesianSpace>() - weapon_pos).normalize_or_zero();

            let dot = to_target.dot(direction);
            let dist = target_tf.translation.truncate().distance(weapon_pos.to_bevy());

            if dot > (melee.arc / 2.0).cos() && dist <= melee.reach {
                target_health.0 -= weapon.damage; // assuming Health wraps a float
            }
        }
    });
}