# Carcassonne

The boardgame Carcassonne implemented in Rust using the ggez game engine.

### Options

```
  -f, --fullscreen [<FULLSCREEN>]      Start in fullscreen; optionally provide a resolution to run with that res. [default: 1920x1080]
  -c, --debug-config <DEBUG_CONFIG>    Immediately start a debug game configuration [possible values: meeple-placement, multiple-segments-per-tile-scoring, multiple-player-ownership, rotation-test, group-coallation]
  -s, --snap-placement                 Enable experimental snapping tile placement
  -i, --ip <IP>                        Default multiplayer Ip address
  -p, --port <PORT>                    Default multiplayer port [default: 11069]  
  -g, --ping-interval <PING_INTERVAL>  Ping interval in seconds for multiplayer games [default: 5]
  -v, --save-games [<SAVE_GAMES>]      Enable to save ongoing game progress to this directory [default: saves/]
  -o, --save-logs [<SAVE_LOGS>]        Enable to save logs to this path [default: logs/]
  -e, --log-level <LOG_LEVEL>          Logging level [default: info] [possible values: off, error, warn, info, debug, trace, full]
  -l, --load <LOAD>                    Load a save file
  -d, --debug                          Enables debug mode: increases log level to 'trace', enables saving log files, and enables saving game state
  -h, --help                           Print help
```

### Controls

* Right click to drag
* WASD / Arrow keys to move
* Scroll to zoom
* R to rotate a tile clockwise
* E to rotate counterclockwise
* Left click to place a tile / meeple
* Hold Tab to see detailed game stats
* Space to skip meeple placement
* Esc to pause

## Todo

### For First Release

* ~~Preview clients moving tiles~~
* ~~Inspect groups on tab~~
* ~~Controls cheatsheet on tab~~
* ~~Rules screen~~
* ~~Gui for connecting to host~~

### Additional Features

* ~~Failsafe if tile can't be placed~~
* ~~Main menu~~
* ~~Better game ending animation~~
* ~~Pause Menu~~
* ~~Undo~~
* ~~Controls screen~~
* ~~Choose player colors~~
* ~~Inspect segment groups~~
* ~~Counterclockwise rotation~~
* ~~Snap tile directly to nearest valid square~~
* ~~Multiplayer~~
* ~~Robust logging & debugging functionality~~
* ~~Reconnection for clients who lose connections~~
* ~~Return to lobby after game end instead of main menu~~
* ~~Persistent host ip & port~~
* ~~Usernames~~
* Minecraft-style server browser
* Highlight on most recently placed tile
* More animations in general
* River tiles
* Special gamemodes
  * Procedurally generated tiles
  * Screensaver mode
* Easter egg: replace meeples with emojis / furries / little protogens :3 (furples)
* Use scans of actual tiles
