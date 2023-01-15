import { pxmatch } from "../../../pixelmatch-simd-wasm/dist/web/pkg/pixelmatch_simd_wasm_bg";

console.log(
  pxmatch(
    new Uint8Array([255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    new Uint8Array([0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    2,
    2
  )
);
