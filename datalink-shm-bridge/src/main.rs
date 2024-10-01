use datalink_memmap_config::GameMemoryMapsConfig;
use mmap::FileMapping;

mod mmap;

fn main() {
    // Currently a shell is spawned for this tool being launched
    print!("Datalink Bridge for game ");

    // for item in std::env::args() {
    //     println!("{item}");
    // }

    let mut args = std::env::args();
    
    let callback = convert_linux_path_to_wine(args.nth(1)).unwrap();
    let game_id = args.next().unwrap();

    // Generating game calle
    let game_exe = convert_linux_path_to_wine(args.next()).unwrap();
    let mut cmd = std::process::Command::new(game_exe);

    for item in args {
        cmd.arg(item);
    }

    
    let (game_id, maps) = match datalink_memmap_config::read_config() { // The LSP pretends the function does not exist
        Ok(Some(config)) => {
            let config: GameMemoryMapsConfig = config; // We can at least code with this still

            let game_id = if let Some(alter) = config.game_id {
                alter
            } else {
                game_id
            };

            println!("{} starting...", game_id.as_str());

            let tmpfs = match mmap::get_tmpfs_mountpoint() {
                Some(p) => p,
                None => error_exit("Unable to find /dev/shm through the wine prefix")
            };
            let mut maps = Vec::<FileMapping>::with_capacity(config.maps.len());

            for item in config.maps {
                match mmap::create_file_mapping(tmpfs.clone(), item.name.as_str(), item.size) {
                    Ok(map) => {
                        maps.push(map);
                        println!("Created MemoryMap {} with size {} successfully", item.name, item.size);
                    },
                    Err(e) => {
                        drop(maps); // Cleanup already created maps
                        error_exit(format!("Failed to create memory map {}: {}", item.name, e).as_str())
                    }
                }
            }
            

            (game_id, maps)
        },
        Ok(None) => {
            println!("{} starting...", game_id.as_str());

            println!("No Config File Found!");
            println!("No Memory Maps will be deployed, dbus will still be notified!");

            (game_id, Vec::<FileMapping>::new())
        },
        Err(e) => {
            println!("{} starting...", game_id.as_str());

            // Should we crash on this?

            println!("Failed to read Config File:");
            println!("{e}");
            println!("No Memory Maps will be deployed, dbus will still be notified!");

            (game_id, Vec::<FileMapping>::new())
        }
    };
    
    // Pre-Game dbus message
    send_dbus(callback.as_str(), "--set-playing", game_id.as_str());


    // Launching the game
    println!("DO NOT CLOSE THIS WINDOW!");
    let mut pro = match cmd.spawn() {
        Ok(pro) => pro,
        Err(e) => error_exit(format!("Failure to launch game: {e}").as_str())
    };

    // ctrlc::set_handler(|| {
    //     
    //     println!("Termination requested");
    //
    // }).expect("Interrupt handler should never fail to be created");
    
    let _ = pro.wait();
    

    
    // Game closed, wrapping up
    println!("Datalink Bridge shutdown (window should close now)");
    send_dbus(callback.as_str(), "--unset-playing", game_id.as_str());
    drop(maps); // Maps and their files are cleaned up on drop

    // std::thread::sleep(std::time::Duration::from_secs(5));
}

fn convert_linux_path_to_wine(path: Option<String>) -> Option<String> {
    let p = path?.replace('/', "\\");
    // TODO: Is alternative mount point for linux root a thing? Or does wine always have it on Z?
    let complete = "Z:".to_string() + &p;

    Some(complete)
}

fn error_exit(msg: &str) -> ! {
    println!("Fatal error occured: {msg}");
    println!("Exiting...");
    
    // User read time
    std::thread::sleep(std::time::Duration::from_secs(5));

    std::process::exit(1)
}

fn send_dbus(callback: &str, op: &str, game_id: &str) {

    // Yes, we are launching a linux process from wine...
    // Apparently wine when calling CreateProcess on a elf-linux will
    // not fail, but instead make wine launch it as a linux process
    let mut cmd = std::process::Command::new(callback);
    cmd.arg(op);
    cmd.arg(game_id);


    // However, as such the child handle is useless, trying to wait on it gives and invalid handle
    // error, so we never know if it succeeded, but we just hope
    let _ = cmd.spawn();
}

