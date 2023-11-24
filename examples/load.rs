use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_file_dialog::{FileDialog, FileDialogPlugin, FileLoadedEvent};

fn main() {
    App::new()
        .add_plugins(
            // run the schedule forever, there is no window, so the app would
            // terminate after one loop and we would not get file events
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(0.1))),
        )
        // Add the file dialog plugin
        .add_plugins(FileDialogPlugin)
        .add_systems(Startup, load)
        .add_systems(Update, file_loaded)
        .run();
}

fn load(mut dialog: ResMut<FileDialog>) {
    dialog.load_file();
}

fn file_loaded(mut ev_loaded: EventReader<FileLoadedEvent>) {
    for ev in ev_loaded.read() {
        eprintln!(
            "Loaded file {} with size of {:?} bytes",
            ev.file_name,
            ev.contents.len()
        );
    }
}
