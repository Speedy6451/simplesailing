alias b := build
alias r := run
alias c := check-zip

build: minify-js minify-rust minify-html

build-rust:
    cargo build --manifest-path pirates/Cargo.toml --target wasm32-unknown-unknown --release

minify-rust: build-rust build-dir
    wasm-strip pirates/target/wasm32-unknown-unknown/release/pirates.wasm
    wasm-opt -o build/pirates.wasm -Oz pirates/target/wasm32-unknown-unknown/release/pirates.wasm
   
minify-js: build-dir
    #minify-js -m module --output build/index.js front/index.js
    minify front/index.js > build/index.js

minify-html: build-dir
    minify front/index.html > build/index.html

[private]
build-dir:
    mkdir -p build

check-size: build
    dust -s build

check-zip: zip
    unzip -v build/release.zip | awk '{printf ("%5s\t%s\n", $3,  $8)}'

    @cat build/release.zip | wc -c | xargs -I {} python3 -c "print(str(round({}/(13*1024),2))+'%')"

zip: build
    zip -r build/release.zip build -x release.zip

run: build
    python3 -m http.server &
    firefox http://0.0.0.0:8080/build/index.html

clean:
    cargo clean --manifest-path pirates/Cargo.toml
    rm -r build
