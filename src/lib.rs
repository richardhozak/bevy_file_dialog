use std::io;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use rfd::AsyncFileDialog;

/// Add this plugin to Bevy App to use the `FileDialog` resource in your system
/// to save/load files.
pub struct FileDialogPlugin;

impl Plugin for FileDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileDialog>();
        app.add_event::<FileLoadedEvent>();
        app.add_event::<FileSavedEvent>();
        app.add_systems(PreUpdate, poll_dialog_result);
    }
}

/// Event that gets sent when file contents get loaded from file system.
#[derive(Event)]
pub struct FileLoadedEvent {
    /// Name of loaded file.
    pub file_name: String,

    /// Byte contents of loaded file.
    pub contents: Vec<u8>,
}

/// Event that gets sent when file contents get saved to file system.
#[derive(Event)]
pub struct FileSavedEvent {
    /// Name of saved file.
    pub file_name: String,

    /// Result of save file system operation.
    pub result: io::Result<()>,
}

/// Resource for creating dialogs.
#[derive(Resource, Default)]
pub struct FileDialog {
    state: Option<FileDialogState>,
}

impl FileDialog {
    /// Open save file dialog and save the `contents` to that file. When file
    /// gets saved, the [`FileSavedEvent`] gets sent. You can get read this event
    /// with Bevy's [`EventReader<FileSavedEvent>`] system param.
    pub fn save_file(&mut self, contents: Vec<u8>) {
        if self.state.is_some() {
            panic!("Cannot save more than one file at once");
        }

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let file = AsyncFileDialog::new().save_file().await;

            if let Some(file) = file {
                Some((file.file_name(), file.write(&contents).await))
            } else {
                None
            }
        });

        self.state = Some(FileDialogState::Saving(task));
    }

    /// Open pick file dialog and load its contents. When file contents get
    /// loaded, the [`FileLoadedEvent`] gets sent. You can read this event with
    /// Bevy's [`EventReader<FileLoadedEvent>`].
    pub fn load_file(&mut self) {
        if self.state.is_some() {
            panic!("Cannot save more than one file at once");
        }

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let file = AsyncFileDialog::new().pick_file().await;

            if let Some(file) = file {
                Some((file.file_name(), file.read().await))
            } else {
                None
            }
        });

        self.state = Some(FileDialogState::Loading(task));
    }
}

enum FileDialogState {
    Saving(Task<Option<(String, io::Result<()>)>>),
    Loading(Task<Option<(String, Vec<u8>)>>),
}

fn poll_dialog_result(
    mut dialog: ResMut<FileDialog>,
    mut ev_saved: EventWriter<FileSavedEvent>,
    mut ev_loaded: EventWriter<FileLoadedEvent>,
) {
    dialog.state = match dialog.state.take() {
        Some(state) => match state {
            FileDialogState::Saving(mut task) => {
                if let Some(result) = future::block_on(future::poll_once(&mut task)) {
                    if let Some((file_name, result)) = result {
                        ev_saved.send(FileSavedEvent { file_name, result });
                        None
                    } else {
                        info!("Save dialog closed");
                        None
                    }
                } else {
                    Some(FileDialogState::Saving(task))
                }
            }
            FileDialogState::Loading(mut task) => {
                if let Some(result) = future::block_on(future::poll_once(&mut task)) {
                    if let Some((file_name, contents)) = result {
                        ev_loaded.send(FileLoadedEvent {
                            file_name,
                            contents,
                        });
                        None
                    } else {
                        info!("Load dialog closed");
                        None
                    }
                } else {
                    Some(FileDialogState::Loading(task))
                }
            }
        },
        None => None,
    };
}
