use std::{fs, path::PathBuf};


use serde::{Deserialize, Deserializer, Serialize};

#[cfg(test)]
mod test;

/// Serves only to trick serde
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
struct DriveLetterWrapper(char);

impl Default for DriveLetterWrapper {
    fn default() -> Self {
        Self('Z')
    }
}

impl DriveLetterWrapper {
    fn is_default(&self) -> bool {
        self.0.eq_ignore_ascii_case(&'z')
    }
}

/// The definitions for memory maps for this specfic Prefix/Game.  
///
/// The game_id is usually read from the AppID in the steam launch command,
/// but can be overwritten here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBridgeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub maps: Vec<MemMapConfig>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apps: Vec<App>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_app: Option<App>,

    // Useful to the individual programm to store the version for example
    // Allows them to update the version number if needed
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "never_fail_notes")]
    notes: Option<String>, 

    // Serves also to prevent manual instanciation, and breakage cause by it and new options
    #[serde(default, skip_serializing_if = "DriveLetterWrapper::is_default")]
    root_mount_point: DriveLetterWrapper
}

/// Insures that even if notes is not properly serialized, that the struct does not fail
fn never_fail_notes<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Ok(s) = Deserialize::deserialize(deserializer) {
        let s: Option<String> = s;
        Ok(s)
    } else {
        // We can not afford to fail this, in case someone wrote some other datatype into this,
        // so that the bridge still works we ommit it this way
        Ok(None)
    }
}

impl Default for GameBridgeConfig {
    fn default() -> Self {
        Self { game_id: None, maps: Vec::default(), apps: Vec::default(), root_mount_point: DriveLetterWrapper::default(), post_app: None, notes: None }
    }
}

impl GameBridgeConfig {
    /// Removes dublicate memory maps and apps
    ///
    /// This is done by keeping the larger maps, while apps are kept as long as at least one
    /// argument is different
    pub fn sanitize(&mut self) {
        self.maps.sort_by(|a,b| b.size.cmp(&a.size));

        let mut cached_names = Vec::<String>::with_capacity(self.maps.len().max(self.apps.len()));
        let mut index = 0;

        while let Some(item) = self.maps.get(index) {
            if cached_names.contains(&item.name) {
                self.maps.remove(index);
            } else {
                cached_names.push(item.name.clone());
                index += 1;
            }
        }

        // Removing dublicate commands
        cached_names.clear();
        index = 0;

        while let Some(item) = self.apps.get(index) {
            let rep = item.to_string();

            if cached_names.contains(&rep) {
                self.apps.remove(index);
            } else {
                cached_names.push(rep);
                index += 1;
            }
        }

    }


    /// Sets Memory maps for this config
    pub fn with_memory_maps(mut self, maps: Vec<MemMapConfig>) -> Self {
        self.maps = maps;
        self
    }

    /// Sets the game_id override for this config
    pub fn with_name_override(mut self, game_id: String) -> Self {
        self.game_id = Some(game_id);
        self
    }

