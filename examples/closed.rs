//! This example demonstrates receving event when user does not pick any file
//! and instead chooses to close the dialog.
//!
//! Try to run this example multiple times, choosing files one time and closing
//! the dialog second time.

use bevy::{log::LogPlugin, prelude::*};
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

fn dialog_canceled(mut ev_canceled: EventReader<DialogFileLoadCanceled<PrintFileName>>) {
    for _ in ev_canceled.read() {
        eprintln!("Canceled dialog for printing file name");
    }
}

fn file_selected(mut ev_loaded: EventReader<DialogFileLoaded<PrintFileName>>) {
    for ev in ev_loaded.read() {
        eprintln!("Selected {}", ev.file_name);
    }
}
