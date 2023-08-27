#### Building from source

##### Arch Linux

```
$ git clone https://github.com/Speedy6451/pirates.git
$ cd pirates
# pacman -S --needed wabt minify just binaryen rustup just
$ rustup toolchain add wasm32-unknown-unknown
$ just build
```