    /// Gives the game_id override, if set
    pub fn get_name_override<'a>(&'a self) -> Option<&'a String> {
        self.game_id.as_ref()
    }

    /// Sets the game_id override
    pub fn set_name_override(&mut self, game_id: Option<String>) {
        self.game_id = game_id;
    }

    /// Sets additional programms to be launched
    pub fn with_autolaunch_apps(mut self, apps: Vec<App>) -> Self {
        self.apps = apps;
        self
    }

    /// The default wine mountpoint for the linux root is Z, but in case this was changed you can
    /// override it with this function
    pub fn with_override_root_mountpoint(mut self, drive_letter: char) -> Self {
        self.root_mount_point = DriveLetterWrapper(drive_letter);
        self
    }

    /// Returns the drive letter under which (according to this config) the root of the linux
    /// filesystem is mounted (usually this is Z)
    pub fn get_root_mount_point(&self) -> char {
        self.root_mount_point.0
    }

    /// This adds an App/Command to run after the game exited and the apps closed.
    /// Useful for cleanup purposes
    pub fn with_post_run_app(mut self, app: App) -> Self {
        self.post_app = Some(app);
        self
    }

    /// Adds notes to this Config.
    ///
    /// This is useful so you can for example denote the version of this config,
    /// so you can check if this config file is up to date at a later date
    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Gives you back the notes for this config (if set)
    ///
    /// This is useful so you can for example denote the version of this config,
    /// so you can check if this config file is up to date at a later date
    pub fn get_notes<'a>(&'a self) -> Option<&'a String> {
        self.notes.as_ref()
    }

    /// Sets the notes for this config to the given value
    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }


    /// Converts a linux path, by appending the drive letter for the mount point and converting the
    /// remainder of the path.
    ///
    /// Path needs to be an absolute path
    #[cfg(target_os = "windows")]
    pub fn convert_linux_path_to_wine(&self, path: String) -> String {
        convert_linux_path_to_wine(self.root_mount_point.0, path)
    }

    /// Writes a config with this name into the prefix.
    ///
    /// If the config file already exists, and overwrite is false, then it won't be written, and
    /// false is returned (flase is also returned if any other write error occures)
    #[cfg(feature = "proton")]
    pub fn write_config(self, game_drive: &proton_finder::GameDrive, name: &str, overwrite: bool) -> bool {
        if let Some(mut path) = get_path_from_prefix(game_drive) {
            path.push(format!("{name}.json"));
            manual_write_config(&path, self, overwrite)
        } else {
            false
        }
    }

    /// Reads the config with this name within this prefix
    #[cfg(feature = "proton")]
    pub fn read_config(game_drive: &proton_finder::GameDrive, name: &str) -> Result<Self, String> {
        let mut path = get_path_from_prefix(game_drive).ok_or("Failed to generate file path".to_string())?;
        path.push(format!("{name}.json"));
        manual_read_config(&path)
    }

    /// Reads all configs for this prefix, the same way the Datalink bridge does it.
    ///
    /// This returns the merger of all the configs:
    /// - Maps with the same name we keep the largest
    /// - Apps only match if both exec and arguments match completly (else both are kept)
    ///
    /// If different game_ids are set, then the Option<Vec> will return Some containing the other names.
    /// If one config has the override unset, then `GameBridgeConfig.game_id` will be None, but the
    /// Option<Vec> can still be some and contain the overrides from other configs.
    ///
    /// If different root_mount_points are set, then the one found first will be used (and a
    /// message is logged out). The order in which is read is not necessarily alphabetic, and could
    /// even change run to run.
    #[cfg(feature = "proton")]
    pub fn read_config_for_prefix(game_drive: &proton_finder::GameDrive) -> (Option<(Self, Option<Vec<String>>)>, Result<(), String>) {
        let path = if let Some(val) = get_path_from_prefix(game_drive) {
            val
        } else {
            return (None, Err("Failed to get folder path".to_string()));
        };
        manual_read_configs_from_folder(&path)
    }
}

/// Size and name for an individual memory map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemMapConfig {
    pub name: String,
    pub size: usize
}

impl MemMapConfig {
    /// Creates a new Config for a Memory Map that can hold the struct `T`
    pub fn new<T>(name: String) -> Self where T:Sized {
        let size = std::mem::size_of::<T>();
        MemMapConfig { name, size }
    }
}

/// Defines an app to be launched with the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    path: String,
    args: Vec<String>
}

impl App {

    /// Creates a new autolaunch app  
    ///
    /// The path needs to follow the following guidelines:
    /// - linux paths can only start from root (Datalink will auto convert these), with /
    /// - Windows paths need a drive letter, and singular \
    /// - Relative paths have to start with a dot, pathing relative to the config location, either
    /// style of slashes supported
    pub fn new(exec: String) -> Option<Self> {
        let c = exec.chars().next()?;

        if c.is_alphabetic() && !exec.contains('/') && !exec.contains("\\\\") {
            // Windows
            let pre = format!("{c}:\\");
            if exec.strip_prefix(&pre).is_some() {
                return Some(Self { path: exec, args: vec![] });
            }

        } else if c == '/' && !exec.contains('\\') {
            // Linux
            return Some(Self { path: exec, args: vec![] });
        } else if c == '.' {
            return Some(Self { path: exec, args: vec![] });
        }
        
        None
    }
    
