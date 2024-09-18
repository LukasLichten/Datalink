fn main() {
    // Cureently a shell is spawned for this tool being launched
    // println!("Bridge deployed:");

    // for item in std::env::args() {
    //     println!("{item}");
    // }

    let mut args = std::env::args();
    
    let _callback = convert_linux_path_to_wine(args.nth(1)).unwrap();
    let _gameid = args.next().unwrap();

    // Generating game calle
    let game_exe = convert_linux_path_to_wine(args.next()).unwrap();
    let mut cmd = std::process::Command::new(game_exe);

    for item in args {
        cmd.arg(item);
    }

    // TODO load config for memory maps
    // TODO deploy memory maps
    // TODO send dbus notification

    let _ = cmd.spawn().unwrap().wait();

    // TODO send dbus notification about shutdown
    // TODO and we need to cleanup what we launched

    // std::thread::sleep(std::time::Duration::from_secs(5));
    // println!("Bridge destroyed");
}

fn convert_linux_path_to_wine(path: Option<String>) -> Option<String> {
    let p = path?.replace('/', "\\");
    let complete = "Z:".to_string() + &p;

    Some(complete)
}
