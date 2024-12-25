//! This example demonstrates picking of a file path when using
//! [`WinitSettings::desktop_app`] or similar settings.

use std::time::Duration;

use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};
use bevy_file_dialog::{DialogFilePicked, FileDialogExt, FileDialogPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_secs(3600)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(3600)),
        })
        // Add the file dialog plugin and specify that we want to pick
        // directories with `PrintFilePath` marker
        .add_plugins(FileDialogPlugin::new().with_pick_file::<PrintFilePath>())
        .add_systems(Startup, pick)
        .add_systems(Update, file_picked)
        .run();
}

struct PrintFilePath;

fn pick(mut commands: Commands) {
    commands.dialog().pick_file_path::<PrintFilePath>();
}

fn file_picked(mut ev_picked: EventReader<DialogFilePicked<PrintFilePath>>) {
    for ev in ev_picked.read() {
        eprintln!("File picked, path {:?}", ev.path);
    }
}