    /// Adds arguments to this App
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Returns the exec path set for this App
    pub fn get_exec<'a>(&'a self) -> &'a str {
        self.path.as_str()
    }

    /// Returns the arguments set for this App
    pub fn get_args_as_ref<'a>(&'a self) -> &'a Vec<String> {
        &self.args
    }

    /// Returns the arguments set for this App, consuming the app
    pub fn get_args(self) -> Vec<String> {
        self.args
    }

    /// Returns a userfacing name for the application being launched.  
    ///
    /// This is usually the exe name
    pub fn get_name<'a>(&'a self) -> &'a str {
        let win = if let Some(last) = self.path.split('\\').last() {
            last
        } else {
            self.path.as_str()
        };

        let lin = if let Some(last) = self.path.split('/').last() {
            last
        } else {
            self.path.as_str()
        };

        // Why do we do this?
        // Because the relative path could technically have both types of symbol, so
        // we check for the path with the shortest result, excluding empty

        match (win.is_empty(), lin.is_empty(), win.len() > lin.len()) {
            (true, true, _) => self.path.as_str(),
            (false, true, _) => win,
            (true, false, _) => lin,
            (false, false, true) => lin,
            (false, false, false) => win
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_command(self, drive_letter: char) -> Option<std::process::Command> {
        let c = self.path.chars().next()?;

        let path = if c.is_alphabetic() {
            // Windows path, already formated
            self.path
        } else if c == '/' {
            // Linux path
            convert_linux_path_to_wine(drive_letter, self.path)
        } else {
            // Relative path
            let rel = self.path.replace("/", "\\");
            
            let mut path = get_config_folder_path()?;
            path.push(rel);

            path.to_str()?.to_string()
        };



        let mut cmd = std::process::Command::new(path);
        cmd.args(self.args);

        Some(cmd)
    }
}

impl ToString for App {
    fn to_string(&self) -> String {
        let mut output = self.path.clone();

        for item in self.args.iter() {
            output = output + item;
        }

        output
    }
}


/// Converts a linux path to a path to a wine/windows path.  
///   
/// This is done by converting the slashes and adding appropriate drive letter.  
///
/// Default letter is usually Z
#[cfg(target_os = "windows")]
pub fn convert_linux_path_to_wine(drive_letter: char, path: String) -> String {
    let p = path.replace('/', "\\");
    let complete = format!("{}:", drive_letter.to_ascii_uppercase()) + &p;

    complete
}

    
/// Returns the expected folder for the config (for this prefix), usually
/// C:\Users\[current]\AppData\Roaming\Datalink\
#[cfg(target_os = "windows")]
pub fn get_config_folder_path() -> Option<PathBuf> {
    let mut path = dirs::config_dir()?;
    path.push("Datalink");
    Some(path)
}

/// Aquires the config (for this prefix) from
/// C:\Users\[current]\AppData\Roaming\Datalink
#[cfg(target_os = "windows")]
pub fn read_config() -> (Option<(GameBridgeConfig, Option<Vec<String>>)>, Result<(), String>) {
    let path = match get_config_folder_path() {
        Some(p) => p,
        None => return (None, Ok(()))
    };

    if !path.exists() {
        if let Err(e) = std::fs::create_dir(path.as_path()) {
            return (None, Err(format!("Failed to created none existant folder: {e}")));
        }
    }

    manual_read_configs_from_folder(&path)
}

/// Finds the folder within the prefix
#[cfg(feature = "proton")]
pub(crate) fn get_path_from_prefix(game_drive: &proton_finder::GameDrive) -> Option<PathBuf> {
    let mut path = game_drive.config_dir()?;
    if !path.exists() {
        return None;
    }

    path.push("Datalink");
    if !path.exists() {
        // Folder does not exist, so we create it
        fs::create_dir(path.as_path()).ok()?;
    }

    Some(path)
}


/// Writes to config to a path you specified.
///
/// If a file already exists, and overwrite is false, then no file is written, and returns false.
/// It can also return false if the write fails.
///
/// Manual forces you to make sure it is written where the Datalink bridge can read it, effectively: 
/// C:\Users\[current]\AppData\Roaming\Datalink\*.json
/// Using the proton feature allows you to use proton-finder and skip finding the path
pub fn manual_write_config(path: &PathBuf, config: GameBridgeConfig, overwrite: bool) -> bool {
    if let Ok(text) = serde_json::to_string_pretty(&config) {
        if path.exists() && !overwrite {
            return false;
        }

        if fs::write(path, text).is_ok() {
            return true;
        }
    }


    false
}

/// Reads a config at the path you manually specified.
///
/// Manual forces you to make sure this is the correct location, effectively:
/// C:\Users\[current]\AppData\Roaming\Datalink\*.json
/// Using the proton feature allows you to use proton-finder and skip finding the path
pub fn manual_read_config(path: &PathBuf) -> Result<GameBridgeConfig, String> {
    let conf = fs::read_to_string(path.as_path()).map_err(|e| format!("Failed to read {}: {}", path.to_str().unwrap_or("<no path>"), e))?;
    serde_json::from_str(&conf).map_err(|e| format!("Failed to parse {}: {}", path.to_str().unwrap_or("<no path>"), e))
}


