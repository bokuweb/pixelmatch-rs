import path from "path";
import fs from "fs";

import { fileURLToPath } from "url";
import {
  createDefaultOptions,
  InvalidFormatError,
  ImageLengthError,
} from "../../common/esm/index.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log(__dirname);

const wasmPath = path.join(
  __dirname,
  "../../dist/node/pkg/pixelmatch_simd_wasm_bg.wasm"
);

const bytes = fs.readFileSync(wasmPath);

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
/**
 * @param {Uint8Array} img1
 * @param {Uint8Array} img2
 * @param {Uint8Array} out
 * @param {number} width
 * @param {number} height
 * @param {boolean} include_anti_alias
 * @param {number} threshold
 * @param {number} diff_color_r
 * @param {number} diff_color_g
 * @param {number} diff_color_b
 * @param {number} diff_color_a
 * @param {number} anti_aliased_color_r
 * @param {number} anti_aliased_color_g
 * @param {number} anti_aliased_color_b
 * @param {number} anti_aliased_color_a
 * @returns {number}
 */
function inner(
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
  } finally {
    out.set(getUint8Memory0().subarray(ptr2 / 1, ptr2 / 1 + len2));
    wasm.__wbindgen_free(ptr2, len2 * 1);
  }
}

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;

export const pixelmatch = (img1, img2, w, h, opts) => {
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

  if (countOrError < 0) {
    if (countOrError === -1) {
      throw new ImageLengthError();
    } else if (countOrError === -2) {
      throw new InvalidFormatError();
    }
  }

  return { count: countOrError, diff: out };
};
