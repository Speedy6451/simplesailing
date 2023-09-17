## Simple Sailing Simulator

my [entry](https://js13kgames.com/entries/simple-sailing-simulator) to js13kgames 2023

![Gameplay screenshot, a pixelated sailboat exits a well-protected bay](https://github.com/Speedy6451/simplesailing/assets/37423245/16b60975-08f3-4f62-b0df-f78ba95454f5)

> Brave the north wind and search for York, or simply explore.

### Controls
|keyboard | controller | action
|---|---|---
|`A`|`D-Left`|rudder left
|`D`|`D-Right`|rudder right
||Left Stick X|rudder
|`+`|`D-Up`|zoom in
|`-`|`D-Down`|zoom out
||Right Stick Y|zoom
|`E`|`B`|raise sails
|`Q`|`A`|lower sails
||Left Stick Y + Left Bumper|control sails
|arrow keys|Right Stick + Right Bumper|pan camera
|`R`|`Y`|reset sailboat
|`/`|`X`|reset camera
|Esc||quit

Your sailboat travels fastest going perpendicular to the wind.

#### Installation

download the [latest release](https://github.com/Speedy6451/simplesailing/releases/latest)

#### Building from source

##### Native

```
cargo run -p client --release
```

##### Wasm (optimized for firefox)

###### Arch Linux

```
# pacman -S --needed wabt minify just binaryen rustup just
$ rustup toolchain add wasm32-unknown-unknown
$ just build
```