/// Reads through all config in the relevant folder, merging them together.
/// 
/// If different game_ids are set, then the Option<Vec> will return Some containing the other names.
/// If one config has the override unset, then `GameBridgeConfig.game_id` will be None, but the
/// Option<Vec> can still be some and contain the overrides from other configs.
///
/// This returns the merger of all the configs:
/// - Maps with the same name we keep the largest
/// - Apps only match if both exec and arguments match completly (else both are kept)
///
/// If different root_mount_points are set, then the one found first will be used (and a
/// message is logged out). The order in which is read is not necessarily alphabetic, and could
/// even change run to run.
///
/// Manual forces you to make sure this is the correct location, effecitvely:
/// C:\Users\[current]\AppData\Roaming\Datalink
/// Using the proton feature allows you to use proton-finder and skip finding the path
pub fn manual_read_configs_from_folder(folder: &PathBuf) -> (Option<(GameBridgeConfig, Option<Vec<String>>)>, Result<(), String>) {
    // Helper function that merges two entries
    fn merge(read: GameBridgeConfig, res: &mut Option<(GameBridgeConfig, Option<Vec<String>>)>, err: &mut String) {
        let (old, alt_name_list) = if let Some((old, alt_name_list)) = res {
            (old, alt_name_list)
        } else {
            *res = Some((read, None));
            return;
        };

        // Merging mountpoint
        if !read.root_mount_point.is_default() && !old.root_mount_point.is_default() &&
            !old.root_mount_point.0.eq_ignore_ascii_case(&read.root_mount_point.0) {

            // Both are set, and to different values
            // Sort of a problem, but eh, screw it
            
            *err = format!("{err}\nTwo configs had different root mountpoints set: {} and {}, using {}", old.get_root_mount_point(), read.get_root_mount_point(), old.get_root_mount_point());

        } else if !old.root_mount_point.is_default() {
            // Original is unset, so we return the new one
            // Possible the new one is also unset, but doesn't matter, this will then just unset it again
            old.root_mount_point.0 = read.root_mount_point.0;
        } else {
            // Original is set, new must be unset or the same, so this is fine
        }

        // Merging appid override
        if old.game_id.is_some() && read.game_id.is_none() {
            // We will set the override in old to none, and move it into the alt_name_list
            // This way we can inform that one config requested the original game_id
            
            let value = old.game_id.clone().expect("We just checked for some");
            
            let list = if let Some(list) = alt_name_list {
                list
            } else {
                *alt_name_list = Some(vec![]);
                alt_name_list.as_mut().expect("We literally just assigned it")
            };

            (*old).game_id = None;
            if !list.contains(&value) {
                list.push(value);
            }
        } else if let Some(new) = read.game_id.as_ref() {
            // old must be none, so we append this one to the alt list and leave old as none
            // because of the reasons mentioned above
            fn write_into_it(alt_name_list: &mut Option<Vec<String>>, new: &String) {
                let list = if let Some(list) = alt_name_list {
                    list
                } else {
                    *alt_name_list = Some(vec![]);
                    alt_name_list.as_mut().expect("We literally just assigned it")
                };

                if !list.contains(new) {
                    list.push(new.to_string());
                }
            }

            if let Some(older) = old.game_id.as_ref() {
                if new != older {
                    write_into_it(alt_name_list, new)
                }
            } else {
                write_into_it(alt_name_list, new)
            }
        }

        // Merge Applist
        let mut read = read;
        old.apps.append(&mut read.apps);

        old.maps.append(&mut read.maps);
    }

    let dir = match folder.read_dir() {
        Ok(dir) => dir,
        Err(e) => return (None, Err(format!("Failed to read folder: {e}")))
    };

    let mut res:Option<(GameBridgeConfig, Option<Vec<String>>)> = None;
    let mut err = String::new();

    for item in dir {
        match item {
            Ok(item) => {
                if let Some(ext) = item.path().extension() {
                    if ext == "json" {
                        match manual_read_config(&item.path()) {
                            Ok(read) => {
                                merge(read, &mut res, &mut err);
                            },
                            Err(e) => {
                                err = format!("{err}\n{}", e);
                            }
                        }
                    }
                    // The extension checks do not need an else case, as we just ignore any file
                    // that doesn't end in json
                }
            },
            Err(e) => {
                err = format!("{err}\nUnable to get directory entry: {}", e);
            }
        }
    }

    // Sanitizing
    if let Some((val, _)) = res.as_mut() {
        val.sanitize();
    }

    let err = if err.is_empty() {
        Ok(())
    } else {
        Err(err)
    };

    (res, err)
}
