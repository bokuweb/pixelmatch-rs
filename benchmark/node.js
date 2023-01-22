const Benchmark = require("benchmark");
const pixelmatch = require("pixelmatch");
const PNG = require("pngjs").PNG;
const { readFileSync } = require("fs");

const simd = require("../pixelmatch-simd-wasm/node/cjs/index.cjs");
const without = require("../pixelmatch-wasm/node/cjs/index.cjs");

const data = [
  {
    base: "../fixtures/000a.png",
    target: "../fixtures/000b.png",
    w: 7488,
    h: 5242,
  },
  {
    base: "../fixtures/001a.png",
    target: "../fixtures/001b.png",
    w: 800,
    h: 578,
  },
  {
    base: "../fixtures/002a.png",
    target: "../fixtures/002b.png",
    w: 10000,
    h: 8000,
  },
];


let cachedUint8Memory0 = new Uint8Array();
function getUint8Memory0() {
  if (cachedUint8Memory0.byteLength === 0) {
    cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8Memory0;
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
  const ptr = malloc(arg.length * 1);
  getUint8Memory0().set(arg, ptr / 1);
  WASM_VECTOR_LEN = arg.length;
  return ptr;
}

const inner = function (
  img1,
  img2,
  out,
  width,
  height,
  include_anti_alias,
  threshold,
  diff_color_r,
  diff_color_g,
  diff_color_b,
  diff_color_a,
  anti_aliased_color_r,
  anti_aliased_color_g,
  anti_aliased_color_b,
  anti_aliased_color_a
) {
  try {
    const ptr0 = passArray8ToWasm0(img1, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(img2, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    var ptr2 = passArray8ToWasm0(out, wasm.__wbindgen_malloc);
    var len2 = WASM_VECTOR_LEN;
    const ret = wasm.pixelmatch(
      ptr0,
      len0,
      ptr1,
      len1,
      ptr2,
      len2,
      width,
      height,
      include_anti_alias,
      threshold,
      diff_color_r,
      diff_color_g,
      diff_color_b,
      diff_color_a,
      anti_aliased_color_r,
      anti_aliased_color_g,
      anti_aliased_color_b,
      anti_aliased_color_a
    );
    return ret;
  } catch (e) {
    console.error(e);
  } finally {
    out.set(getUint8Memory0().subarray(ptr2 / 1, ptr2 / 1 + len2));
    wasm.__wbindgen_free(ptr2, len2 * 1);
  }
};

const createDefaultOptions = () => {
  return {
    includeAntiAlias: false,
    threshold: 0.1,
    diffColor: [255, 119, 119, 255],
    antiAliasedColor: [243, 156, 18, 255],
  };
};

const simd_without_glue = (img1, img2, w, h, opts) => {
  const out = new Uint8Array(img1.length);
  const defaultOptions = createDefaultOptions();
  const diffColor = opts.diffColor ?? defaultOptions.diffColor;
  const antiAliasedColor =
    opts.antiAliasedColor ?? defaultOptions.antiAliasedColor;
  const countOrError = inner(
    img1,
    img2,
    out,
    w,
    h,
    opts.includeAntiAlias ?? defaultOptions.includeAntiAlias,
    opts.threshold ?? defaultOptions.threshold,
    diffColor[0],
    diffColor[1],
    diffColor[2],
    diffColor[3],
    antiAliasedColor[0],
    antiAliasedColor[1],
    antiAliasedColor[2],
    antiAliasedColor[3]
  );
  return { count: countOrError, diff: out };
};

const path = require("path").join(
  __dirname,
  "../pixelmatch-simd-wasm/dist/node/pkg/pixelmatch_simd_wasm_bg.wasm"
);
const bytes = require("fs").readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, {});
wasm = wasmInstance.exports;

(async () => {
  data.forEach(({ target, base, w, h }) => {
    const suite = new Benchmark.Suite("simd");
    const img1 = PNG.sync.read(readFileSync(base)).data;
    const img2 = PNG.sync.read(readFileSync(target)).data;
    suite
      .add("js", {
        fn: () => {
          const out = new Uint8Array(img1.length);
          pixelmatch(img1, img2, out, w, h, { includeAA: false });
        },
      })
      .add("default", {
        fn: () =>
          without.pixelmatch(img1, img2, w, h, {
            includeAntiAlias: false,
          }),
      })
      .add("without glue", {
        fn: () => simd_without_glue(img1, img2, w, h, { includeAntiAlias: false }),
      })
      .add("simd", {
        fn: () =>
          simd.pixelmatch(img1, img2, w, h, { includeAntiAlias: false }),
      })
      .on("complete", () => {
        console.log(
          "Fastest is " + suite.filter("fastest").map("name"),
          "js",
          suite[0].stats.mean,
          "default",
          suite[1].stats.mean,
          "without glue",
          suite[2].stats.mean,
          "simd",
          suite[3].stats.mean
        );
      })
      .run();
  });
})().catch((e) => console.error(e));
