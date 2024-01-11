# bevy_file_dialog

[![crates.io](https://img.shields.io/crates/v/bevy_file_dialog)](https://crates.io/crates/bevy_file_dialog)
[![docs.rs](https://docs.rs/bevy_file_dialog/badge.svg)](https://docs.rs/bevy_file_dialog)

A plugin for loading and saving files using system dialogs for the Bevy game engine.

## Usage
See usage below for loading and saving and the [examples](https://github.com/richardhozak/bevy_file_dialog/tree/main/examples) for separate load/save dialogs. This example is also present in [examples](https://github.com/richardhozak/bevy_file_dialog/tree/main/examples) and you can run it with `cargo run --example save_and_load`.

```rust
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
        .add_systems(Update, (dialog, file_loaded, file_saved))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn dialog(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    // Ctrl+S - save file
    // Ctrl+O - load file

    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keys.just_pressed(KeyCode::S) {
            // save contents to selected file
            commands
                .dialog()
                .add_filter("Text", &["txt"])
                .save_file::<TextFileContents>(b"hello".to_vec());
        } else if keys.just_pressed(KeyCode::O) {
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

fn file_saved(mut ev_saved: EventReader<DialogFileSaved<TextFileContents>>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => eprintln!("File {} successfully saved", ev.file_name),
            Err(ref err) => eprintln!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}
```

## File dialogs on Linux and BSDs

You can use one of the two backends on linux to create file dialogs that is specified with features, either `gtk3` or `xdg-portal`. By default `bevy_file_dialog` uses the default provided by `rfd` which is `gtk3`. You can change this by specifying the correct features in your `Cargo.toml`:
```
bevy_file_dialog = { version = "*", default-features = false, features = ["xdg-portal"] }
```
More information in [rfd docs](https://docs.rs/rfd/0.12.1/rfd/index.html#linux--bsd-backends), the information there matches `bevy_file_dialog`.

---

| bevy | bevy_file_dialog |
| ---- | ---------------- |
| 0.12 | 0.1.0 - 0.2.0    |
