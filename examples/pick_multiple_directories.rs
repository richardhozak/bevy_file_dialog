//! Example showing how to pick multiple directory paths from file system with a dialog.
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
        .pick_multiple_directory_paths::<PrintDirectoryPath>();
}

fn directory_picked(mut ev_picked: EventReader<DialogDirectoryPicked<PrintDirectoryPath>>) {
    if ev_picked.is_empty() {
        return;
    }

    eprintln!("Picked {} directories", ev_picked.len());
    for ev in ev_picked.read() {
        eprintln!("Directory picked, path {:?}", ev.path);
    }
}
