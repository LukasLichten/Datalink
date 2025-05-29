//! Unfortunatly some games (Madness Engine, aka AMS2, PCars 2, etc.) can act a bit funny, and close the launch process.
//! In that case we need to insure the game has closed and is not magically running in the
//! background still.

use std::{path::{Path, PathBuf}, str::FromStr, time::Duration};

use sysinfo::{self, Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

const POLL_RATE: Duration = Duration::from_secs(5);

pub fn poll_game(game_exe: String) -> Option<()> {
    println!("The initial Process of the game has closed, falling back to polling...");

    let mut path = PathBuf::from_str(game_exe.as_str()).ok()?;
    path.pop();
    let folder = path.as_path();

    let mut system = System::new();
    let mut pid: Option<Pid> = None;
    loop {
        pid = check_for_program_running(&mut system, pid, folder);
        
        if pid.is_none() {
            println!("No Game Process can be found anymore");
            return Some(());
        }

        std::thread::sleep(POLL_RATE);
    }
}

/// This checks if a certain process (from a certain folder) is running, and retrieves the pid.
/// Passing in the previous PID will cut down on having to load and search through all processes.
/// But in case this process is no longer running or a different process, it will do a full check
/// for another process.
/// PID is determined on with find and contains on the cmdline, meaning if multiple processes are
/// running the same (or similar) then the first PID is grabbed
fn check_for_program_running(system: &mut System, pid: Option<Pid>, folder: &Path) -> Option<Pid> {
    if let Some(pid) = &pid {
        // We know a pid which it ran with, so we just gather info on this one
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[*pid]), 
            false, 
            ProcessRefreshKind::nothing()
                .with_exe(UpdateKind::Always)
        );

        if let Some(pro) = system.process(*pid) {
            if let Some(path) = pro.exe() {
                if path.starts_with(folder) && pro.exists() {
                    return Some(pro.pid());
                }
            }

            // There is a foreign process running under our ID, we retry
            #[cfg(debug_assertions)]
            println!("Process closed/switched?");
            return check_for_program_running(system, None, folder);
        }

        // Our process is gone, but don't worry, maybe he is still out there under another pid
        return check_for_program_running(system, None, folder);
    } else {
        // We scan all processes
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_exe(UpdateKind::OnlyIfNotSet)
        );

        if let Some(pro) = system.processes().values().find(|val| {
            if let Some(path) = val.exe() {
                path.starts_with(folder)
            } else {
                false
            }
        }) {
            return Some(pro.pid());
        }
    }
    
    None
}
