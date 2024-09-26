use crate::{GameMemoryMapsConfig, MemMapConfig};

const GAME_ID:u32 = 2420510;

#[test]
#[cfg(feature = "proton")]
pub fn find_path() {
    let drive = match proton_finder::get_game_drive(GAME_ID) {
        Ok(Some(d)) => d,
        Err(Some(d)) => d,
        _ => panic!("Failed to get GameDrive for {}, is it installed?", GAME_ID.to_string())
    };


    assert!(crate::get_path_from_prefix(&drive).is_some(), "Failed to get path to the config file for {}", GAME_ID.to_string());
}

#[test]
pub fn sanitize_game_mappings_conf() {
    let mut sample = GameMemoryMapsConfig { game_id: None, maps: vec![
        MemMapConfig { name: "test".to_string(), size: 512 },
        MemMapConfig { name: "hello".to_string(), size: 361 },
        MemMapConfig { name: "Test".to_string(), size: 2048 },
        MemMapConfig { name: "test".to_string(), size: 1000 }
    ] };

    sample.sanitize();

    assert_eq!(sample.maps.len(), 3, "Did not remove the expected amount of lines, Map is: {:?}", sample.maps);

    assert_eq!(sample.maps[0].name, "Test", "Unexpected Name for row 1, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[0].size, 2048, "Unexpected Size for row 1, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[1].name, "test", "Unexpected Name for row 2, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[1].size, 1000, "Unexpected Size for row 2, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[2].name, "hello", "Unexpected Name for row 3, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[2].size, 361, "Unexpected Size for row 3, Full Map is: {:?}", sample.maps);
}
