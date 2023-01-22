import { pixelmatch } from "@bokuweb/pixelmatch-wasm";

console.log(
  pixelmatch(
    new Uint8Array([255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    new Uint8Array([255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    2,
    2
  )
);
