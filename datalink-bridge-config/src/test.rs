use crate::{App, GameBridgeConfig, MemMapConfig};

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
    let mut sample = GameBridgeConfig::default().with_memory_maps(vec![
        MemMapConfig { name: "test".to_string(), size: 512 },
        MemMapConfig { name: "hello".to_string(), size: 361 },
        MemMapConfig { name: "Test".to_string(), size: 2048 },
        MemMapConfig { name: "test".to_string(), size: 1000 }
    ]);

    sample.sanitize();

    assert_eq!(sample.maps.len(), 3, "Did not remove the expected amount of lines, Map is: {:?}", sample.maps);

    assert_eq!(sample.maps[0].name, "Test", "Unexpected Name for row 1, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[0].size, 2048, "Unexpected Size for row 1, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[1].name, "test", "Unexpected Name for row 2, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[1].size, 1000, "Unexpected Size for row 2, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[2].name, "hello", "Unexpected Name for row 3, Full Map is: {:?}", sample.maps);
    assert_eq!(sample.maps[2].size, 361, "Unexpected Size for row 3, Full Map is: {:?}", sample.maps);
}

#[test]
pub fn sanitize_game_apps_conf() {
    let mut sample = GameBridgeConfig::default().with_autolaunch_apps(vec![
        App::new("C:\\users\\steamuser\\Documents\\test.exe".to_string()).unwrap().with_args(vec!["--".to_string()]),
        App::new("C:\\users\\steamuser\\Documents\\test.exe".to_string()).unwrap(),
        App::new("C:\\users\\steamuser\\Documents\\t.exe".to_string()).unwrap(),
        App::new("C:\\users\\steamuser\\Documents\\test.exe".to_string()).unwrap().with_args(vec!["--".to_string()]),
    ]);

    sample.sanitize();

    assert_eq!(sample.apps.len(), 3, "Did not remove the expected amount of lines, apps is: {:?}", sample.apps);

    assert_eq!(sample.apps[0].get_exec(), "C:\\users\\steamuser\\Documents\\test.exe", "Unexpected Name for row 1, Full apps is: {:?}", sample.apps);
    assert_eq!(sample.apps[0].get_args_as_ref().len(), 1, "Unexpected Size for row 1, Full apps is: {:?}", sample.apps);
    assert_eq!(sample.apps[1].get_exec(), "C:\\users\\steamuser\\Documents\\test.exe", "Unexpected Name for row 2, Full apps is: {:?}", sample.apps);
    assert_eq!(sample.apps[1].get_args_as_ref().len(), 0, "Unexpected Size for row 2, Full apps is: {:?}", sample.apps);
    assert_eq!(sample.apps[2].get_exec(), "C:\\users\\steamuser\\Documents\\t.exe", "Unexpected Name for row 3, Full apps is: {:?}", sample.apps);
    assert_eq!(sample.apps[2].get_args_as_ref().len(), 0, "Unexpected Size for row 3, Full apps is: {:?}", sample.apps);
}
