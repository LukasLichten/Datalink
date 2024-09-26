use std::{fs, path::PathBuf};


use serde::{Serialize, Deserialize};

#[cfg(test)]
mod test;

/// The definitions for memory maps for this specfic Prefix/Game.  
/// The game_id is usually read from the AppID in the steam launch command,
/// but can be overwritten here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMemoryMapsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,

    pub maps: Vec<MemMapConfig>

}

impl GameMemoryMapsConfig {
    /// Removes dublicate memory maps (keeping the larger one)
    /// This is done by keeping the larger maps
    pub fn sanitize(&mut self) {
        self.maps.sort_by(|a,b| b.size.cmp(&a.size));

        let mut cached_names = Vec::<String>::with_capacity(self.maps.len());
        let mut index = 0;

        while let Some(item) = self.maps.get(index) {
            if cached_names.contains(&item.name) {
                self.maps.remove(index);
            } else {
                cached_names.push(item.name.clone());
                index += 1;
            }
        }
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


/// Aquires the config (for this prefix) from
/// C:\Users\[current]\AppData\Roaming\Datalink\config.json
#[cfg(target_os = "windows")]
pub fn read_config() -> Option<GameMemoryMapsConfig> {
    let mut path = dirs::config_dir()?;
    path.push("Datalink");
    path.push("config.json");

    let conf = fs::read_to_string(path).ok()?;
    serde_json::from_str(&conf).ok()
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
pub fn manual_force_write_config(path: &PathBuf, config: GameMemoryMapsConfig) -> bool {
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
///
/// Manual forces you to make sure it is written where the Datalink bridge can read it, effectively: 
/// C:\Users\[current]\AppData\Roaming\Datalink\config.json
/// Using the proton feature allows you to use proton-finder and skip finding the path
pub fn manual_write_config(path: &PathBuf, mut new_config: GameMemoryMapsConfig) -> Result<GameMemoryMapsConfig, Option<GameMemoryMapsConfig>> {
    let target_config = if let Some(original) = manual_read_config(path) {
        // There is already a config, we are merging them
        if new_config.game_id != original.game_id {
            // But not if the name disagree
            return Err(Some(original));
        }
        
        let mut merged_config = original.clone();

        merged_config.maps.append(&mut new_config.maps);
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
pub fn manual_read_config(path: &PathBuf) -> Option<GameMemoryMapsConfig> {
    let conf = fs::read_to_string(path.as_path()).ok()?;
    serde_json::from_str(&conf).ok()
}
