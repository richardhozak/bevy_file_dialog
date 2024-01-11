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
        // Add the file dialog plugin and specify that we want to save `MyContents`
        .add_plugins(FileDialogPlugin::new().with_save::<MyContents>())
        .add_systems(Startup, save)
        .add_systems(Update, file_saved)
        .run();
}

struct MyContents;

fn save(mut commands: Commands) {
    commands.dialog().save_file::<MyContents>(b"hello".to_vec());
}

fn file_saved(mut ev_saved: EventReader<DialogFileSaved<MyContents>>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => eprintln!("File {} successfully saved", ev.file_name),
            Err(ref err) => eprintln!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}
