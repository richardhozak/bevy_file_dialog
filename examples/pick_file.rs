//! This example demonstrates picking of file path.
//!
//! Note that picking a file path does not work on wasm, if you want cross
//! platform solution including wasm, you need to pick+load at once which is
//! provided with `load_file` instead of `pick_file_path`. See the example
//! `load.rs` for that.

use bevy::{log::LogPlugin, prelude::*};
use bevy_file_dialog::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
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
