use std::{fs, os::unix::process::CommandExt, path::PathBuf};

mod dbus_handler;

fn main() {
    let mut args = std::env::args();

    let (exe, exec_override, user_debug) = if let Some(args) = handle_instructions(&mut args) {
        args
    } else {
        return;
    };
    println!("Launching with Datalink...");

    // for arg in std::env::args() {
    //     println!("{arg}");
    // }
    // println!("Done");

    let mut cmd = std::process::Command::new(exe);

    let mut gameid = String::new(); // We pass it anyway as a parameter, might as well be string
    let mut is_proton = false;

    while let Some(item) = args.next() {
        // Scanning for info and manipulating command
        if let Some(id) = item.strip_prefix("AppId=") {
            gameid = id.to_string();
        }

        if item == "waitforexitandrun" {
            // We could also check if previously proton was launched, but we need to anyway find
            // this, as it preceeds the exe and launch parameters
            cmd.arg(item);

            is_proton = true;

            if let Some((this_exec, bridge_exec)) = inject_bridge(user_debug) {
                cmd.arg(bridge_exec);

                cmd.arg(this_exec);
                cmd.arg(gameid.as_str());
                
                if let Some(over) = exec_override.as_ref() {
                    cmd.arg(over.clone());
                } else if let Some(game_exec) = args.next() {
                    cmd.arg(game_exec);
                } else {
                    panic!("Error one deploying bridge, no game exec given");
                }
            } else {
                panic!("Error on deploying bridge");
                // Is a panic overkill? Yes. But letting the launch continue would lead to a
                // degraded state, that the user could not easily track down
            }

        } else {
            // regular append
            cmd.arg(item);
        }
    }

    if is_proton {
        println!("Datalink prep for game {gameid} finished, switching into Proton...");
        let err = cmd.exec();
        panic!("Failed to launch proton: {}", err.to_string())
    } else {
        println!("Launching native Game {} as child", gameid.as_str());
        if let Ok(mut run) = cmd.spawn() {
            let file_opt = get_runningfile_path(gameid.as_str());


            // Send message on dbus and set running file
            if let Some(f) = file_opt.as_ref() {
                // We use this wrapper as the pid
                let _ = fs::write(f, std::process::id().to_string());
            }
            dbus_handler::set_playing(gameid.clone());


            // Running the game
            let res = run.wait();


            // Game exited, deleting running file and sending dbus message
            if let Some(f) = file_opt {
                let _ = fs::remove_file(f);
            }
            dbus_handler::unset_playing(gameid.clone());


            match res { 
                Err(e) => panic!("Exiting Datalink due to game crash:\n{e}"),
                Ok(_) => println!("Game shutdown, exiting Datalink")
            }
        } else {
            panic!("Failed to start game");
        }

    }
        
}

/// The bridge will use our programm to handle dbus and other resources,
/// so we will check if the parameter is an instruction, or an exec,
/// and we hand the exec back for execution
fn handle_instructions(args: &mut std::env::Args) -> Option<(String, Option<String>, bool)> {
    if let Some(instr) = args.nth(1) {
        match instr.as_str() {
            "--help" => print_help(),
            "--set-playing" => {
                let game = args.next()?; // technically should error, but this is enough
                
                // Even if writing the cache file fails, we will still send the dbus message
                if let Some(file) = get_runningfile_path(game.as_str()) {

                    // When called like this we use the parent process
                    // which when the call came (as expected) from the bridge.exe
                    // will not be the bridge, but instead the pressure-vessel/wine
                    fs::write(file, std::os::unix::process::parent_id().to_string()).ok()?;
                }

                dbus_handler::set_playing(game)?;
            },
            "--unset-playing" => {
                let game = args.next()?;

                // Even if deleting the cache file fails, we will still send the dbus message
                if let Some(file) = get_runningfile_path(game.as_str()) {
                    if file.exists() {
                        let _ = fs::remove_file(file);
                    }
                }

                dbus_handler::unset_playing(game)?;
            },
            "-D" => return handle_debug_flag(args),
            "-d" => return handle_debug_flag(args),
            "--debug" => return handle_debug_flag(args),
            "--override" => return exec_override(args, false),
            "-O" => return exec_override(args, false),
            "-o" => return exec_override(args, false),
            _ => return Some((instr, None, false))
        }
    } else {
        print_help();
    }

    None
}

fn check_if_debug_allowed() -> bool {
    if cfg!(feature = "include-debug") {
        true
    } else {
        println!("You enbaled debug but your packager did not include the debug exe. Please recompile with feature 'include-debug' enabled");
        false
    }
}

