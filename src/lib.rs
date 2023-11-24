use std::io;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use rfd::{AsyncFileDialog, FileHandle};

pub struct FileDialogPlugin;

impl Plugin for FileDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileDialog>();
        app.add_event::<FileLoadedEvent>();
        app.add_event::<FileSavedEvent>();
        app.add_systems(PreUpdate, poll_dialog_result);
    }
}

#[derive(Event)]
pub struct FileLoadedEvent {
    pub file_name: String,
    pub contents: Vec<u8>,
}

#[derive(Event)]
pub struct FileSavedEvent {
    pub file_name: String,
    pub result: io::Result<()>,
}

#[derive(Resource, Default)]
pub struct FileDialog {
    state: Option<FileDialogState>,
}

impl FileDialog {
    pub fn save_file(&mut self, contents: Vec<u8>) {
        if self.state.is_some() {
            panic!("Cannot save more than one file at once");
        }

        let task = AsyncComputeTaskPool::get().spawn(AsyncFileDialog::new().save_file());
        self.state = Some(FileDialogState::Opening(task, DialogKind::Save(contents)));
    }

    pub fn load_file(&mut self) {
        if self.state.is_some() {
            panic!("Cannot save more than one file at once");
        }

        let task = AsyncComputeTaskPool::get().spawn(AsyncFileDialog::new().pick_file());
        self.state = Some(FileDialogState::Opening(task, DialogKind::Load));
    }
}

enum DialogKind {
    Save(Vec<u8>),
    Load,
}

enum FileDialogState {
    Opening(Task<Option<FileHandle>>, DialogKind),
    Saving(Task<io::Result<()>>, String),
    Loading(Task<Vec<u8>>, String),
}

fn poll_dialog_result(
    mut dialog: ResMut<FileDialog>,
    mut ev_saved: EventWriter<FileSavedEvent>,
    mut ev_loaded: EventWriter<FileLoadedEvent>,
) {
    dialog.state = match dialog.state.take() {
        Some(state) => match state {
            FileDialogState::Opening(mut task, kind) => {
                if let Some(result) = future::block_on(future::poll_once(&mut task)) {
                    match result {
                        Some(file_handle) => match kind {
                            DialogKind::Save(contents) => {
                                let file_name = file_handle.file_name();
                                Some(FileDialogState::Saving(
                                    AsyncComputeTaskPool::get()
                                        .spawn(async move { file_handle.write(&contents).await }),
                                    file_name,
                                ))
                            }
                            DialogKind::Load => {
                                let file_name = file_handle.file_name();
                                Some(FileDialogState::Loading(
                                    AsyncComputeTaskPool::get()
                                        .spawn(async move { file_handle.read().await }),
                                    file_name,
                                ))
                            }
                        },
                        None => {
                            // user closed the dialog
                            None
                        }
                    }
                } else {
                    Some(FileDialogState::Opening(task, kind))
                }
            }
            FileDialogState::Saving(mut task, file_name) => {
                if let Some(result) = future::block_on(future::poll_once(&mut task)) {
                    ev_saved.send(FileSavedEvent { file_name, result });
                    None
                } else {
                    Some(FileDialogState::Saving(task, file_name))
                }
            }
            FileDialogState::Loading(mut task, file_name) => {
                if let Some(contents) = future::block_on(future::poll_once(&mut task)) {
                    ev_loaded.send(FileLoadedEvent {
                        file_name,
                        contents,
                    });
                    None
                } else {
                    Some(FileDialogState::Loading(task, file_name))
                }
            }
        },
        None => None,
    };
}
