use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use bevy_file_dialog::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            // run the schedule forever, there is no window, so the app would
            // terminate after one loop and we would not get file events
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(0.1))),
        )
        .add_plugins(LogPlugin::default())
        // Add the file dialog plugin and specify that we want to load `MyContents`
        .add_plugins(FileDialogPlugin::new().with_load::<MyContents>())
        .add_systems(Startup, load)
        .add_systems(Update, file_loaded)
        .run();
}

struct MyContents;

fn load(mut commands: Commands) {
    commands.dialog().load_file::<MyContents>();
}

fn file_loaded(mut ev_loaded: EventReader<DialogFileLoaded<MyContents>>) {
    for ev in ev_loaded.read() {
        eprintln!(
            "Loaded file {} with size of {} bytes",
            ev.file_name,
            ev.contents.len()
        );
    }
}
