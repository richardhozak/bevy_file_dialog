#![warn(missing_docs)]

//! Bevy plugin that allows you to save and load files with file dialogs.
//!
//! In order to use it you need to add [`FileDialogPlugin`] with one or more calls
//! to [`FileDialogPlugin::with_save`] or [`FileDialogPlugin::with_load`]:
//!
//! Here is a complete example showing all the features of the plugin:
//! ```rust
//! use bevy::prelude::*;
//! use bevy_file_dialog::prelude::*;
//!
//! struct LevelContents;
//! struct SaveGameContents;
//!
//! fn main() {
//!     App::new()
//!            .add_plugins(DefaultPlugins)
//!            // Add the file dialog plugin and specify that we want to load `LevelContents`
//!            // and save and load `SaveGameContents`
//!            .add_plugins(
//!                 FileDialogPlugin::new()
//!                     .with_load::<LevelContents>()
//!                     .with_save::<SaveGameContents>()
//!                     .with_load::<SaveGameContents>()
//!             )
//!            .add_systems(PreUpdate, handle_input)
//!            .add_systems(
//!                 Update,
//!                 (
//!                     level_contents_loaded,
//!                     save_game_contents_loaded,
//!                     save_game_contents_saved
//!                 )
//!            )
//!            .run();
//! }
//!
//! fn level_contents_loaded(mut ev_level_contents: EventReader<DialogFileLoaded<LevelContents>>) {
//!     for event in ev_loaded.read() {
//!         eprintln!("Loaded level {} with size of {} bytes", event.file_name, event.contents.len());
//!         // You can now deserialize the bytes contained in event.contents into a level
//!     }
//! }
//!
//! fn save_game_contents_loaded(mut ev_level_contents: EventReader<DialogFileLoaded<LevelContents>>) {
//!     for event in ev_loaded.read() {
//!         eprintln!("Loaded save game {} with size of {} bytes", event.file_name, event.contents.len());
//!         // You can now deserialize the bytes contained in event.contents into a save game
//!     }
//! }
//!
//! fn save_game_contents_saved(mut ev_level_contents: EventReader<DialogFileSaved<LevelContents>>) {
//!     for event in ev_loaded.read() {
//!         eprintln!("Loaded save game {} with result {:?}", event.file_name, event.result);
//!         // You can inspect event.result and show player the result of saving a game
//!     }
//! }
//!
//! fn handle_input(mut commands: Commands, input: Res<Input<KeyCode>>) {
//!     if input.just_pressed(KeyCode::L) {
//!         commands
//!             .dialog()
//!             .add_filter("level", &["level", "lvl"])
//!             .set_title("Load level")
//!             .load_file::<LevelContents>();
//!     } else if input.just_pressed(KeyCode::S) {
//!         let save_game_content = Vec::new(); // You'd serialize your save game to bytes here instead of Vec::new()
//!
//!         commands
//!             .dialog()
//!             .set_directory("/")
//!             .set_title("Save game")
//!             .save_file::<SaveGameContents>(save_game_content);
//!     } else if input.just_pressed(KeyCode::O) {
//!         commands
//!             .dialog()
//!             .set_directory("/")
//!             .set_title("Load game")
//!             .load_file::<SaveGameContents>();
//!     }
//! }
//! ```
//!
//! [`FileDialogPlugin::with_save`] and [`FileDialogPlugin::with_load`] can be
//! called as many times as you want, the type parameters act as markers that
//! allow you to call [`FileDialog::save_file`] and [`FileDialog::load_file`] and
//! receive the result in [`DialogFileSaved`] and [`DialogFileLoaded`] events.

use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_tasks::{prelude::*, Task};
use bevy_utils::tracing::*;
use futures_lite::future;
use rfd::AsyncFileDialog;

pub mod prelude {
    //! Prelude containing all types you need for saving/loading files with dialogs.
    pub use crate::{DialogFileLoaded, DialogFileSaved, FileDialogExt, FileDialogPlugin};
}

/// Add this plugin to Bevy App to use the `FileDialog` resource in your system
/// to save/load files.
#[derive(Default)]
pub struct FileDialogPlugin(Vec<RegisterAction>);

type RegisterAction = Box<dyn Fn(&mut App) + Send + Sync + 'static>;

/// Marker trait saying that data can be saved to file.
pub trait SaveContents: Send + Sync + 'static {}

/// Marker trait saying that data can be loaded from file
pub trait LoadContents: Send + Sync + 'static {}

impl<T> SaveContents for T where T: Send + Sync + 'static {}

impl<T> LoadContents for T where T: Send + Sync + 'static {}

impl FileDialogPlugin {
    /// Create new file dialog plugin. Do not forget to call at least one
    /// `with_save` or `with_load` on the plugin to allow you to save/load
    /// files.
    pub fn new() -> Self {
        Default::default()
    }

