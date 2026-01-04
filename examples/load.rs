use bevy::{log::LogPlugin, prelude::*};
use bevy_file_dialog::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        // Add the file dialog plugin and specify that we want to load `ByteLenContents`
        .add_plugins(FileDialogPlugin::new().with_load_file::<ByteLenContents>())
        .add_systems(Startup, load)
        .add_systems(Update, file_loaded)
        .run();
}

struct ByteLenContents;

fn load(mut commands: Commands) {
    commands.dialog().load_file(ByteLenContents);
}

fn file_loaded(mut ev_loaded: MessageReader<DialogFileLoaded<ByteLenContents>>) {
    for ev in ev_loaded.read() {
        eprintln!(
            "Loaded file {} with size of {} bytes",
            ev.file_name,
            ev.contents.len()
        );
    }
}
