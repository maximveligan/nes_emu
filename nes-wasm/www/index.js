import { memory } from "nes-wasm/nes_wasm_bg";
import { EmuInterface } from "nes-wasm";

const PIXEL_SCALE = 1; // px
const SCREEN_HEIGHT = 240;
const SCREEN_WIDTH = 256;
const SCREEN_SIZE = SCREEN_HEIGHT * SCREEN_WIDTH;
const COLOR_CHANNELS = 3;

var nes_fe;

const canvas = document.getElementById("nes-wasm-canvas");
canvas.height = PIXEL_SCALE * SCREEN_HEIGHT;
canvas.width = PIXEL_SCALE * SCREEN_WIDTH;

const ctx = canvas.getContext('2d');

const drawFrameBuff = (frameBuffPtr, length) => {
    const frameBuffer = new Uint8Array(
        memory.buffer, frameBuffPtr, length);
    var pixelBuffer = ctx.createImageData(SCREEN_WIDTH, SCREEN_HEIGHT);
    for (var i=0; i < SCREEN_SIZE; i++) {
        pixelBuffer.data[(i * 4)] = frameBuffer[i * 3];
        pixelBuffer.data[(i * 4) + 1] = frameBuffer[(i * 3) + 1];
        pixelBuffer.data[(i * 4) + 2] = frameBuffer[(i * 3) + 2];
    // No transparent pixels here
        pixelBuffer.data[(i * 4) + 3] = 0xFF;
    }
    ctx.putImageData(pixelBuffer, 0, 0);
};

const renderLoop = () => {
  const bufferStruct = nes_fe.get_frame();
  drawFrameBuff(bufferStruct.pointer, bufferStruct.length);
  requestAnimationFrame(renderLoop);
};

document.querySelector("#file-input").addEventListener('change', function() {
    var reader = new FileReader();
    reader.onload = function() {
        const romBuffer = new Uint8Array(this.result);

        nes_fe = EmuInterface.new(romBuffer);
        requestAnimationFrame(renderLoop);
    }

    reader.readAsArrayBuffer(this.files[0]);
}, false);
