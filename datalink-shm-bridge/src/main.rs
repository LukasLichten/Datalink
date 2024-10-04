use datalink_bridge_config::GameBridgeConfig;
use mmap::FileMapping;

mod mmap;

fn main() {
    // Currently a shell is spawned for this tool being launched
    print!("Datalink Bridge for game ");

    // for item in std::env::args() {
    //     println!("{item}");
    // }

    let mut args = std::env::args();
    
    let callback = expect_exit(args.nth(1), "Missing argument, expected callback path");
    let game_id = expect_exit(args.next(), "Missing argument, expected game_id");

    let game_exe = expect_exit(args.next(), "Missing argument, expected game executable");
    
    // Reading the config
    let (callback, game_exe, game_id, maps, apps) = match datalink_bridge_config::read_config() { // The LSP pretends the function does not exist
        Ok(Some(config)) => {
            let config: GameBridgeConfig = config; // We can at least code with this still

            let game_id = if let Some(alter) = config.game_id.as_ref() {
                alter.clone()
            } else {
                game_id
            };

            println!("{} starting...", game_id.as_str());


            // Memory maps
            let tmpfs = match mmap::get_tmpfs_mountpoint(config.get_root_mount_point()) {
                Some(p) => p,
                None => error_exit("Unable to find /dev/shm through the wine prefix")
            };
            let mut maps = Vec::<FileMapping>::with_capacity(config.maps.len());

            for item in &config.maps {
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

            // Apps
            let root = config.get_root_mount_point();
            let mut apps = Vec::<std::process::Child>::with_capacity(config.apps.len());

            for item in config.apps {
                match start_side_app(root, item) {
                    Ok(c) => apps.push(c),
                    Err(e) => {
                        // Cleanup already created maps
                        drop(maps);
                        close_apps(apps);

                        error_exit(e.as_str());
                    }
                }


            }

            (convert_linux_path(root, callback), convert_linux_path(root, game_exe), game_id, maps, apps)
        },
        Ok(None) => {
            println!("{} starting...", game_id.as_str());

            println!("No Config File Found!");
            println!("No Memory Maps and Apps will be deployed, dbus will still be notified!");

            let root = datalink_bridge_config::GameBridgeConfig::default().get_root_mount_point();

            (convert_linux_path(root, callback), convert_linux_path(root, game_exe), game_id, Vec::<FileMapping>::new(), Vec::<std::process::Child>::new())
        },
        Err(e) => {
            println!("{} starting...", game_id.as_str());

            // Should we crash on this?

            println!("Failed to read Config File:");
            println!("{e}");
            println!("No Memory Maps and Apps will be deployed, dbus will still be notified!");

            let root = datalink_bridge_config::GameBridgeConfig::default().get_root_mount_point();

            (convert_linux_path(root, callback), convert_linux_path(root, game_exe), game_id, Vec::<FileMapping>::new(), Vec::<std::process::Child>::new())
        }
    };

    // Generating game calle
    let mut cmd = std::process::Command::new(game_exe);

    for item in args {
        cmd.arg(item);
    }
    
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
    println!("Datalink Bridge shutting down...");
    send_dbus(callback.as_str(), "--unset-playing", game_id.as_str());
    drop(maps); // Maps and their files are cleaned up on drop
    if !apps.is_empty() {
        println!("Terminating auxilary apps...");
        if close_apps(apps) {
            
        }
    }

    println!("Shutdown finished, window should close now");


    // std::thread::sleep(std::time::Duration::from_secs(5));
}

fn convert_linux_path(drive_letter: char, path: String) -> String {
    // The LSP pretends the function does not exist
    // But it does under windows, for which we compile it
    datalink_bridge_config::convert_linux_path_to_wine(drive_letter, path)
}

/// Starts another app on the side
fn start_side_app(root: char, app: datalink_bridge_config::App) -> Result<std::process::Child, String> {
    let name = app.get_name().to_string();

    let mut cmd: std::process::Command = app.get_command(root).ok_or(format!("Failed to generate command for App {name}"))?;

    let child = cmd.spawn().map_err(|e|  format!("Failed to spawn process for App {name}: {}", e.to_string()))?;
    println!("Successfully launched App {name}");
    Ok(child)
}


/// Tries to close the children, but if one won't we try the others and return false to signal not
/// complete (but likely sufficient cleanup)
fn close_apps(mut apps: Vec<std::process::Child>) -> bool {
    let mut clean = true;
    let mut closed = true;

    for item in apps.iter_mut() {
        match item.try_wait() {
            Ok(Some(_)) => (),
            Ok(None) => {
                let mut killer = std::process::Command::new("taskkill");
                killer.arg("/pid");
                killer.arg(item.id().to_string());

                let _ = killer.spawn();
                closed = false;
            },
            Err(e) => {
                let _ = e;
                clean = false;
            }
        }
    }


    // Waiting for gracefull termination
    for _ in 0..20 {
        if closed {
            return clean;
        }

        closed = true;
        let start = std::time::Instant::now();

        for item in apps.iter_mut() {
            match item.try_wait() {
                Ok(None) => {
                    closed = false;
                    break;
                },
                _ => ()
            }
        }

        if let Some(time) = std::time::Duration::from_millis(250).checked_sub(std::time::Instant::now() - start) {
            std::thread::sleep(time);
        }
    }
    
    // Forcefull termination
    for item in apps.iter_mut() {
        match item.try_wait() {
            Ok(None) => {
                println!("A app reached timeout for graceful shutdown, forcefull shutdown used");

                let mut killer = std::process::Command::new("taskkill");
                killer.arg("/f");
                killer.arg("/pid");
                killer.arg(item.id().to_string());

                let _ = killer.spawn();
            },
            _ => ()
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(2));


    clean
}

/// Unwraps value with our error handler
fn expect_exit<T>(value: Option<T>, msg: &str) -> T {
    if let Some(res) = value {
        res
    } else {
        error_exit(msg);
    }
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

