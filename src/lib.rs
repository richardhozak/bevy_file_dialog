#![warn(missing_docs)]

//! Bevy plugin that allows you to save and load files with file dialogs.
//!
//! In order to use it you need to add [`FileDialogPlugin`] to your [`App`] with
//! at least one or more calls to:
//! - [`FileDialogPlugin::with_save_file::<T>`]
//! - [`FileDialogPlugin::with_load_file::<T>`]
//! - [`FileDialogPlugin::with_pick_directory::<T>`]
//!
//! these functions can be called as many times as you want, the type parameter
//! acts as marker that allows you to call:
//! - [`FileDialog::save_file`]
//!   - for [`FileDialogPlugin::with_save_file::<T>`]
//! - [`FileDialog::load_file`]
//! - [`FileDialog::load_multiple_files`]
//!   - for [`FileDialogPlugin::with_load_file::<T>`]
//! - [`FileDialog::pick_directory_path`]
//! - [`FileDialog::pick_multiple_directory_paths`]
//!   - for [`FileDialogPlugin::with_pick_directory::<T>`]
//!
//! with same type marker and then receive the result in
//! - [`DialogFileSaved`] ([`EventReader<DialogFileSaved<T>>`])
//! - [`DialogFileLoaded`] ([`EventReader<DialogFileLoaded<T>>`])
//! - [`pick::DialogDirectoryPicked`] ([`EventReader<pick::DialogDirectoryPicked<T>>`])
//!
//! events
//!
//! [`FileDialog`] can be created by calling [`FileDialogExt::dialog`],
//! [`FileDialogExt`] as an extension trait implemented for [`Commands`]
//! and is included in `bevy_file_dialog::prelude`:
//!
//! ```rust
//! fn system(mut commands: Commands) {
//!     commands
//!         .dialog()
//!         .set_directory("/")
//!         .set_title("My Save Dialog")
//!         .add_filter("Text", &["txt"])
//!         .save_file::<MySaveDialog>();
//! }
//! ```
//!
//! When you load multiple files at once with
//! [`FileDialog::load_multiple_files`], you receive them each as separate event
//! in [`EventReader<DialogFileLoaded<T>>`] but they are sent as a batch,
//! meaning you get them all at once.
//!
//! The same thing applies to [`FileDialog::pick_multiple_directory_paths`] and
//! [`EventReader<pick::DialogDirectoryPicked<T>>`].

use std::io;
use std::marker::PhantomData;
use std::path::Path;

use bevy_app::prelude::*;
use bevy_derive::Deref;
use bevy_ecs::prelude::*;
use bevy_tasks::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};
use rfd::AsyncFileDialog;

#[cfg(not(target_arch = "wasm32"))]
mod pick;

#[cfg(not(target_arch = "wasm32"))]
pub use pick::*;

pub mod prelude {
    //! Prelude containing all types you need for saving/loading files with dialogs.
    pub use crate::{
        DialogFileLoadCanceled, DialogFileLoaded, DialogFileSaveCanceled, DialogFileSaved,
        FileDialogExt, FileDialogPlugin,
    };

    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::{
        DialogDirectoryPickCanceled, DialogDirectoryPicked, DialogFilePickCanceled,
        DialogFilePicked,
    };
}

/// Add this plugin to Bevy App to use the `FileDialog` resource in your system
/// to save/load files.
#[derive(Default)]
pub struct FileDialogPlugin(Vec<RegisterIntent>);

type RegisterIntent = Box<dyn Fn(&mut App) + Send + Sync + 'static>;

/// Marker trait saying that data can be saved to file.
pub trait SaveContents: Send + Sync + 'static {}

/// Marker trait saying that data can be loaded from file.
pub trait LoadContents: Send + Sync + 'static {}

impl<T> SaveContents for T where T: Send + Sync + 'static {}

impl<T> LoadContents for T where T: Send + Sync + 'static {}

impl FileDialogPlugin {
    /// Create new file dialog plugin. Do not forget to call at least one
    /// `with_save_file`, `with_load_file` or `with_pick_directory` on the plugin to allow you to
    /// save/load files and pick directories.
    pub fn new() -> Self {
        Default::default()
    }

