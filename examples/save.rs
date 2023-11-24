use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_file_dialog::{FileDialog, FileDialogPlugin, FileSavedEvent};

fn main() {
    App::new()
        .add_plugins(
            // run the schedule forever, there is no window, so the app would
            // terminate after one loop and we would not get file events
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(0.1))),
        )
        // Add the file dialog plugin
        .add_plugins(FileDialogPlugin)
        .add_systems(Startup, save)
        .add_systems(Update, file_saved)
        .run();
}

fn save(mut dialog: ResMut<FileDialog>) {
    dialog.save_file(b"hello".to_vec());
}

fn file_saved(mut ev_saved: EventReader<FileSavedEvent>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => eprintln!("File {} successfully saved", ev.file_name),
            Err(ref err) => eprintln!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}
