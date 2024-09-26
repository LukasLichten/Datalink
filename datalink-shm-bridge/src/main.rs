use datalink_memmap_config::GameMemoryMapsConfig;

fn main() {
    // Currently a shell is spawned for this tool being launched
    print!("Datalink Bridge for game ");

    // for item in std::env::args() {
    //     println!("{item}");
    // }

    let mut args = std::env::args();
    
    let _callback = convert_linux_path_to_wine(args.nth(1)).unwrap();
    let game_id = args.next().unwrap();

    // Generating game calle
    let game_exe = convert_linux_path_to_wine(args.next()).unwrap();
    let mut cmd = std::process::Command::new(game_exe);

    for item in args {
        cmd.arg(item);
    }

    let (game_id, maps) = if let Some(config) = datalink_memmap_config::read_config() { // The LSP pretends the function does not exist
        let config: GameMemoryMapsConfig = config; // We can at least code with this still

        let game_id = if let Some(alter) = config.game_id {
            alter
        } else {
            game_id
        };
        
        println!("{} starting...", game_id.as_str());




        (game_id, ())
    } else {
        println!("{} starting...", game_id.as_str());

        println!("No Config File Found!");
        println!("No Memory Maps will be deployed, dbus will still be notified!");
        (game_id, ())
    };

    // TODO deploy memory maps
    // TODO send dbus notification

    println!("Do Not Close This Window!");

    let _ = cmd.spawn().unwrap().wait();

    // TODO send dbus notification about shutdown
    // TODO and we need to cleanup what we launched

    // std::thread::sleep(std::time::Duration::from_secs(5));
    println!("Datalink Bridge shutdown (window should close now)");
}

fn convert_linux_path_to_wine(path: Option<String>) -> Option<String> {
    let p = path?.replace('/', "\\");
    let complete = "Z:".to_string() + &p;

    Some(complete)
}
