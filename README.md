# bevy_file_dialogs

[![crates.io](https://img.shields.io/crates/v/bevy_file_dialogs)](https://crates.io/crates/bevy_file_dialogs)
[![docs.rs](https://docs.rs/bevy_file_dialogs/badge.svg)](https://docs.rs/bevy_file_dialogs)

A plugin for loading and saving files using system dialogs for the Bevy game engine.

## Usage
See usage below for loading and saving and the [examples](https://github.com/richardhozak/bevy_file_dialog/tree/main/examples) for separate load/save dialogs.

```rust
use bevy::prelude::*;
use bevy_file_dialog::{FileDialog, FileDialogPlugin, FileLoadedEvent, FileSavedEvent};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the file dialog plugin
        .add_plugins(FileDialogPlugin)
        .add_systems(Update, (dialog, file_loaded, file_saved))
        .run();
}

fn dialog(keys: Res<Input<KeyCode>>, mut dialog: ResMut<FileDialog>) {
    // Ctrl+S - save file
    // Ctrl+O - load file

    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keys.just_pressed(KeyCode::S) {
            // save contents to selected file
            dialog.save_file(b"hello".to_vec());        
        } else if keys.just_pressed(KeyCode::O) {
            // read contents from selected file
            dialog.load_file();
        }
    }
}

fn file_loaded(mut ev_loaded: EventReader<FileLoadedEvent>) {
    for ev in ev_loaded.read() {
        eprintln!("Loaded file {} {:?}", ev.file_name, ev.contents);
    }
}

fn file_saved(mut ev_saved: EventReader<FileSavedEvent>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => eprintln!("File {} successfully saved", ev.file_name),
            Err(ref err) => eprintln!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}
```

| bevy | bevy_file_dialog |
| ---- | ---------------- |
| 0.12 | 0.1.0            |
