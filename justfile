alias b := build
alias r := run
alias c := check-zip

build: minify-js minify-rust minify-html
    cp front/style.css build/style.css

build-rust:
    cargo build --manifest-path pirates/Cargo.toml --target wasm32-unknown-unknown --features wasm --release
    cp target/wasm32-unknown-unknown/release/pirates.wasm front/index.wasm

minify-rust: build-rust build-dir
    wasm-strip front/index.wasm
    wasm-opt -o build/index.wasm -Oz front/index.wasm
   
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

    @cat build/release.zip | wc -c | xargs -I {} python3 -c "print(str(round({}/(13*1024)*100,2))+'%')"

zip: build
    cd build; zip -r release.zip * -x release.zip

run: build
    python3 -m http.server &
    firefox http://0.0.0.0:8000/build/index.html

clean:
    cargo clean --manifest-path pirates/Cargo.toml
    rm -r build
