//! This serves for the setting of env variables for further execution.  
//! This is useful for racing games, where SDL_JOYSTICK_DEVICE needs to be set so devices get
//! initialized reliably in the correct order.
//!
//! This is all achieved by going into ~/.config/Datalink/[gameid]/ and opening the env file 
//! Each line encodes one env variable

use std::{fs, path::PathBuf};

pub fn do_env(gameid: &str) -> Result<(), String> {
    let file = get_env_filepath(gameid).ok_or(format!("Failed to create/open ~/.config/Datalink/{} folder", gameid))?;

    if !file.exists() {
        // No env file found, continuing as normal
        return Ok(())
    }

    println!("Datalink has found an env file for game {}, applying...", gameid);
    let pairs = read_env_file(file)?;
    let count = pairs.len();

    for (key, value) in pairs {
        println!("{}={}",key.as_str(), value.as_str());

        unsafe {
            std::env::set_var(key, value);
        }
    }
    println!("Successfully applied {} Enviroment Vairables", count);
    Ok(())
}



fn get_env_filepath(gameid: &str) -> Option<PathBuf> {
    let mut buff = crate::get_config_folder()?;

    // This may need to be spun into it's own, in case we add more configuarable options
    buff.push(gameid);
    if !buff.exists() {
        fs::create_dir(buff.as_path()).ok()?;
    } else if !buff.is_dir() {
        return None;
    }

    buff.push("env");
    Some(buff)
}

fn read_env_file(file: PathBuf) -> Result<Vec<(String, String)>, String> {
    let mut pairs = Vec::<(String, String)>::new();

    let text = fs::read_to_string(file.as_path()).map_err(|e| e.to_string())?;

    let mut line_counter = 1; // Serves to provide debugging help
    for raw_l in text.lines() {
        line_counter += 1;
        let l = raw_l.trim_start();
        
        if l.strip_prefix("#").is_some() || l.strip_prefix("//").is_some() {
            // Comment line, ignoring
            continue;
        }

        if let Some((raw_key,raw_value)) = l.split_once('=') {
            let key = raw_key.trim_end().to_string();

            // An interesting conundrum: Should the end of the value be trimmed, afterall the
            // spaces could be intentional, but most likely aren't, as such we trim them
            // let value = raw_value.trim_start(); 
            let value = raw_value.trim().to_string(); 

            pairs.push((key, value));

        } else if l.is_empty() {
            // Ignoring Whitespace
            continue;
        } else {
            // Unable to parse value, reporting and exiting
            return Err(format!("Unable to parse line {}, does not adherre to format: {}", line_counter, raw_l));
        }

    }

    

    Ok(pairs)
}
