# Datalink
Simple wrapper around Steam/Proton Games that can deploy a memorymap bridge and launch further application from a config file within the prefix

## Usage
Apppend Datalink infront of the launch command (same way as Mangohud) by setting the Launch Options on Steam like this:
```
Datalink %command%
```

You are free to combine this with Mangohud and or regular launch options.  
This works even with Linux native steam games, although no memorymaps or apps will be setup for those (but the launch is still communicated).

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

### Autolaunching Apps
But we can also launch windows apps within the prefix alongside our game. 
This is useful if there isn't a linux native version of the software yet, and circumfents some issues listed above.  

The software is launched after the memory maps have been created, and before the game is launched.  
Software is launched in order (but can not garantee there is enough spacing, so using a script and launching that might be recommended).  
  
`taskkill` is used to gracefully shutdown the apps after the game exits (which send `w_close` to all windows of the application).
Some apps will then hide into the tray, and will be (like stuck tasks) after 5s terminated through use of the `/f` flag.  
  
Additionally a single `post_app` can be configured, allowing you to do clean up after the game (and apps exited).  

### Configuring The Bridge
The config is on a per Proton Prefix basis, located in `C:\users\steamuser\AppData\Roaming\Datalink\[name].json`
(although username might be different if the prefix is configured differently).  
  
You can use the, present in this repository, config crate `datalink-bridge-config`,
which contains the structs and function for deserialzing the crate.  
Additionally with the `proton` feature you can use [proton-finder](https://github.com/LukasLichten/proton-finder)
crate to automatically find the prefix for the game and with it the config file.  
  
As multiple programms could want different configurations, the bridge reads all json in the folder and merges them together.  
This occures on basic rules: 
- maps with the same name the larger is used
- apps are merged if path and args match completly, otherwise it keeps both commands
- if different game_id's are set, then all game_id's will be notified over the dbus
 - Including the default, if one or more config is unset/null
- root_mount_point the first none Z value that is read is used, any missmatches will result in them being logged
 - Order Is NOT necesaarily alphabetic
 - Errors are only logged, but ignored
  
To avoid conflict with other config files, it is best practice to set your config to a unique name, 
for example reverse domain: `com.github.lukaslichten.datalink.json`  
  
In case of manual writing (e.g. other programming language), this is an example config:
```
{
    "game_id":"override",
    "maps": [
        {
          "name": "acpmf_crewchief",
          "size": 15660
        }
    ],
    "root_mount_point": "L",
    "apps": [
        {
          "path": "C:\\users\\steamuser\\Documents\\Little Navconnect\\littlenavconnect.exe",
          "args": []
        }
    ],
    "post_app": {
        "path": "del",
        "args": [
            "C:\\users\\steamuser\\AppData\\Roaming\\running"
        ]
    },
    "notes": "v1"
}
```
All fields are optional (except for contained structs):  
 - `game_id` changes the game reported over dbus (and debug console), omitting it or setting to null will use the value from steam. As shown, doesn't have to be a number, can be any valid string
 - `maps` has to be an array (or ommited), each map MUST contain a `name` (used by windows and then also in `/dev/shm`) and a `size`.  
 - `root_mount_point` optionally sets the letter ot override the default `Z:\` mount point that wine uses to mount in the linux filesystem (in case of an unusal wine prefix).
 - `apps` has to be and array (or ommitted), each app MUST contain a `path`
 (path to the executable, either an absolute linux or windows path (remember, json also uses `\` as escape character, so `\\` above is only one, and the correct way of doing it),
 or a relative path relative to the `AppData\Roaming\Datalink` folder) and an args array
 - `post_app` is a single app struct (as defined above), or null/omitted
 - `notes` an additional field, should contain a string. Is not read by the bridge, and only used by you to for example note a version number for this config.


### Game Status Notification
Once the memory maps are setup the bridge will send out a signal over the dbus, and when the game closes another one.  
This also works for native games.
  
To avoid running a background server the message spec is not inspectable, as usually for dbus services.  
Further, the message is broadcast (to everyone on the session bus), aka not unicast, and the sender does not have a "well-known" name,
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

