const PNG = require("pngjs").PNG;
const { readFileSync } = require("fs");

const simd = require("../pixelmatch-simd-wasm/node/cjs/index.cjs");

const data = {
  base: "../fixtures/001a.png",
  target: "../fixtures/001b.png",
  w: 800,
  h: 578,
};

(async () => {
  const { base, target, w, h } = data;
  const img1 = PNG.sync.read(readFileSync(base)).data;
  const img2 = PNG.sync.read(readFileSync(target)).data;
  simd.pixelmatch(img1, img2, w, h, {
    includeAntiAlias: false,
    threshold: 0.1,
  });
})();