    /// Allow saving file contents. This allows you to call
    ///  `dialog().save_file::<T>()` on [`Commands`]. For each `with_save` you
    /// will receive [`DialogFileSaved<T>`] in your systems when `save_file`
    /// completes.
    pub fn with_save<T: SaveContents>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            app.add_event::<DialogFileSaved<T>>();
            app.add_systems(
                First,
                poll_save_dialog_result::<T>.run_if(resource_exists::<SaveDialog<T>>()),
            );
        }));
        self
    }

    /// Allow loading file contents. This allows you to call
    ///  `dialog().load_file::<T>()` on [`Commands`]. For each `with_load` you
    /// will receive [`DialogFileLoaded<T>`] in your systems when `load_file`
    /// completes.
    pub fn with_load<T: LoadContents>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            app.add_event::<DialogFileLoaded<T>>();
            app.add_systems(
                First,
                poll_load_dialog_result::<T>.run_if(resource_exists::<LoadDialog<T>>()),
            );
        }));
        self
    }
}

fn poll_load_dialog_result<T: LoadContents>(
    mut commands: Commands,
    mut dialog: ResMut<LoadDialog<T>>,
    mut ev_saved: EventWriter<DialogFileLoaded<T>>,
) {
    if let Some(result) = future::block_on(future::poll_once(&mut dialog.task)) {
        if let Some((file_name, contents)) = result {
            ev_saved.send(DialogFileLoaded {
                file_name,
                contents,
                marker: PhantomData,
            });
        } else {
            info!("Load dialog closed");
        }

        commands.remove_resource::<LoadDialog<T>>();
    }
}

fn poll_save_dialog_result<T: SaveContents>(
    mut commands: Commands,
    mut dialog: ResMut<SaveDialog<T>>,
    mut ev_saved: EventWriter<DialogFileSaved<T>>,
) {
    if let Some(result) = future::block_on(future::poll_once(&mut dialog.task)) {
        if let Some((file_name, result)) = result {
            ev_saved.send(DialogFileSaved {
                file_name,
                result,
                marker: PhantomData,
            });
        } else {
            info!("Save dialog closed");
        }

        commands.remove_resource::<SaveDialog<T>>();
    }
}

#[derive(Resource)]
struct LoadDialog<T: LoadContents> {
    task: Task<Option<(String, Vec<u8>)>>,
    marker: PhantomData<T>,
}

#[derive(Resource)]
struct SaveDialog<T: SaveContents> {
    task: Task<Option<(String, Result<(), io::Error>)>>,
    marker: PhantomData<T>,
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

impl Plugin for FileDialogPlugin {
    fn build(&self, app: &mut App) {
        assert!(!self.0.is_empty(), "File dialog not initialized, use at least one FileDialogPlugin::with_save or FileDialogPlugin::with_load");

        for action in &self.0 {
            action(app);
        }
    }
}

/// File dialog for saving/loading files. You can further customize what can be
/// saved/loaded and the initial state of dialog with its functions.
pub struct FileDialog<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    filters: Vec<(String, Vec<String>)>,
    starting_directory: Option<PathBuf>,
    file_name: Option<String>,
    title: Option<String>,
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
        self.filters.push((
            name.into(),
            extensions.iter().map(|e| e.to_string()).collect(),
        ));
        self
    }

    /// Set starting directory of the dialog. Supported platforms:
    ///   * Linux ([GTK only](https://github.com/PolyMeilex/rfd/issues/42))
    ///   * Windows
    ///   * Mac
    pub fn set_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path = path.as_ref();
        if path.to_str().map(|p| p.is_empty()).unwrap_or(false) {
            self.starting_directory = None;
        } else {
            self.starting_directory = Some(path.into());
        }
        self
    }

    /// Set starting file name of the dialog. Supported platforms:
    ///  * Windows
    ///  * Linux
    ///  * Mac
    pub fn set_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    /// Set the title of the dialog. Supported platforms:
    ///  * Windows
    ///  * Linux
    ///  * Mac (Only below version 10.11)
    ///  * WASM32
    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Open save file dialog and save the `contents` to that file. When file
    /// gets saved, the [`DialogFileSaved<T>`] gets sent. You can get read this event
    /// with Bevy's [`EventReader<DialogFileSaved<T>>`] system param.
    pub fn save_file<T: SaveContents>(self, contents: Vec<u8>) {
        self.commands.add(|world: &mut World| {
            let task = AsyncComputeTaskPool::get().spawn(async move {
                let file = AsyncFileDialog::new().save_file().await;

                if let Some(file) = file {
                    Some((file.file_name(), file.write(&contents).await))
                } else {
                    None
                }
            });

            let marker = PhantomData::<T>;

            world.remove_resource::<SaveDialog<T>>();
            world.insert_resource(SaveDialog { task, marker });
        });
    }

    /// Open pick file dialog and load its contents. When file contents get
    /// loaded, the [`DialogFileLoaded<T>`] gets sent. You can read this event with
    /// Bevy's [`EventReader<DialogFileLoaded<T>>`].
    pub fn load_file<T: LoadContents>(self) {
        self.commands.add(|world: &mut World| {
            let task = AsyncComputeTaskPool::get().spawn(async move {
                let file = AsyncFileDialog::new().pick_file().await;

                if let Some(file) = file {
                    Some((file.file_name(), file.read().await))
                } else {
                    None
                }
            });

            let marker = PhantomData::<T>;

            world.remove_resource::<LoadDialog<T>>();
            world.insert_resource(LoadDialog { task, marker });
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
            filters: Vec::new(),
            starting_directory: None,
            file_name: None,
            title: None,
        }
    }
}
