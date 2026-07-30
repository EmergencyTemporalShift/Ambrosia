use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::log::LogPlugin;


fn main() {
    let default_plugins = DefaultPlugins.set(ImagePlugin::default_nearest()).disable::<LogPlugin>();

    #[cfg(feature = "pie")]
    {
        default_plugins = jackdaw_runtime::maybe_windowless(default_plugins);
    }

    let default_plugins = default_plugins.build();

    let mut app = App::new();
    app.add_plugins(default_plugins)
        // All gameplay lives in the library crate so the editor can link it too.
        .add_plugins(ambrosia::GamePlugin);
    // .edit_schedule(Update, |schedule| {
    //     schedule.set_build_settings(ScheduleBuildSettings {
    //         ambiguity_detection: LogLevel::Warn,
    //         ..default()
    //     });
    // })

    // Pass default settings as the third argument
    // let dot_output = bevy_mod_debugdump::schedule_graph_dot(
    //     &mut app,
    //     Update,
    //     &bevy_mod_debugdump::schedule_graph::Settings::default()
    // );

    // std::fs::write("schedule.dot", dot_output).expect("Failed to write schedule data");


    app.run();
}