fn handle_debug_flag(args: &mut std::env::Args) -> Option<(String, Option<String>, bool)> {
    let user_debug = check_if_debug_allowed();

    let instr = args.next()?;

    match instr.as_str() {
        "--override" => exec_override(args, user_debug),
        "-O" => exec_override(args, user_debug),
        "-o" => exec_override(args, user_debug),
        _ => Some((instr, None, user_debug))
    }
}

fn exec_override(args: &mut std::env::Args, user_debug: bool) -> Option<(String, Option<String>, bool)> {
    let over = args.next()?;
    let instr = args.next()?;

    // late debug flag
    let (user_debug, instr) = match instr.as_str() {
        "--debug" => (check_if_debug_allowed(), args.next()?),
        "-d" => (check_if_debug_allowed(), args.next()?),
        "-D" => (check_if_debug_allowed(), args.next()?),
        _ => (user_debug, instr)
    };

    Some((instr, Some(over), user_debug))
}

fn print_help() {
    let debugging_help = if cfg!(feature = "include-debug") {
"
If you run into issues (Memory Maps not deploying etc) then use the -d flag like this:
Datalink -d %command%
"        
    } else {
        ""
    };

    println!("Datalink is a command wrapper that notifies
when the wrapped programm is run and when it exits.
If the wrapped command is a Proton launch it will deploy a wrapper into the prefix,
which will also map the Windows Shared Memory Maps to /dev/shm.
https://github.com/LukasLichten/Datalink

Standard usage is setting the Launch Option on Steam to:
Datalink %command% 

You can override the Program that should be used (launching a mod manager for example) using:
Datalink -O /full/path/to/exec %command%
-o, -O, --override are all valid
{}
Generally, if you want to modify the settings you can check within the prefix the folder:
/drive_c/users/steamuser/AppData/Roaming/Datalink/
Changing the file ending away from json will disable them, further instructions on editing can be found here:
https://github.com/LukasLichten/Datalink?tab=readme-ov-file#configuring-the-bridge
", debugging_help);
}

fn get_runningfile_path(game: &str) -> Option<PathBuf> {
    let mut path = get_cache_folder()?;
    path.push("running");

    if !path.exists() {
        fs::create_dir(path.as_path()).ok()?;
    } else if !path.is_dir() {
        return None;
    }

    path.push(game);
    Some(path)
}

/// This folder is ~/.cache/Datalink
/// If it doesn't exist we create it, if all that fails None is returned
fn get_cache_folder() -> Option<PathBuf> {
    let mut buff = dirs::cache_dir()?;
    buff.push("Datalink");

    if !buff.exists() {
        fs::create_dir(buff.as_path()).ok()?;
    } else if !buff.is_dir() {
        return None;
    }

    Some(buff)
}

fn place_bridge_exe(user_debug: bool) -> Option<String> {
    let mut path = get_cache_folder()?;

    let res = if user_debug {
        path.push("datalink-shm-bridge-debug.exe");
        let win_exec = {
            // Build release, no debug feature. Case should not happen, but return release anyway
            #[cfg(all(not(debug_assertions), not(feature = "include-debug")))]
            { include_bytes!(concat!(env!("OUT_DIR"), "/../../../../x86_64-pc-windows-gnu/release/datalink-shm-bridge.exe")) }

            // Build release with debug feature, or debug build. Return debug
            #[cfg(any(debug_assertions, feature = "include-debug"))]
            { include_bytes!(concat!(env!("OUT_DIR"), "/../../../../x86_64-pc-windows-gnu/debug/datalink-shm-bridge.exe")) }
        };
        fs::write(path.as_path(), win_exec)
    } else {
        path.push("datalink-shm-bridge.exe");
        let win_exec = {
            // Build release, return release
            #[cfg(not(debug_assertions))]
            { include_bytes!(concat!(env!("OUT_DIR"), "/../../../../x86_64-pc-windows-gnu/release/datalink-shm-bridge.exe")) }

            // Build debug, return debug
            #[cfg(debug_assertions)]
            { include_bytes!(concat!(env!("OUT_DIR"), "/../../../../x86_64-pc-windows-gnu/debug/datalink-shm-bridge.exe")) }
        };
        fs::write(path.as_path(), win_exec)
    };

    if res.is_err() {
        // Failed to write the exe, but maybe there is already an exe we can use?
        if !path.exists() {
            // Nope, that failed too, aborting
            return None;
        }
    }
    
    // We don't need to convert the path to the windows path, as proton just takes the linux path
    // and does it itself

    path = path.canonicalize().ok()?;
    Some(path.to_str()?.to_string())
}

fn inject_bridge(user_debug: bool) -> Option<(String, String)> {
    Some((
        std::env::current_exe().ok()?.canonicalize().ok()?.to_str()?.to_string(),
        place_bridge_exe(user_debug)?
    ))
}
