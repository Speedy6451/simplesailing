## Simple Sailing Simulator

my [entry](https://js13kgames.com/entries/simple-sailing-simulator) to js13kgames 2023

> Brave the north wind and search for York, or simply explore.

### Controls
    +/-: zoom
    A: rudder left
    D: rudder right

Your sailboat travels fastest going perpendicular to the wind.


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
