//! This example demonstrates observing and handling entity-scoped file dialog events.
//!
//! Expected behaviors:
//! When user picks a file it prints the operation, some name for the observed entity, and then moves on to present another dialogue.
//! When user closes the dialog, it exits the app.

use bevy::{log::LogPlugin, prelude::*};
use bevy_app::AppExit;
use bevy_file_dialog::{prelude::*, EntityFileDialogExt, EntityScopedDialogEvent};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(FileDialogPlugin::new())
        .add_systems(Startup, load)
        .run();
}

fn load(mut commands: Commands) {
    commands
        .spawn(Name::new("Watched Load Entity"))
        .observe(watch_load)
        .with_dialog()
        .set_title("File Pick Dialog")
        .load_file();
}
fn watch_load(
    source: On<EntityScopedDialogEvent>,
    name_of: Query<&Name>,
    mut ev_exit: MessageWriter<AppExit>,
    mut commands: Commands,
) {
    use bevy_file_dialog::EntityScopedDialogResult::*;
    let Ok(name) = name_of.get(source.entity) else {
        panic!("Failed to locate entity under test");
    };
    match source.result.clone() {
        Load(file_loaded) => {
            eprintln!(
                "Observed load selection {} under {}",
                file_loaded.file_name, name
            );
            eprintln!("Moving to load_multiple test");
            commands
                .spawn(Name::new("Watched Load Multiple Entity"))
                .observe(watch_load_multiple)
                .with_dialog()
                .set_title("M Pick Dialog")
                .load_multiple_files();
        }
        Canceled => {
            eprintln!("Canceled dialog for printing file name, exiting app");
            ev_exit.write_default();
        }
        _ => panic!("Encountered unexpected variant under test"),
    };
}
fn watch_load_multiple(
    source: On<EntityScopedDialogEvent>,
    name_of: Query<&Name>,
    mut count: Local<usize>,
    mut ev_exit: MessageWriter<AppExit>,
    mut commands: Commands,
) {
    use bevy_file_dialog::EntityScopedDialogResult::*;
    let Ok(name) = name_of.get(source.entity) else {
        panic!("Failed to locate entity under test");
    };
    match source.result.clone() {
        Load(file_loaded) => {
            *count = *count + 1;
            eprintln!(
                "Observed count {} multi load selection {} under {}",
                *count, file_loaded.file_name, name
            );
            // As this is observing a call to `load_multiple_files`, we may call this function several times for one operation,
            // but we of course do not wish to open several subsequent dialogues.
            //
            if *count != 1 {
                return;
            }
            eprintln!("Moving to load_multiple test");

            commands
                .spawn(Name::new("Watched Pick Directory Entity"))
                .observe(watch_pick_directory)
                .with_dialog()
                .set_title("Directory Pick Dialog")
                .pick_directory_path();
        }
        Canceled => {
            eprintln!("Canceled dialog for printing file name, exiting app");
            ev_exit.write_default();
        }
        _ => panic!("Encountered unexpected variant under test"),
    };
}
fn watch_pick_directory(
    source: On<EntityScopedDialogEvent>,
    name_of: Query<&Name>,
    mut ev_exit: MessageWriter<AppExit>,
) {
    use bevy_file_dialog::EntityScopedDialogResult::*;
    let Ok(name) = name_of.get(source.entity) else {
        panic!("Failed to locate entity under test");
    };
    match source.result.clone() {
        Pick(path_picked) => {
            eprintln!(
                "Observed pick directory selection {} under {}",
                path_picked.path.to_string_lossy(),
                name
            );

            eprintln!("Reached the end of our test cases, exiting app");
            ev_exit.write_default();
        }
        Canceled => {
            eprintln!("Canceled dialog for printing file name, exiting app");
            ev_exit.write_default();
        }
        _ => panic!("Encountered unexpected variant under test"),
    };
}
