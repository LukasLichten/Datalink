use datalink_bridge_config::{GameBridgeConfig, MemMapConfig};

// Requires feature proton to be enabled, you can use this command:
// cargo run --all-features --example create_acc_config
fn main() {
    let drive = match proton_finder::get_game_drive(805550) {
        Ok(Some(d)) => d,
        Err(Some(d)) => d,
        _ => panic!("Could not find ACC prefix, is it installed?")
    };

    let config = GameBridgeConfig::default().with_memory_maps(vec![
        MemMapConfig { name: "acpmf_crewchief".to_string(), size: 15660 },
        MemMapConfig { name: "acpmf_static".to_string(), size: 2048 },
        MemMapConfig { name: "acpmf_physics".to_string(), size: 2048 },
        MemMapConfig { name: "acpmf_graphics".to_string(), size: 2048 },
    ]);

    let _ = dbg!( config.write_config(&drive, "com.github.lukaslichten.datalink.test", true) ); // LSP doesn't know we have the feature enabled
}
