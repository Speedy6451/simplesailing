var ctx;
var image;
var memory;
var exports;
const width = 160;
const height = 144;
const TIME_GAIN = 0.1; // game is unplayable at full speed with wasm refresh rates

function blit_frame() {
    // this is required when using an allocator from wasm as there is 
    // no way to update the internal pointer when linear memory shifts
    image = new ImageData(
        new Uint8ClampedArray(
            memory.buffer,
            exports.BUFFER.value,
            4 * width * height,
        ),
        width,
    );

    ctx.putImageData(image, 0, 0);
}

function blit_text(text, len, x, y, size) {
    let decoded = (new TextDecoder())
        .decode(new Uint8Array(memory.buffer, text, len));
    ctx.font = size +'px serif'
    ctx.fillText(decoded,x,y);
}

function keyboard_callback(e) {
    const keycode_address = exports.KEYCODE.value;
    const value = new Uint8ClampedArray(exports.memory.buffer, keycode_address, 2);

    value[0] = e.keyCode;

    exports.keyboard_input();
}

async function init() {
    const canvas = document.getElementById("window");
    canvas.width = width;
    canvas.height = height;

    ctx = canvas.getContext("2d");

    const { instance } = await WebAssembly.instantiateStreaming(
        fetch("./index.wasm"),
        {
            "env" : {
                "js_sin": Math.sin,
                "blit_frame": blit_frame,
                "blit_text": blit_text,
            },
        }
    );

    exports = instance.exports;
    document.getElementById("body").onkeydown=keyboard_callback;

    memory = instance.exports.memory
    instance.exports.frame_entry();
    ctx.textBaseline = 'top'
    ctx.textAlign = 'left';

    var last;
    var elapsed;

    const render = (time) => {
        if(!last) { elapsed = 0; }
        else {
            elapsed = time-last;
        }
        last=time;

        if(!elapsed) {
            elapsed = 0.0;
        }

        const FRAME_TIME = new Float32Array(exports.memory.buffer, exports.LAST_FRAME_TIME, 1);
        FRAME_TIME[0] = elapsed * TIME_GAIN;

        instance.exports.frame_entry();

        requestAnimationFrame(render);
    }
    render();
}

init();
