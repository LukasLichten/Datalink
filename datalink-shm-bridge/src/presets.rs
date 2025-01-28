use datalink_bridge_config::{GameBridgeConfig, MemMapConfig};

use crate::built_info;


pub(super) fn get_preset(game: &str) -> Option<GameBridgeConfig> {
    let (mmaps, game) = match game {
        "805550" => (vec![
            MemMapConfig { name: "acpmf_static".to_string(), size: 2048 },
            MemMapConfig { name: "acpmf_physics".to_string(), size: 2048 },
            MemMapConfig { name: "acpmf_graphics".to_string(), size: 2048 },
        ], "Assetto Corsa Competizione"),
        "3058630" => (vec![
            MemMapConfig { name: "acpmf_static".to_string(), size: 2048 },
            MemMapConfig { name: "acpmf_physics".to_string(), size: 2048 },
            MemMapConfig { name: "acpmf_graphics".to_string(), size: 2048 },
        ], "Assetto Corsa Evo"),
        "244210" => (vec![
            MemMapConfig { name: "acpmf_crewchief".to_string(), size: 15660 },
            MemMapConfig { name: "acpmf_static".to_string(), size: 2048 },
            MemMapConfig { name: "acpmf_physics".to_string(), size: 2048 },
            MemMapConfig { name: "acpmf_graphics".to_string(), size: 2048 },
        ], "Assetto Corsa"),
        "378860" => (vec![
            MemMapConfig { name: "$pcars2$".to_string(), size: 102288 },
        ], "Project Cars 2"),
        "1066890" => (vec![
            MemMapConfig { name: "$pcars2$".to_string(), size: 102288 },
        ], "Automobilista 2"),
        "365960" => (vec![
            MemMapConfig { name: "$rFactor2SMMP_Scoring$".to_string(), size: 50000 },
            MemMapConfig { name: "$rFactor2SMMP_Telemetry$".to_string(), size: 50000 },
        ], "rFactor 2"),
        _ => return None
    };

    let conf = GameBridgeConfig::default().with_memory_maps(mmaps)
        .with_notes(format!("Datalink v{}.{}.{} default config for {}, do Not modify this file (you can copy it to creat your own). Rename the ending/Delete this file to disable it", 
            built_info::PKG_VERSION_MAJOR, built_info::PKG_VERSION_MINOR, built_info::PKG_VERSION_PATCH, game));

    Some(conf)
}
