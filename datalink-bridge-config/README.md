# Datalink-bridge-config
Provides the config file formate for [Datalink](https://github.com/LukasLichten/Datalink), 
and provides methodes for easily reading and writing these configuration.

The config is on a per Proton Prefix basis, located in `C:\users\steamuser\AppData\Roaming\Datalink\[name].json`
(although username might be different if the prefix is configured differently).  
  
With the `proton` feature you can use [proton-finder](https://github.com/LukasLichten/proton-finder)
crate to automatically find the prefix for the game and with it the config file.  
  
As multiple programms could want different configurations, the bridge reads all json in the folder and merges them together.  
This occures on basic rules: 
- maps with the same name the larger is used
- apps are merged if path and args match completly, otherwise it keeps both commands
- post_apps are merged with the same rules as apps
- if different game_id's are set, then all game_id's will be notified over the dbus
 - Including the default, if one or more config is unset/null
- root_mount_point the first none Z value that is read is used, any missmatches will result in them being logged
 - Order Is NOT necesaarily alphabetic
 - Errors are only logged, but ignored
  
To avoid conflict with other config files, it is best practice to set your config to a unique name, 
for example reverse domain: `com.github.lukaslichten.datalink.json`  
  
`notes` field is deserialized in a way, that if there is an error (someone set it to an int for example)
then instead of failing deserialization, it instead sets the field to None.  
Also the read_folder functions do not merge the notes (but may include the `notes` from the first config it read).

