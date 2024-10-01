use dbus::{blocking::Connection, channel::Sender, Message};

const INTERFACE_NAME:&str = "com.github.lukaslichten.datalink";
const PATH_NAME:&str = "/com/github/lukaslichten/datalink";

const PLAYING_SINGAL:&str = "StartedPlaying";
const STOPPED_SINGAL:&str = "StoppedPlaying";

pub(crate) fn set_playing(game_name: String) -> Option<()> {
    send_state(game_name, PLAYING_SINGAL)
}

pub(crate) fn unset_playing(game_name: String) -> Option<()> {
    send_state(game_name, STOPPED_SINGAL)
}

/// Sends signal of name state with playload name
fn send_state(game_name: String, state: &str) -> Option<()> {
    let c = Connection::new_session().ok()?;

    let msg = Message::new_signal(PATH_NAME, INTERFACE_NAME, state).ok()?;
    let msg = msg.append1(game_name);

    c.send(msg).ok()?;

    Some(())
}

// use dbus-monitor for debugging


// Example code of using dbus crate for receiving these messages
//
// let _ = c.add_match(MatchRule::new()
//     .with_type(dbus::MessageType::Signal)
//     .with_member("StartedPlaying")
//     .with_interface("com.github.lukaslichten.datalink")
// , |h: GameStatusStruct, _: &Connection, _: &Message| {
//     println!("Hello happened from sender: {}", h.sender);
//     true
// });
//
// struct GameStatusStruct must implement ReadAll

