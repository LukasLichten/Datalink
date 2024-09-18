use std::{fs, os::unix::process::CommandExt, path::PathBuf};

fn main() {
    println!("Launching with Datalink...");
    let mut args = std::env::args();

    let exe = if let Some(exe) = handle_instructions(&mut args) {
        exe
    } else {
        return;
    };

    // for arg in std::env::args() {
    //     println!("{arg}");
    // }
    // println!("Done");

    let mut cmd = std::process::Command::new(exe);

    let mut gameid = String::new(); // We pass it anyway as a parameter, might as well be string
    let mut is_proton = false;

    for item in args {
        // Scanning for info and manipulating command
        if let Some(id) = item.strip_prefix("AppId=") {
            gameid = id.to_string();
        }

        if item == "waitforexitandrun" {
            // We could also check if previously proton was launched, but we need to anyway find
            // this, as it preceeds the exe and launch parameters
            cmd.arg(item);

            is_proton = true;

            if let Some((this_exec, bridge_exec)) = inject_bridge() {
                cmd.arg(bridge_exec);

                cmd.arg(this_exec);
                cmd.arg(gameid.as_str());
                
                
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

            // Todo dbus start

            let res = run.wait();

            // Todo dbus start

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
fn handle_instructions(args: &mut std::env::Args) -> Option<String> {
    if let Some(instr) = args.nth(1) {
        match instr.as_str() {
            "--help" => print_help(),

            _ => return Some(instr)
        }
    } else {
        print_help();
    }

    None
}

fn print_help() {
    println!("Datalink is a command wrapper that notifies\n
        when the wrapped programm is run and when it exits.\n
        If the wrapped command is a Proton launch it will deploy a wrapper into the prefix,\n
        which will also map the Windows Shared Memory Maps to /dev/shm.\n
        \n
        Standard usage is setting the Launch Option on Steam to:\n
        Datalink %command%
        ");
}

/// This folder is ~/.cache/Datalink
/// If it doesn't exist we create it, if all that fails None is returned
fn get_cache_folder() -> Option<PathBuf> {
    let mut buff = dirs::cache_dir()?;
    buff.push("Datalink");

    if !buff.exists() {
        fs::create_dir(buff.as_path()).ok()?;
    }

    Some(buff)
}

fn place_bridge_exe() -> Option<String> {
    #[cfg(not(debug_assertions))]
    let win_exec = include_bytes!("../../target/x86_64-pc-windows-gnu/release/datalink-shm-bridge.exe");

    #[cfg(debug_assertions)]
    let win_exec = include_bytes!("../../target/x86_64-pc-windows-gnu/debug/datalink-shm-bridge.exe");


    let mut path = get_cache_folder()?;
    path.push("datalink-shm-bridge.exe");

    if fs::write(path.as_path(), win_exec).is_err() {
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

fn inject_bridge() -> Option<(String, String)> {
    Some((
        std::env::current_exe().ok()?.canonicalize().ok()?.to_str()?.to_string(),
        place_bridge_exe()?
    ))
}
