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
            // $rFactor2SMMP_Telemetry$ 241680
            // $rFactor2SMMP_Scoring$ 75304
            // $rFactor2SMMP_Rules$ 45264
            // $rFactor2SMMP_MultiRules$ 39788
            // $rFactor2SMMP_ForceFeedback$ 16
            // $rFactor2SMMP_Graphics$ 272
            // $rFactor2SMMP_PitInfo$ 340
            // $rFactor2SMMP_Weather$ 632
            // $rFactor2SMMP_Extended$ 10152
            MemMapConfig { name: "$rFactor2SMMP_Telemetry$".to_string(), size: 241680 },
            MemMapConfig { name: "$rFactor2SMMP_Scoring$".to_string(), size: 75304 },
            MemMapConfig { name: "$rFactor2SMMP_Rules$".to_string(), size: 45264 },
            MemMapConfig { name: "$rFactor2SMMP_MultiRules$".to_string(), size: 39788 },
            MemMapConfig { name: "$rFactor2SMMP_ForceFeedback$".to_string(), size: 16 },
            MemMapConfig { name: "$rFactor2SMMP_Graphics$".to_string(), size: 272 },
            MemMapConfig { name: "$rFactor2SMMP_PitInfo$".to_string(), size: 340 },
            MemMapConfig { name: "$rFactor2SMMP_Weather$".to_string(), size: 632 },
            MemMapConfig { name: "$rFactor2SMMP_Extended$".to_string(), size: 10152 },
        ], "rFactor 2"),
        // SCSTelemetry 21619
        "227300" => (vec![
            MemMapConfig { name: "SCSTelemetry".to_string(), size: 21619 }
        ], "Euro Truck Simulator 2"),
        "270880" => (vec![
            MemMapConfig { name: "SCSTelemetry".to_string(), size: 21619 }
        ], "American Truck Simulator"),
        "211500" => (vec![
            // $R3E 39320
            MemMapConfig { name: "$R3E".to_string(), size: 39320 }
        ], "RaceRoom Racing Expierence"),
        _ => return None
    };

    let conf = GameBridgeConfig::default().with_memory_maps(mmaps)
        .with_notes(format!("Datalink v{}.{}.{} default config for {}, do Not modify this file (you can copy it to create your own). Rename the ending/Delete this file to disable it", 
            built_info::PKG_VERSION_MAJOR, built_info::PKG_VERSION_MINOR, built_info::PKG_VERSION_PATCH, game));

    Some(conf)
}
