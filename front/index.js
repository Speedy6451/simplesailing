var ctx;
var image;
var memory;
const width = 256;
const height = 224;

function blit_frame() {
    ctx.putImageData(image, 0, 0);
}

function blit_text(text, len, x, y, size) {
    let decoded = (new TextDecoder())
        .decode(new Uint8Array(memory.buffer, text, len));
    ctx.font = size +'vh serif'
    ctx.fillText(decoded,x,y);
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

    memory = instance.exports.memory
    const buffer_address = instance.exports.BUFFER.value;
    image = new ImageData(
        new Uint8ClampedArray(
            memory.buffer,
            buffer_address,
            4 * width * height,
        ),
        width,
    );

    instance.exports.frame_entry();
    ctx.textBaseline = 'top'
    ctx.textAlign = 'left';

    const render = () => {
        instance.exports.frame_entry();

        requestAnimationFrame(render);
    }

    render();
}

init();
