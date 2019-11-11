import { memory } from "nes-wasm/nes_wasm_bg";
import { EmuInterface, KeyCode } from "nes-wasm";

const SCREEN_HEIGHT = 240;
const SCREEN_WIDTH = 256;
const SCREEN_SIZE = SCREEN_HEIGHT * SCREEN_WIDTH;
const COLOR_CHANNELS = 3;

var nes_fe = null;
let animationId = null;

const playPauseButton = document.getElementById("play-pause");
const canvas = document.getElementById("nes-wasm-canvas");

// Need to get actual scaling working here
const scale = 3;

canvas.width = SCREEN_WIDTH * scale;
canvas.height = SCREEN_HEIGHT * scale;

const ctx = canvas.getContext('2d');
playPauseButton.textContent = "⏸";

const play = () => {
  playPauseButton.textContent = "⏸";
  renderLoop();
};

const pause = () => {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
};

playPauseButton.addEventListener("click", event => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

document.addEventListener('keydown', function(event) {
    if (nes_fe != null) {
        nes_fe.set_button(KeyCode.new(event.keyCode), true);
    }
});

document.addEventListener('keyup', function(event) {
    if (nes_fe != null) {
        nes_fe.set_button(KeyCode.new(event.keyCode), false);
    }
});

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
    ctx.drawImage(ctx.canvas, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0, 0, canvas.width, canvas.height);
};

const isPaused = () => {
  return animationId === null;
};

const renderLoop = () => {
  const bufferStruct = nes_fe.get_frame();
  drawFrameBuff(bufferStruct.pointer, bufferStruct.length);
  animationId = requestAnimationFrame(renderLoop);
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
