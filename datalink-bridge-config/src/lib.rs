use std::{fs, path::PathBuf};


use serde::{Serialize, Deserialize};

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


    #[serde(default, skip_serializing_if = "DriveLetterWrapper::is_default")]
    root_mount_point: DriveLetterWrapper
    // Serves also to prevent manual instanciation, and breakage cause by it and new options
}

impl Default for GameBridgeConfig {
    fn default() -> Self {
        Self { game_id: None, maps: Vec::default(), apps: Vec::default(), root_mount_point: DriveLetterWrapper::default() }
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


    /// Converts a linux path, by appending the drive letter for the mount point and converting the
    /// remainder of the path.
    ///
    /// Path needs to be an absolute path
    #[cfg(target_os = "windows")]
    pub fn convert_linux_path_to_wine(&self, path: String) -> String {
        convert_linux_path_to_wine(self.root_mount_point.0, path)
    }

    /// Reads (if present) the config at the prefix,
    /// if compatible with the new_config passed in they will be merged and written into the file (with the
    /// result being returned within the Ok()).
    /// If the two are not compatible, then Err(Some()) with the value being the current config.
    /// Err(None) if there is an IO error on write.
    ///
    /// Merge rules are:
    /// - The same game_id has to be set (so if the original has None, the new_config has to be too)
    /// - The list of memory maps is appeneded and sanitized:
    ///   - If there is a dupplicate, then the smaller map is removed
    /// - Apps is appeneded and sanitized
    ///   - Commands are only considered dublicates, if exec and all arguments match
    /// - root_mount_point is considered unset if set to Z, if one has a different value then Z that
    /// different value is used, if both are set then they have to be the same or else it will be
    /// treated as a missmatch like game_id
    #[cfg(feature = "proton")]
    pub fn write_config(self, game_drive: &proton_finder::GameDrive) -> Result<Self, Option<Self>> {
        if let Some(path) = get_path_from_prefix(game_drive) {
            manual_write_config(&path, self)
            
        } else {
            Err(None)
        }
    }

    /// Writes to config for the prefix, overwriting any config already present.
    ///
    /// This is undesirable, as another programm might want to reserve different maps in this
    /// prefix, and you would efecitely delete all them. Use `write_config` instead.
    #[cfg(feature = "proton")]
    pub fn force_write_config(self, game_drive: &proton_finder::GameDrive) -> bool {
        if let Some(path) = get_path_from_prefix(game_drive) {
            manual_force_write_config(&path, self)
        } else {
            false
        }
    }

    /// Reads the config within this prefix
    #[cfg(feature = "proton")]
    pub fn read_config(game_drive: &proton_finder::GameDrive) -> Option<Self> {
        let path = get_path_from_prefix(game_drive)?;
        manual_read_config(&path)
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
pub fn convert_linux_path_to_wine(drive_letter: char ,path: String) -> String {
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
/// C:\Users\[current]\AppData\Roaming\Datalink\config.json
#[cfg(target_os = "windows")]
pub fn read_config() -> Result<Option<GameBridgeConfig>, String> {
    let mut path = match get_config_folder_path() {
        Some(p) => p,
        None => return Ok(None)
    };
    path.push("config.json");

    if !path.exists() {
        return Ok(None);
    }

    let conf = fs::read_to_string(path).map_err(|e| format!("{e}"))?;
    let mut res:GameBridgeConfig = serde_json::from_str(&conf).map_err(|e| format!("{e}"))?;
    res.sanitize();
    Ok(Some(res))
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

    path.push("config.json");
    Some(path)
}


/// Writes to config to a path you specified, overwriting a file if present already.
///
/// Manual forces you to make sure it is written where the Datalink bridge can read it, effectively: 
/// C:\Users\[current]\AppData\Roaming\Datalink\config.json
/// Using the proton feature allows you to use proton-finder and skip finding the path
pub fn manual_force_write_config(path: &PathBuf, config: GameBridgeConfig) -> bool {
    if let Ok(text) = serde_json::to_string_pretty(&config) {
        if fs::write(path, text).is_ok() {
            return true;
        }
    }


    false
}

/// Reads (if present) the config at the manually specified location,
/// if compatible with the new_config passed in they will be merged and written into the file (with the
/// result being returned within the Ok()).
/// If the two are not compatible, then Err(Some()) with the value being the current config.
/// Err(None) if there is an IO error on write.
///
/// Merge rules are:
/// - The same game_id has to be set (so if the original has None, the new_config has to be too)
/// - The list of memory maps is appeneded and sanitized:
///   - If there is a dupplicate, then the smaller map is removed
/// - Apps is appeneded and sanitized
///   - Commands are only considered dublicates, if exec and all arguments match
/// - root_mount_point is considered unset if set to Z, if one has a different value then Z that
/// different value is used, if both are set then they have to be the same or else it will be
/// treated as a missmatch like game_id
///
/// Manual forces you to make sure it is written where the Datalink bridge can read it, effectively: 
/// C:\Users\[current]\AppData\Roaming\Datalink\config.json
/// Using the proton feature allows you to use proton-finder and skip finding the path
pub fn manual_write_config(path: &PathBuf, mut new_config: GameBridgeConfig) -> Result<GameBridgeConfig, Option<GameBridgeConfig>> {
    let target_config = if let Some(original) = manual_read_config(path) {
        // There is already a config, we are merging them
        if new_config.game_id != original.game_id {
            // But not if the name disagree
            return Err(Some(original));
        }

        // Handling mountpoint
        let root = if !new_config.root_mount_point.is_default() && !original.root_mount_point.is_default() && !original.root_mount_point.0.eq_ignore_ascii_case(&new_config.root_mount_point.0) {
            // Both are set, and to different values, exit
            return Err(Some(original))
        } else if !original.root_mount_point.is_default() {
            // Original is unset, so we return the new one
            // Possible the new one is also unset, but doesn't matter, this will then just unset it again
            new_config.root_mount_point
        } else {
            // Original is set, new must be unset or the same, so this is fine
            original.root_mount_point
        };
        
        let mut merged_config = original.clone();

        merged_config.maps.append(&mut new_config.maps);
        
        if !new_config.apps.is_empty() {
            merged_config.apps.append(&mut new_config.apps);
        }

        merged_config.root_mount_point = root;
        merged_config.sanitize();

        merged_config
    } else {
        new_config
    };

    
    if manual_force_write_config(&path, target_config.clone()) {
        Ok(target_config)
    } else {
        Err(None)
    }
}

/// Reads the config at the path you manually specified.
///
/// Manual forces you to make sure this is where the correct location, effectively:
/// C:\Users\[current]\AppData\Roaming\Datalink\config.json
/// Using the proton feature allows you to use proton-finder and skip finding the path
pub fn manual_read_config(path: &PathBuf) -> Option<GameBridgeConfig> {
    let conf = fs::read_to_string(path.as_path()).ok()?;
    serde_json::from_str(&conf).ok()
}
