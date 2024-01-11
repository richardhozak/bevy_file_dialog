use std::marker::PhantomData;
use std::path::PathBuf;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_tasks::{prelude::*, Task};
use bevy_utils::tracing::*;
use futures_lite::future;
use rfd::AsyncFileDialog;

use crate::{FileDialog, FileDialogPlugin};

/// Event that gets sent when directory path gets selected from file system.
#[derive(Event)]
pub struct DialogDirectoryPathPicked<T: PickDirectoryPath> {
    /// Path of picked directory.
    pub path: PathBuf,

    marker: PhantomData<T>,
}

/// Marker trait saying that data can be loaded from file
pub trait PickDirectoryPath: Send + Sync + 'static {}

impl<T> PickDirectoryPath for T where T: Send + Sync + 'static {}

impl FileDialogPlugin {
    /// Allow picking directory paths. This allows you to call
    /// [`FileDialog::pick_directory_path`] and
    /// [`FileDialog::pick_multiple_directory_paths`] on [`Commands`]. For each
    /// `with_pick` you will receive [`DialogDirectoryPathPicked<T>`] in your
    /// systems when picking completes.
    ///
    /// Does not exist in `WASM32`.
    pub fn with_pick<T: PickDirectoryPath>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            app.add_event::<DialogDirectoryPathPicked<T>>();
            app.add_systems(
                First,
                (
                    poll_pick_directory_path_dialog_result::<T>
                        .run_if(resource_exists::<PickDirectoryPathDialog<T>>()),
                    poll_pick_multiple_directory_paths_dialog_result::<T>
                        .run_if(resource_exists::<PickMultipleDirectoryPathsDialog<T>>()),
                ),
            );
        }));
        self
    }
}

fn poll_pick_directory_path_dialog_result<T: PickDirectoryPath>(
    mut commands: Commands,
    mut dialog: ResMut<PickDirectoryPathDialog<T>>,
    mut ev_saved: EventWriter<DialogDirectoryPathPicked<T>>,
) {
    if let Some(result) = future::block_on(future::poll_once(&mut dialog.task)) {
        if let Some(path) = result {
            ev_saved.send(DialogDirectoryPathPicked {
                path,
                marker: PhantomData,
            });
        } else {
            info!("Pick directory dialog closed");
        }

        commands.remove_resource::<PickDirectoryPathDialog<T>>();
    }
}

fn poll_pick_multiple_directory_paths_dialog_result<T: PickDirectoryPath>(
    mut commands: Commands,
    mut dialog: ResMut<PickMultipleDirectoryPathsDialog<T>>,
    mut ev_saved: EventWriter<DialogDirectoryPathPicked<T>>,
) {
    if let Some(result) = future::block_on(future::poll_once(&mut dialog.task)) {
        if let Some(paths) = result {
            ev_saved.send_batch(paths.into_iter().map(|path| DialogDirectoryPathPicked {
                path,
                marker: PhantomData,
            }));
        } else {
            info!("Pick directory dialog closed");
        }

        commands.remove_resource::<PickMultipleDirectoryPathsDialog<T>>();
    }
}

#[derive(Resource)]
struct PickDirectoryPathDialog<T: PickDirectoryPath> {
    task: Task<Option<PathBuf>>,
    marker: PhantomData<T>,
}

#[derive(Resource)]
struct PickMultipleDirectoryPathsDialog<T: PickDirectoryPath> {
    task: Task<Option<Vec<PathBuf>>>,
    marker: PhantomData<T>,
}

impl<'w, 's, 'a> FileDialog<'w, 's, 'a> {
    /// Open pick directory dialog and send [`DialogDirectoryPathPicked<T>`]
    /// event. You can read this event with Bevy's
    /// [`EventReader<DialogDirectoryPathPicked<T>>`].
    ///
    /// Does not exist in `wasm32`.
    pub fn pick_directory_path<T: PickDirectoryPath>(self) {
        self.commands.add(|world: &mut World| {
            let task = AsyncComputeTaskPool::get().spawn(async move {
                let file = AsyncFileDialog::new().pick_folder().await;

                if let Some(file) = file {
                    Some(file.path().to_path_buf())
                } else {
                    None
                }
            });

            let marker = PhantomData::<T>;

            world.remove_resource::<PickDirectoryPathDialog<T>>();
            world.insert_resource(PickDirectoryPathDialog { task, marker });
        });
    }

    /// Open pick multiple directories dialog and send
    /// [`DialogDirectoryPathPicked<T>`] for each selected directory path. You
    /// can get each path by reading every event received with with Bevy's
    /// [`EventReader<DialogDirectoryPathPicked<T>>`].
    ///
    /// Does not exist in `wasm32`.
    pub fn pick_multiple_directory_paths<T: PickDirectoryPath>(self) {
        self.commands.add(|world: &mut World| {
            let task = AsyncComputeTaskPool::get().spawn(async move {
                let files = AsyncFileDialog::new().pick_folders().await;

                if let Some(files) = files {
                    let paths = files
                        .into_iter()
                        .map(|file| file.path().to_path_buf())
                        .collect();

                    Some(paths)
                } else {
                    None
                }
            });

            let marker = PhantomData::<T>;

            world.remove_resource::<PickMultipleDirectoryPathsDialog<T>>();
            world.insert_resource(PickMultipleDirectoryPathsDialog { task, marker });
        });
    }
}
