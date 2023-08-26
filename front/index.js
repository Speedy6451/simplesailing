async function init() {
    const { instance } = await WebAssembly.instantiateStreaming(
        fetch("./index.wasm")
    );

    const width = 600;
    const height = 600;

    const canvas = document.getElementById("window");
    canvas.width = width;
    canvas.height = height;

    const buffer_address = instance.exports.BUFFER.value;
    const image = new ImageData(
        new Uint8ClampedArray(
            instance.exports.memory.buffer,
            buffer_address,
            4 * width * height,
        ),
        width,
    );

    const ctx = canvas.getContext("2d");

    const render = () => {
        instance.exports.frame_entry();
        ctx.putImageData(image, 0, 0);
        ctx.font = '84px sans-serif'
        ctx.textBaseline = 'top'
        ctx.textAlign = 'left';
        ctx.fillText("demo",12,12);

        requestAnimationFrame(render);
    }

    render();
}

init();