    /// Allow saving file contents. This allows you to call
    ///  `dialog().save_file::<T>()` on [`Commands`]. For each `with_save_file` you
    /// will receive [`DialogFileSaved<T>`] in your systems when `save_file`
    /// completes.
    pub fn with_save_file<T: SaveContents>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            let (tx, rx) = bounded::<DialogResult<DialogFileSaved<T>>>(1);
            app.insert_resource(StreamSender(tx));
            app.insert_resource(StreamReceiver(rx));
            app.add_event::<DialogFileSaved<T>>();
            app.add_event::<DialogFileSaveCanceled<T>>();
            app.add_systems(
                First,
                handle_dialog_result::<DialogFileSaved<T>, DialogFileSaveCanceled<T>>,
            );
        }));
        self
    }

    /// Allow loading file contents. This allows you to call
    ///  `dialog().load_file::<T>()` on [`Commands`]. For each `with_load_file` you
    /// will receive [`DialogFileLoaded<T>`] in your systems when `load_file`
    /// completes.
    pub fn with_load_file<T: LoadContents>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            let (tx, rx) = bounded::<DialogResult<DialogFileLoaded<T>>>(1);
            app.insert_resource(StreamSender(tx));
            app.insert_resource(StreamReceiver(rx));
            app.add_event::<DialogFileLoaded<T>>();
            app.add_event::<DialogFileLoadCanceled<T>>();
            app.add_systems(
                First,
                handle_dialog_result::<DialogFileLoaded<T>, DialogFileLoadCanceled<T>>,
            );
        }));
        self
    }
}

#[derive(Resource, Deref)]
struct StreamReceiver<T>(Receiver<T>);

#[derive(Resource, Deref)]
struct StreamSender<T>(Sender<T>);

enum DialogResult<T> {
    Single(T),
    Batch(Vec<T>),
    Canceled,
}

fn handle_dialog_result<E: Event, C: Event + Default>(
    receiver: Res<StreamReceiver<DialogResult<E>>>,
    mut ev_done: EventWriter<E>,
    mut ev_canceled: EventWriter<C>,
) {
    for result in receiver.try_iter() {
        match result {
            DialogResult::Single(event) => ev_done.send(event),
            DialogResult::Batch(events) => ev_done.send_batch(events),
            DialogResult::Canceled => ev_canceled.send_default(),
        }
    }
}

/// Event that gets sent when file contents get saved to file system.
#[derive(Event)]
pub struct DialogFileSaved<T: SaveContents> {
    /// Name of saved file.
    pub file_name: String,

    /// Result of save file system operation.
    pub result: io::Result<()>,

    marker: PhantomData<T>,
}

/// Event that gets sent when file contents get loaded from file system.
#[derive(Event)]
pub struct DialogFileLoaded<T: LoadContents> {
    /// Name of loaded file.
    pub file_name: String,

    /// Byte contents of loaded file.
    pub contents: Vec<u8>,

    marker: PhantomData<T>,
}

/// Event that gets sent when user closes file load dialog without picking any file.
#[derive(Event)]
pub struct DialogFileLoadCanceled<T: LoadContents>(PhantomData<T>);

impl<T: LoadContents> Default for DialogFileLoadCanceled<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Event that gets sent when user closes file save dialog without saving any file.
#[derive(Event)]
pub struct DialogFileSaveCanceled<T: SaveContents>(PhantomData<T>);

impl<T: SaveContents> Default for DialogFileSaveCanceled<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl Plugin for FileDialogPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            !self.0.is_empty(),
            "File dialog not initialized, use at least one FileDialogPlugin::with_*"
        );

        for action in &self.0 {
            action(app);
        }
    }
}

/// File dialog for saving/loading files. You can further customize what can be
/// saved/loaded and the initial state of dialog with its functions.
pub struct FileDialog<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    dialog: AsyncFileDialog,
}

