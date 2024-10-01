# Datalink
Simple wrapper around Steam/Proton Games that sends notification of their launch and deploys a Shared memory bridge

## Usage
Apppend Datalink infront of the launch command (same way as Mangohud) by setting the Launch Options on Steam like this:
```
Datalink %command%
```

You are free to combine this with Mangohud and or regular launch options.  
This works even with Linux native steam games, although no memorymaps will be setup for those (but the launch is still communicated).

## Programmatical Usage
For writing game tools this wrapper exposes resources (memory maps) and notifies when the game is launched (so you can start reading data).  
  
### Shared Memory And The Bridge
Some Windows games utilize Shared Memory Map (also known as Named Shared Memory, Shared Memory Page, FileMapping, etc.) to share telemetry etc.  
This produces an issue, as only windows software (and running in the same prefix and session) can access these maps.  
  
To allow linux native software to read telemetry (and also be informed when the game is running, as there is no good universal way of doing so)
the wrapper modifies the launch command of the game for proton to launch our bridge instead, which then launches the game.  

Prior to launching the game the bridge will map the Memory Maps (which are defined in it's config) to `/dev/shm` on the linux side.  
This way the bridge can sleep while the game is running, not taking up any processing time,
while the data is directly available on the linux side thourgh `shm_open`.  
  
Doing it this way circumnavigates numerous problems found in [Related Porjects](#related):
- complex launch parameters for end users
- software launched through flatpak protontricks not being able to interact with memory maps at all
- steam being unable to launch or fully close the game due to software launched through protontricks
- when using this method of making windows api/wine back the memorymaps to `/dev/shm` you have to create them before the game
- Otherwise you have to employ a software that constantly copied memory from the wine side to the linux side

### Configuring Memory Maps
The config is on a per Proton Prefix basis, located in `C:\users\steamuser\AppData\Roaming\Datalink\config.json`
(although username might be different if the prefix is configured differently).  
  
You can use the, present in this repository, config crate `datalink-memmap-config`,
which contains the structs and function for deserialzing the crate.  
Additionally with the `proton` feature you can use [proton-finder](https://github.com/LukasLichten/proton-finder)
crate to automatically find the prefix for the game and with it the config file.  
  
It is recommened to always use the write methods instead of force write methods, so multiple programs can contribute to the config file,
and all of them can read the data they want. If a map with the same name already exists, then the larger of the two is kept, all others are kept.  
If the game_id was set (by your config or the existing), then they need to match (and only one being set is also a missmatch).
  
In case of manual writing (e.g. other programming language), this is an example config:
```
{
    "game_id":"override"
    "maps": [
        {
          "name": "acpmf_crewchief",
          "size": 15660
        }
    ]
}
```
The `game_id` is optional (can be null-ed or omitted) and changes the game reported over dbus (and debug console).  
The `maps` field is required, and needs to have at minimum an empty array.


### Game Status Notification
Once the memory maps are setup the bridge will send out a signal over the dbus, and when the game closes another one.  
This also works for native games.
  
To avoid running a background server the message spec is not inspectable, as usually for dbus services.  
Further, the message is broadcast (to everyone on the session bus), not unicast, and the sender does not have a "well-known" name,
not even fixed name for the same game session.  
  
As such you will have to employ message filtering:
```
path=/com/github/lukaslichten/datalink
interface=com.github.lukaslichten.datalink
Signals:
 - member: StartedPlaying
   args: string game_id
 - member: StoppedPlaying
   args: string game_id
```
  
Additionally, to allow mid-session checks, while the game is running you will find under 
`~/.cache/Datalink/running/` a file with the same name as `game_id`.  
Inside you will find the pid (of the pressure vessel, or the wrapper for native games),
which you can check if they are still running.  
  
This file is created (and deleted) BEFORE the dbus is notified, meaning if your programm first setups a dbus listener,
then checks if the game is already runnning using the file, and then start executing based on that,
you might also receive the signal "StartedPlaying" afterwards.
So your code needs to handle this potential "double impulse", but this is done this way to avoid missing a game launching at the exact same time.

## Building
Requires rust, but also gnu-windows cross compile target:
```
rustup target add x86_64-pc-windows-gnu
```
  
Project can be build via
```
make
```
  
Your lsp might flag the `include_bytes` line, use
```
make debug-build
```
to build the missing bridge.exe in debug, which will fix the missing file.  
  
For debugging you can add the full path to the binary (`[...]/Datalink/target/release/Datalink`)
to the Launch Option of the game you use for testing.  
To get debug output you can cold start steam from the terminal, then the first stage will log into that terminal session
(however the second stage will not, but is currently erroneously spawning a wine-terminal window).

## Related

This project (in large parts) is a fork of [shm-bridge](https://github.com/poljar/shm-bridge).  
  
Other related Projects can be found here: [awesome-linux-simeracing/libraries and dev utils](https://github.com/LukasLichten/awesome-linux-simracing#libraries-headers-and-other-dev-utils)

