# Datalink-memmap-config
Provides the config file formate for [Datalink](https://github.com/LukasLichten/Datalink), 
and provides methodes for easily reading and writing these configuration.

The config is on a per Proton Prefix basis, located in `C:\users\steamuser\AppData\Roaming\Datalink\config.json`
(although username might be different if the prefix is configured differently).  
  
With the `proton` feature you can use [proton-finder](https://github.com/LukasLichten/proton-finder)
crate to automatically find the prefix for the game and with it the config file.  
  
It is recommened to always use the write methods instead of force write methods, so multiple programs can contribute to the config file,
and all of them can read the data they want. If a map with the same name already exists, then the larger of the two is kept, all others are kept.  
If the game_id was set (by your config or the existing), then they need to match (and only one being set is also a missmatch).
