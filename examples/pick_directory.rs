//! Example showing how to pick directory path from file system with a dialog.
//!
//! Does not work on wasm.

use bevy::{log::LogPlugin, prelude::*};
use bevy_file_dialog::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        // Add the file dialog plugin and specify that we want to pick
        // directories with `PrintDirectoryPath` marker
        .add_plugins(FileDialogPlugin::new().with_pick_directory::<PrintDirectoryPath>())
        .add_systems(Startup, pick)
        .add_systems(Update, directory_picked)
        .run();
}

struct PrintDirectoryPath;

fn pick(mut commands: Commands) {
    commands
        .dialog()
        .pick_directory_path::<PrintDirectoryPath>();
}

fn directory_picked(mut ev_picked: EventReader<DialogDirectoryPicked<PrintDirectoryPath>>) {
    for ev in ev_picked.read() {
        eprintln!("Directory picked, path {:?}", ev.path);
    }
}
