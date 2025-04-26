use bevy::prelude::*;
use bevy_file_dialog::prelude::*;

struct TextFileContents;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the file dialog plugin
        .add_plugins(
            FileDialogPlugin::new()
                // allow saving of files marked with TextFileContents
                .with_save_file::<TextFileContents>()
                // allow loading of files marked with TextFileContents
                .with_load_file::<TextFileContents>(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                dialog,
                file_loaded,
                file_saved,
                file_load_canceled,
                file_save_canceled,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn dialog(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    // Ctrl+S - save file
    // Ctrl+O - load file

    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keys.just_pressed(KeyCode::KeyS) {
            // save contents to selected file
            commands
                .dialog()
                .add_filter("Text", &["txt"])
                .set_file_name("hello.txt")
                .save_file::<TextFileContents>(b"hello".to_vec());
        } else if keys.just_pressed(KeyCode::KeyO) {
            // read contents from selected file
            commands
                .dialog()
                .add_filter("Text", &["txt"])
                .load_file::<TextFileContents>();
        }
    }
}

fn file_loaded(mut ev_loaded: EventReader<DialogFileLoaded<TextFileContents>>) {
    for ev in ev_loaded.read() {
        eprintln!(
            "Loaded file {} with contents '{}'",
            ev.file_name,
            std::str::from_utf8(&ev.contents).unwrap()
        );
    }
}

fn file_load_canceled(mut ev_canceled: EventReader<DialogFileLoadCanceled<TextFileContents>>) {
    for _ in ev_canceled.read() {
        eprintln!("Text file content load canceled");
    }
}

fn file_saved(mut ev_saved: EventReader<DialogFileSaved<TextFileContents>>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => eprintln!("File {} successfully saved", ev.file_name),
            Err(ref err) => eprintln!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}

fn file_save_canceled(mut ev_canceled: EventReader<DialogFileSaveCanceled<TextFileContents>>) {
    for _ in ev_canceled.read() {
        eprintln!("Text file content save canceled");
    }
}
