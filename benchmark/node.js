const Benchmark = require("benchmark");
const pixelmatch = require("pixelmatch");
const PNG = require("pngjs").PNG;
const { readFileSync } = require("fs");

const simd = require("../pixelmatch-simd-wasm/node/cjs/index.cjs");
const without = require("../pixelmatch-wasm/node/cjs/index.cjs");

(async () => {
  const suite = new Benchmark.Suite("simd");
  const img1 = PNG.sync.read(
    readFileSync("../pixelmatch/examples/4a.png")
  ).data;
  const img2 = PNG.sync.read(
    readFileSync("../pixelmatch/examples/4b.png")
  ).data;
  suite
    .add("without", {
      fn: () =>
        without.pixelmatch(img1, img2, 800, 578, { includeAntiAlias: true }),
    })
    .add("simd", {
      fn: () =>
        simd.pixelmatch(img1, img2, 800, 578, { includeAntiAlias: true }),
    })
    .add("js", {
      fn: () => {
        const out = new Uint8Array(img1.length);
        pixelmatch(img1, img2, out, 800, 578, { includeAA: true });
      },
    })
    .on("complete", () => {
      console.log(
        "Fastest is " + suite.filter("fastest").map("name"),
        suite[0].stats,
        suite[1].stats,
        suite[2].stats
      );
    })
    .run();
})();
