const Benchmark = require("benchmark");
const pixelmatch = require("pixelmatch");
const PNG = require("pngjs").PNG;
const { readFileSync } = require("fs");

const simd2 = require("../pixelmatch-simd-wasm/node/cjs/index.cjs");
const without = require("../pixelmatch-wasm/node/cjs/index.cjs");

let imports = {};
let wasm;

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

const simd = (img1, img2, w, h, opts) => {
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
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;

(async () => {
  const suite = new Benchmark.Suite("simd");
  const img1 = PNG.sync.read(readFileSync("../fixtures/002a.png")).data;
  const img2 = PNG.sync.read(readFileSync("../fixtures/002b.png")).data;
  suite
    // .add("without", {
    //   fn: () =>
    //     without.pixelmatch(img1, img2, 7488, 5242, { includeAntiAlias: true }),
    // })
    .add("simd", {
      fn: () => simd(img1, img2, 10000, 8000, { includeAntiAlias: false }),
    })
    .add("js", {
      fn: () => {
        const out = new Uint8Array(img1.length);
        pixelmatch(img1, img2, out, 10000, 8000, { includeAA: false });
      },
    })
    .add("simd2", {
      fn: () =>
        simd2.pixelmatch(img1, img2, 10000, 8000, { includeAntiAlias: false }),
    })
    .on("complete", () => {
      console.log(
        "Fastest is " + suite.filter("fastest").map("name"),
        suite[0].stats,
        suite[1].stats,
        suite[2].stats
        // suite[3].stats
      );
    })
    .run();
})().catch((e) => console.error(e));
