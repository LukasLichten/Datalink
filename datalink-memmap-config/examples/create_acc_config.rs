use datalink_memmap_config::{GameMemoryMapsConfig, MemMapConfig};

// Requires feature proton to be enabled, you can use this command:
// cargo run --all-features --example create_acc_config
fn main() {
    let drive = match proton_finder::get_game_drive(805550) {
        Ok(Some(d)) => d,
        Err(Some(d)) => d,
        _ => panic!("Could not find ACC prefix, is it installed?")
    };

    let config = GameMemoryMapsConfig { game_id: None, maps: vec![
        MemMapConfig { name: "acpmf_crewchief".to_string(), size: 15660 },
        MemMapConfig { name: "acpmf_static".to_string(), size: 2048 },
        MemMapConfig { name: "acpmf_physics".to_string(), size: 2048 },
        MemMapConfig { name: "acpmf_graphics".to_string(), size: 2048 },
    ] };

    let _ = dbg!( config.write_config(&drive) ); // LSP doesn't know we have the feature enabled
}
