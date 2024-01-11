use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_tasks::{prelude::*, Task};
use bevy_utils::tracing::*;
use futures_lite::future;
use rfd::AsyncFileDialog;

/// Add this plugin to Bevy App to use the `FileDialog` resource in your system
/// to save/load files.
#[derive(Default)]
pub struct FileDialogPlugin(Vec<RegisterAction>);

type RegisterAction = Box<dyn Fn(&mut App) + Send + Sync + 'static>;

impl FileDialogPlugin {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_save<T: Send + Sync + 'static>(mut self) -> Self {
        self.0.push(Box::new(|app| {
            app.add_event::<DialogFileSaved<T>>();
            app.add_systems(
                First,
                poll_save_dialog_result::<T>.run_if(resource_exists::<SaveDialog<T>>()),
            );
        }));
        self
    }

    pub fn with_load<T: Send + Sync + 'static>(mut self) -> Self {
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

fn poll_load_dialog_result<T: Send + Sync + 'static>(
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
            info!("Save dialog closed");
        }

        commands.remove_resource::<LoadDialog<T>>();
    }
}

fn poll_save_dialog_result<T: Send + Sync + 'static>(
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
struct LoadDialog<T: Send + Sync + 'static> {
    task: Task<Option<(String, Vec<u8>)>>,
    marker: PhantomData<T>,
}

#[derive(Resource)]
struct SaveDialog<T: Send + Sync + 'static> {
    task: Task<Option<(String, Result<(), io::Error>)>>,
    marker: PhantomData<T>,
}

/// Event that gets sent when file contents get saved to file system.
/// TODO: more docs
#[derive(Event)]
pub struct DialogFileSaved<T: Send + Sync + 'static> {
    /// Name of saved file.
    pub file_name: String,

    /// Result of save file system operation.
    pub result: io::Result<()>,

    marker: PhantomData<T>,
}

/// Event that gets sent when file contents get loaded from file system.
/// TODO: more docs
#[derive(Event)]
pub struct DialogFileLoaded<T: Send + Sync + 'static> {
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

pub struct Dialog<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    filters: Vec<(String, Vec<String>)>,
    starting_directory: Option<PathBuf>,
    file_name: Option<String>,
    title: Option<String>,
}

impl<'w, 's, 'a> Dialog<'w, 's, 'a> {
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
    /// TODO: more examples
    pub fn save_file<T: Send + Sync + 'static>(self, contents: Vec<u8>) {
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
    /// TODO: more examples
    pub fn load_file<T: Send + Sync + 'static>(self) {
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

pub trait FileDialogExt<'w, 's> {
    #[must_use]
    fn dialog<'a>(&'a mut self) -> Dialog<'w, 's, 'a>;
}

impl<'w, 's> FileDialogExt<'w, 's> for Commands<'w, 's> {
    fn dialog<'a>(&'a mut self) -> Dialog<'w, 's, 'a> {
        Dialog {
            commands: self,
            filters: Vec::new(),
            starting_directory: None,
            file_name: None,
            title: None,
        }
    }
}
