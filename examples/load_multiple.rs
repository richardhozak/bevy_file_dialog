use bevy::{log::LogPlugin, prelude::*};
use bevy_file_dialog::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        // Add the file dialog plugin and specify that we want to load `ByteLenContents`
        .add_plugins(FileDialogPlugin::new().with_load_file::<ByteLenContents>())
        .add_systems(Startup, load)
        .add_systems(Update, files_loaded)
        .run();
}

struct ByteLenContents;

fn load(mut commands: Commands) {
    commands.dialog().load_multiple_files::<ByteLenContents>();
}

fn files_loaded(mut ev_loaded: MessageReader<DialogFileLoaded<ByteLenContents>>) {
    if ev_loaded.is_empty() {
        return;
    }

    eprintln!("Loaded {} files at once", ev_loaded.len());
    for ev in ev_loaded.read() {
        eprintln!(
            "Loaded file {} with size of {} bytes",
            ev.file_name,
            ev.contents.len()
        );
    }
}
