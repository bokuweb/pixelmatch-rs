const {
  pixelmatch,
} = require("../../../pixelmatch-simd-wasm/node/cjs/index.cjs");

const res = pixelmatch(
  new Uint8Array([255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
  new Uint8Array([0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
  2,
  2,
  {
    includeAntiAlias: true,
    threshold: 0.2,
  }
);

console.log(res);
