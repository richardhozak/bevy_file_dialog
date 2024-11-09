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
pub struct DialogDirectoryPicked<T: PickDirectoryPath> {
    /// Path of picked directory.
    pub path: PathBuf,

    marker: PhantomData<T>,
}

/// Event that gets sent when user closes pick directory dialog without picking any directory.
#[derive(Event)]
pub struct DialogDirectoryPickCanceled<T: PickDirectoryPath>(PhantomData<T>);

impl<T: PickDirectoryPath> Default for DialogDirectoryPickCanceled<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Marker trait saying what directory path are we picking.
pub trait PickDirectoryPath: Send + Sync + 'static {}

impl<T> PickDirectoryPath for T where T: Send + Sync + 'static {}

/// Event that gets sent when file path gets selected from file system.
#[derive(Event)]
pub struct DialogFilePicked<T: PickFilePath> {
    /// Path of picked file.
    pub path: PathBuf,

    marker: PhantomData<T>,
}

/// Event that gets sent when user closes pick file dialog without picking any file.
#[derive(Event)]
pub struct DialogFilePickCanceled<T: PickFilePath>(PhantomData<T>);

impl<T: PickFilePath> Default for DialogFilePickCanceled<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Marker trait saying what file path are we picking.
pub trait PickFilePath: Send + Sync + 'static {}

impl<T> PickFilePath for T where T: Send + Sync + 'static {}

impl FileDialogPlugin {
    /// Allow picking directory paths. This allows you to call
    /// [`FileDialog::pick_directory_path`] and
    /// [`FileDialog::pick_multiple_directory_paths`] on [`Commands`]. For each
    /// `with_pick_directory` you will receive [`DialogDirectoryPicked<T>`] in your
    /// systems when picking completes.
    ///
    /// Does not exist in `WASM32`.
    pub fn with_pick_directory<T: PickDirectoryPath>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            let (tx, rx) = bounded::<DialogResult<DialogDirectoryPicked<T>>>(1);
            app.insert_resource(StreamSender(tx));
            app.insert_resource(StreamReceiver(rx));
            app.add_event::<DialogDirectoryPicked<T>>();
            app.add_event::<DialogDirectoryPickCanceled<T>>();
            app.add_systems(
                First,
                handle_dialog_result::<DialogDirectoryPicked<T>, DialogDirectoryPickCanceled<T>>,
            );
        }));
        self
    }

    /// Allow picking file paths. This allows you to call
    /// [`FileDialog::pick_file_path`] and
    /// [`FileDialog::pick_multiple_file_paths`] on [`Commands`]. For each
    /// `with_pick_file` you will receive [`DialogFilePicked<T>`] in your
    /// systems when picking completes.
    ///
    /// Does not exist in `WASM32`. If you want cross-platform solution for
    /// files, you need to use [`FileDialogPlugin::with_load_file`], which
    /// allows picking and loading in one step which is compatible with wasm.
    pub fn with_pick_file<T: PickFilePath>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            let (tx, rx) = bounded::<DialogResult<DialogFilePicked<T>>>(1);
            app.insert_resource(StreamSender(tx));
            app.insert_resource(StreamReceiver(rx));
            app.add_event::<DialogFilePicked<T>>();
            app.add_event::<DialogFilePickCanceled<T>>();
            app.add_systems(
                First,
                handle_dialog_result::<DialogFilePicked<T>, DialogFilePickCanceled<T>>,
            );
        }));
        self
    }
}

impl<'w, 's, 'a> FileDialog<'w, 's, 'a> {
    /// Open pick directory dialog and send [`DialogDirectoryPicked<T>`]
    /// event. You can read this event with Bevy's
    /// [`EventReader<DialogDirectoryPicked<T>>`].
    ///
    /// Does not exist in `wasm32`.
    pub fn pick_directory_path<T: PickDirectoryPath>(self) {
        self.commands.queue(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogDirectoryPicked<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_pick_directory::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let file = self.dialog.pick_folder().await;

                    let Some(file) = file else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let event = DialogDirectoryPicked {
                        path: file.path().to_path_buf(),
                        marker: PhantomData,
                    };

                    sender.send(DialogResult::Single(event)).unwrap();
                })
                .detach();
        });
    }

    /// Open pick multiple directories dialog and send
    /// [`DialogDirectoryPicked<T>`] for each selected directory path. You
    /// can get each path by reading every event received with with Bevy's
    /// [`EventReader<DialogDirectoryPicked<T>>`].
    ///
    /// Does not exist in `wasm32`.
    pub fn pick_multiple_directory_paths<T: PickDirectoryPath>(self) {
        self.commands.queue(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogDirectoryPicked<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_pick_directory::<T>()'")
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
                        .map(|file| DialogDirectoryPicked {
                            path: file.path().to_path_buf(),
                            marker: PhantomData,
                        })
                        .collect();

                    sender.send(DialogResult::Batch(events)).unwrap();
                })
                .detach();
        });
    }

    /// Open pick file dialog and send [`DialogFilePicked<T>`]
    /// event. You can read this event with Bevy's
    /// [`EventReader<DialogFilePicked<T>>`].
    ///
    /// Does not exist in `wasm32`. If you want cross-platform solution, you
    /// need to use [`FileDialog::load_file`], which does picking and loading in
    /// one step which is compatible with wasm.
    pub fn pick_file_path<T: PickFilePath>(self) {
        self.commands.queue(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogFilePicked<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_pick_file::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let file = self.dialog.pick_file().await;

                    let Some(file) = file else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let event = DialogFilePicked {
                        path: file.path().to_path_buf(),
                        marker: PhantomData,
                    };

                    sender.send(DialogResult::Single(event)).unwrap();
                })
                .detach();
        });
    }

    /// Open pick multiple files dialog and send
    /// [`DialogFilePicked<T>`] for each selected file path. You
    /// can get each path by reading every event received with with Bevy's
    /// [`EventReader<DialogFilePicked<T>>`].
    ///
    /// Does not exist in `wasm32`. If you want cross-platform solution, you
    /// need to use [`FileDialog::load_multiple_files`], which does picking and
    /// loading in one step which is compatible with wasm.
    pub fn pick_multiple_file_paths<T: PickDirectoryPath>(self) {
        self.commands.queue(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogFilePicked<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_pick_file::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let files = AsyncFileDialog::new().pick_files().await;

                    let Some(files) = files else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let events = files
                        .into_iter()
                        .map(|file| DialogFilePicked {
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
