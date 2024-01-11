use bevy::{log::LogPlugin, prelude::*};
use bevy_file_dialog::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        // Add the file dialog plugin and specify that we want to save `ByteContents`
        .add_plugins(FileDialogPlugin::new().with_save::<ByteContents>())
        .add_systems(Startup, save)
        .add_systems(Update, file_saved)
        .run();
}

struct ByteContents;

fn save(mut commands: Commands) {
    commands
        .dialog()
        .save_file::<ByteContents>(b"hello".to_vec());
}

fn file_saved(mut ev_saved: EventReader<DialogFileSaved<ByteContents>>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => eprintln!("File {} successfully saved", ev.file_name),
            Err(ref err) => eprintln!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}
