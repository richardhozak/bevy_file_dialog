//! This example demonstrates receving event when user does not pick any file
//! and instead chooses to close the dialog.
//!
//! When user picks a file it prints the file name and asks for another file.
//! When user closes the dialog, it exits the app.

use bevy::{log::LogPlugin, prelude::*};
use bevy_app::AppExit;
use bevy_file_dialog::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        // Add the file dialog plugin and specify that we want to load `PrintFileName`
        .add_plugins(FileDialogPlugin::new().with_load_file::<PrintFileName>())
        .add_systems(Startup, load)
        .add_systems(Update, (file_selected, dialog_canceled))
        .run();
}

struct PrintFileName;

fn load(mut commands: Commands) {
    commands.dialog().load_file::<PrintFileName>();
}

fn dialog_canceled(
    mut ev_canceled: EventReader<DialogFileLoadCanceled<PrintFileName>>,
    mut ev_exit: EventWriter<AppExit>,
) {
    for _ in ev_canceled.read() {
        eprintln!("Canceled dialog for printing file name, exiting app");
        ev_exit.write_default();
    }
}

fn file_selected(
    mut commands: Commands,
    mut ev_loaded: EventReader<DialogFileLoaded<PrintFileName>>,
) {
    for ev in ev_loaded.read() {
        eprintln!("Selected {}", ev.file_name);
        eprintln!("Select another or close the dialog...");
        commands.dialog().load_file::<PrintFileName>();
    }
}
