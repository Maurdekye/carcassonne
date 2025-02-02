# Carcassonne

The boardgame Carcassonne implemented in Rust using the ggez game engine.

### Options

```
  -f, --fullscreen [<FULLSCREEN>]      Start in fullscreen; optionally provide a resolution to run with that res. [default: 1920x1080]
  -d, --debug-config <DEBUG_CONFIG>    Immediately start a debug game configuration [possible values: meeple-placement, multiple-segments-per-tile-scoring, multiple-player-ownership, rotation-test]
  -s, --snap-placement                 Enable experimental snapping tile placement
  -i, --ip <IP>                        Ip address to attempt to connect to a multiplayer game
  -p, --port <PORT>                    Port to host a multiplayer game on / connect to [default: 11069]
  -g, --ping-interval <PING_INTERVAL>  Ping interval in seconds for multiplayer games [default: 5]
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
* Esc to pause

## Todo

### For First Release

* ~~Preview clients moving tiles~~
* ~~Inspect groups on tab~~
* ~~Controls cheatsheet on tab~~
* ~~Rules screen~~
* Gui for connecting to host

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
* Persistent host ip & port / Minecraft-style server browser
* Reconnection for clients who lose connections
* Return to lobby after game end instead of main menu
* Highlight on most recently placed tile
* More animations in general
* River tiles
* Special gamemodes
  * Procedurally generated tiles
  * Screensaver mode
* Easter egg: replace meeples with emojis / furries / little protogens :3 (furples)
* Use scans of actual tiles
