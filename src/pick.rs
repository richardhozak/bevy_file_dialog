use std::marker::PhantomData;
use std::path::PathBuf;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_tasks::prelude::*;
use crossbeam_channel::bounded;
use rfd::AsyncFileDialog;

use crate::{
    handle_dialog_result, DialogResult, FileDialog, FileDialogPlugin, StreamReceiver, StreamSender,
};

/// Event that gets sent when directory path gets selected from file system.
#[derive(Event)]
pub struct DialogDirectoryPathPicked<T: PickDirectoryPath> {
    /// Path of picked directory.
    pub path: PathBuf,

    marker: PhantomData<T>,
}

/// Event that gets sent when user closes pick directory dialog without picking any directory.
#[derive(Event)]
pub struct DialogDirectoryPathPickCanceled<T: PickDirectoryPath>(PhantomData<T>);

impl<T: PickDirectoryPath> Default for DialogDirectoryPathPickCanceled<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Marker trait saying that data can be loaded from file
pub trait PickDirectoryPath: Send + Sync + 'static {}

impl<T> PickDirectoryPath for T where T: Send + Sync + 'static {}

impl FileDialogPlugin {
    /// Allow picking directory paths. This allows you to call
    /// [`FileDialog::pick_directory_path`] and
    /// [`FileDialog::pick_multiple_directory_paths`] on [`Commands`]. For each
    /// `with_pick_directory` you will receive [`DialogDirectoryPathPicked<T>`] in your
    /// systems when picking completes.
    ///
    /// Does not exist in `WASM32`.
    pub fn with_pick_directory<T: PickDirectoryPath>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            let (tx, rx) = bounded::<DialogResult<DialogDirectoryPathPicked<T>>>(1);
            app.insert_resource(StreamSender(tx));
            app.insert_resource(StreamReceiver(rx));
            app.add_event::<DialogDirectoryPathPicked<T>>();
            app.add_event::<DialogDirectoryPathPickCanceled<T>>();
            app.add_systems(
                First,
                handle_dialog_result::<
                    DialogDirectoryPathPicked<T>,
                    DialogDirectoryPathPickCanceled<T>,
                >,
            );
        }));
        self
    }
}

impl<'w, 's, 'a> FileDialog<'w, 's, 'a> {
    /// Open pick directory dialog and send [`DialogDirectoryPathPicked<T>`]
    /// event. You can read this event with Bevy's
    /// [`EventReader<DialogDirectoryPathPicked<T>>`].
    ///
    /// Does not exist in `wasm32`.
    pub fn pick_directory_path<T: PickDirectoryPath>(self) {
        self.commands.add(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogDirectoryPathPicked<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_load_file::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let file = AsyncFileDialog::new().pick_folder().await;

                    let Some(file) = file else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let event = DialogDirectoryPathPicked {
                        path: file.path().to_path_buf(),
                        marker: PhantomData,
                    };

                    sender.send(DialogResult::Single(event)).unwrap();
                })
                .detach();
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
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogDirectoryPathPicked<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_load_file::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let files = AsyncFileDialog::new().pick_folders().await;

                    let Some(files) = files else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let events = files
                        .into_iter()
                        .map(|file| DialogDirectoryPathPicked {
                            path: file.path().to_path_buf(),
                            marker: PhantomData,
                        })
                        .collect();

                    sender.send(DialogResult::Batch(events)).unwrap();
                })
                .detach();
        });
    }
}