impl<'w, 's, 'a> FileDialog<'w, 's, 'a> {
    /// Add file extension filter.
    ///
    /// Takes in the name of the filter, and list of extensions
    ///
    /// The name of the filter will be displayed on supported platforms:
    ///   * Windows
    ///   * Linux
    ///
    /// On platforms that don't support filter names, all filters will be merged into one filter
    pub fn add_filter(mut self, name: impl Into<String>, extensions: &[impl ToString]) -> Self {
        self.dialog = self.dialog.add_filter(name, extensions);
        self
    }

    /// Set starting directory of the dialog. Supported platforms:
    ///   * Linux ([GTK only](https://github.com/PolyMeilex/rfd/issues/42))
    ///   * Windows
    ///   * Mac
    pub fn set_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.dialog = self.dialog.set_directory(path);
        self
    }

    /// Set starting file name of the dialog. Supported platforms:
    ///  * Windows
    ///  * Linux
    ///  * Mac
    pub fn set_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.dialog = self.dialog.set_file_name(file_name);
        self
    }

    /// Set the title of the dialog. Supported platforms:
    ///  * Windows
    ///  * Linux
    ///  * Mac (Only below version 10.11)
    ///  * WASM32
    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.dialog = self.dialog.set_title(title);
        self
    }

    /// Open save file dialog and save the `contents` to that file. When file
    /// gets saved, the [`DialogFileSaved<T>`] gets sent. You can get read this event
    /// with Bevy's [`EventReader<DialogFileSaved<T>>`] system param.
    pub fn save_file<T: SaveContents>(self, contents: Vec<u8>) {
        self.commands.add(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogFileSaved<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_save_file::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let file = self.dialog.save_file().await;

                    let Some(file) = file else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let event = DialogFileSaved {
                        file_name: file.file_name(),
                        result: file.write(&contents).await,
                        marker: PhantomData,
                    };

                    sender.send(DialogResult::Single(event)).unwrap();
                })
                .detach();
        });
    }

    /// Open pick file dialog and load its contents. When file contents get
    /// loaded, the [`DialogFileLoaded<T>`] gets sent. You can read this event with
    /// Bevy's [`EventReader<DialogFileLoaded<T>>`].
    pub fn load_file<T: LoadContents>(self) {
        self.commands.add(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogFileLoaded<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_load_file::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let file = self.dialog.pick_file().await;

                    let Some(file) = file else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let event = DialogFileLoaded {
                        file_name: file.file_name(),
                        contents: file.read().await,
                        marker: PhantomData,
                    };

                    sender.send(DialogResult::Single(event)).unwrap();
                })
                .detach();
        });
    }

    /// Open pick file dialog for multiple files and load contents for all
    /// selected files. When file contents get loaded, the
    /// [`DialogFileLoaded<T>`] gets sent for each file. You can read each file
    /// by reading every event received with with Bevy's
    /// [`EventReader<DialogFileLoaded<T>>`].
    pub fn load_multiple_files<T: LoadContents>(self) {
        self.commands.add(|world: &mut World| {
            let sender = world
                .get_resource::<StreamSender<DialogResult<DialogFileLoaded<T>>>>()
                .expect("FileDialogPlugin not initialized with 'with_load_file::<T>()'")
                .0
                .clone();

            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let files = AsyncFileDialog::new().pick_files().await;

                    let Some(files) = files else {
                        sender.send(DialogResult::Canceled).unwrap();
                        return;
                    };

                    let mut events = Vec::new();
                    for file in files {
                        events.push(DialogFileLoaded {
                            file_name: file.file_name(),
                            contents: file.read().await,
                            marker: PhantomData,
                        });
                    }

                    sender.send(DialogResult::Batch(events)).unwrap();
                })
                .detach();
        });
    }
}

/// Extension trait for [`Commands`] that allow you to create dialogs.
pub trait FileDialogExt<'w, 's> {
    /// Create dialog for loading/saving files.
    #[must_use]
    fn dialog<'a>(&'a mut self) -> FileDialog<'w, 's, 'a>;
}

impl<'w, 's> FileDialogExt<'w, 's> for Commands<'w, 's> {
    fn dialog<'a>(&'a mut self) -> FileDialog<'w, 's, 'a> {
        FileDialog {
            commands: self,
            dialog: AsyncFileDialog::new(),
        }
    }
}
