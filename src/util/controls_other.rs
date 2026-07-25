use bevy::{app::AppExit, prelude::*};
use leafwing_input_manager::prelude::*;
use crate::living::player::IsPlayer;
use crate::util::game_states::GameState;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum SystemAction {
    ToggleEgui,
    Exit,
    TogglePause,
}

/// State and programmatic controls for non-player actions.
#[derive(Resource, Debug, Clone, Copy)]
pub struct OtherControls {
    egui_visible: bool,
}

impl Default for OtherControls {
    fn default() -> Self {
        Self {
            egui_visible: true,
        }
    }
}

impl OtherControls {
    #[cfg(feature = "egui")]
    pub fn is_egui_visible(&self) -> bool {
        self.egui_visible
    }
    #[cfg(not(feature = "egui"))]
    pub fn is_egui_visible(&self) -> bool {
        false
    }

    #[allow(dead_code)]
    pub fn show_egui(&mut self) {
        self.egui_visible = true;
    }
    #[allow(dead_code)]
    pub fn hide_egui(&mut self) {
        self.egui_visible = false;
    }

    pub fn toggle_egui(&mut self) {
        self.egui_visible = !self.egui_visible;
    }

    pub fn exit(exit_events: &mut MessageWriter<AppExit>) {
        exit_events.write(AppExit::Success);
    }
}

pub struct OtherControlsPlugin;

impl Plugin for OtherControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OtherControls>()
            .add_plugins(InputManagerPlugin::<SystemAction>::default())
            .add_systems(Startup, setup_system_controls)
            .add_systems(Update, handle_non_player_controls);
    }
}

// Spawns a dedicated entity to hold our global system inputs
fn setup_system_controls(mut commands: Commands) {
    commands.spawn((
        InputMap::new([
            (SystemAction::ToggleEgui, KeyCode::F1),
            (SystemAction::Exit, KeyCode::Escape),
            (SystemAction::TogglePause, KeyCode::KeyP), // Temporary, we'll get a menu eventually.
        ]),
        ActionState::<SystemAction>::default(),
    ));
}

pub fn handle_non_player_controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    action_state_q: Query<&ActionState<SystemAction>>,
    mut controls: ResMut<OtherControls>,
    mut exit_events: MessageWriter<AppExit>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    // 1. Debug key press (KeyP) to dump player entity hierarchy
    if keys.just_pressed(KeyCode::KeyO) {
        commands.queue(|world: &mut World| {
            let mut player_q = world.query_filtered::<Entity, With<IsPlayer>>();
            let Ok(player_entity) = player_q.single(world) else {
                info!("Debug Dump: No entity with IsPlayer found.");
                return;
            };

            info!("================ PLAYER HIERARCHY DUMP ================");
            dump_entity_tree(world, player_entity, 0);
            info!("=======================================================");
        });
    }

    // 2. ActionState system actions
    let Ok(action_state) = action_state_q.single() else {
        return;
    };

    if action_state.just_pressed(&SystemAction::ToggleEgui) {
        controls.toggle_egui();
    }

    if action_state.just_pressed(&SystemAction::Exit) {
        OtherControls::exit(&mut exit_events);
    }

    if action_state.just_pressed(&SystemAction::TogglePause) {
        match current_state.get() {
            GameState::Running => {
                next_state.set(GameState::Paused);
                time.pause();
            }
            GameState::Paused => {
                next_state.set(GameState::Running);
                time.unpause();
            }
        }
    }
}

fn dump_entity_tree(world: &World, entity: Entity, depth: usize) {
    let Some(entity_ref) = world.get_entity(entity).ok() else { return };

    let entity_label = entity_ref
        .get::<Name>()
        .map(|n| n.as_str())
        .unwrap_or("Unnamed Entity");

    let mut comp_names: Vec<String> = entity_ref
        .archetype()
        .components()
        .into_iter()
        .filter_map(|id| world.components().get_name(*id))
        .map(|debug_name| {
            let full_path = &*debug_name;
            full_path
                .split("::")
                .last()
                .unwrap_or(full_path)
                .to_string()
        })
        .collect();

    comp_names.sort();

    let indent = "  ".repeat(depth);
    info!("{indent}▶ {entity_label} ({entity:?})");
    info!("{indent}   Components: [{}]", comp_names.join(", "));

    if let Some(children) = entity_ref.get::<Children>() {
        for child in children.iter() {
            dump_entity_tree(world, child, depth + 1);
        }
    }
}