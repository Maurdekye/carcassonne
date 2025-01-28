# Carcassonne

The boardgame Carcassonne implemented in Rust using the ggez game engine.

### Options

 * `--fullscreen` / `-f`: Start in fullscreen. Provide a resolution to start in that fullscreen resolution. Default 1920x1080
 * `--debug <CONFIG>` / `-d <CONFIG>`: Open up immediately into a debug game configuration. Run with no value to see available configs.
 * `--snap-placement` / `-s`: Enable snapping tile placement.

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
* Multiplayer
* More animations in general
* River tiles
* Use scans of actual tiles
